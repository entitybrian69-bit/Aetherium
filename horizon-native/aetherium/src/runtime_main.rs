// Aetherium's game side, added to Horizon's game as `game::aetherium_rt`: the zoom, waypoints, the
// info overlay and the markers, and the key and command that open Aetherium's screen. The screen
// itself is `horizon_client::options_ui::aetherium` (gui.rs); the two share its `SHARED` state.
// The zoom and waypoint maths are the `.hmod`'s (horizon/aetherium/src), copied in below by
// horizon-native/tools/assemble.py, which writes mixins/aetherium/runtime.rs from this file.

use std::cell::RefCell;
use std::time::Duration;

use horizon_client::extras::Corner;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::options_ui::aetherium as gui;

use super::native_hooks::{NativeMod, NativeZoom};
use super::{Game, Screen};

const DESCRIPTION: &str = "Aetherium's settings screen (Video Settings > Aetherium Settings...), smooth zoom, \
     waypoints with on-screen markers and an info overlay.";

// Colours (RGB) of the overlay and markers, as the Java mod and the .hmod use them.
const GOLD: u32 = 0xFFAA00;
const RED: u32 = 0xFF5555;
const GREEN: u32 = 0x55FF55;
const YELLOW: u32 = 0xFFFF55;
const WHITE: u32 = 0xFFFFFF;
const AQUA: u32 = 0x55FFFF;
const NETHER_RED: u32 = 0xFF7755;
const PALETTE: [u32; 6] = [0x55FFFF, 0xFFAA00, 0x55FF55, 0xFF55FF, 0xFFFF55, 0x5555FF];
const PANEL: u32 = 0x9000_0000;
const OUTLINE: u32 = 0xC000_0000;
const HUD_WAYPOINTS: usize = 5;
const UNBOUND: &str = "key.keyboard.unknown";

struct Rt {
    config: gui::Config,
    config_gen: u64,
    fov: f32,
    zoom: zoom::Zoom,
    view: zoom::View,
    waypoints: waypoints::Waypoints,
    /// The list changed: save and show it.
    waypoints_changed: bool,
    /// Zoom, zoom in, zoom out held.
    held: [bool; 3],
    add_pressed: bool,
    /// Open the screen at this tab on the next frame while playing.
    open: Option<usize>,
    dead: bool,
}

thread_local! {
    static RT: RefCell<Option<Rt>> = const { RefCell::new(None) };
}

fn zoom_config(c: &gui::Config, fov: f32) -> zoom::ZoomConfig {
    zoom::ZoomConfig {
        toggle: c.zoom_toggle,
        default_zoom: f64::from(c.zoom_default) / 100.0,
        smooth_s: f64::from(c.zoom_smooth_ms) / 1000.0,
        scrolling: c.zoom_scroll,
        mouse_floor: f64::from(c.zoom_mouse) / 100.0,
        fov: f64::from(fov),
    }
}

fn block(v: f64) -> i32 {
    v.floor() as i32
}

fn fps_colour(fps: u32) -> u32 {
    if fps >= 60 {
        GREEN
    } else if fps >= 30 {
        YELLOW
    } else {
        RED
    }
}

fn corner(i: i32) -> Corner {
    match i {
        1 => Corner::TopRight,
        2 => Corner::BottomLeft,
        3 => Corner::BottomRight,
        _ => Corner::TopLeft,
    }
}

impl Rt {
    fn load(game: &Game<'_>) -> Rt {
        let config = game
            .native_setting(gui::ID, "config")
            .map_or(gui::Config::DEFAULT, |t| gui::Config::from_text(&t));
        let list = game
            .native_setting(gui::ID, "waypoints")
            .map(|t| waypoints::Waypoints::from_text(&t))
            .unwrap_or_default();
        let fov = game.settings.fov;
        let config_gen = gui::with(|sh| {
            sh.config = config;
            sh.loaded = true;
            sh.config_dirty = false;
            sh.config_gen += 1;
            sh.config_gen
        });
        let mut rt = Rt {
            config,
            config_gen,
            fov,
            zoom: zoom::Zoom::new(zoom_config(&config, fov)),
            view: zoom::View::NONE,
            waypoints: list,
            waypoints_changed: false,
            held: [false; 3],
            add_pressed: false,
            open: None,
            dead: false,
        };
        rt.publish_waypoints();
        rt
    }

    fn publish_waypoints(&mut self) {
        let rows: Vec<gui::WaypointRow> = self
            .waypoints
            .list
            .iter()
            .map(|w| gui::WaypointRow {
                name: w.name.clone(),
                dimension: w.dimension.clone(),
                x: w.x,
                y: w.y,
                z: w.z,
                shown: w.shown,
            })
            .collect();
        gui::with(|sh| {
            sh.waypoints = rows;
            sh.waypoints_gen += 1;
        });
    }

    /// Saves where the player stands; the chat line to show.
    fn add_here(&mut self, game: &Game<'_>, name: Option<&str>) -> String {
        let Some(p) = game.native_player() else {
            return "Not in a world".into();
        };
        let name = name
            .map(waypoints::clean_name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| self.waypoints.next_name());
        let wp = waypoints::Waypoint {
            name: name.clone(),
            dimension: p.dimension.to_owned(),
            x: block(p.x),
            y: block(p.y),
            z: block(p.z),
            shown: true,
        };
        let text = format!(
            "Waypoint {} saved at {} {} {} ({})",
            name,
            wp.x,
            wp.y,
            wp.z,
            waypoints::dimension_name(&wp.dimension)
        );
        match self.waypoints.set(wp) {
            Ok(_) => {
                self.waypoints_changed = true;
                text
            }
            Err(e) => e,
        }
    }

    /// Shown waypoints converted to `dimension`, at the block's centre: (index, x, y, z).
    fn here(&self, dimension: &str) -> Vec<(usize, f64, f64, f64)> {
        self.waypoints
            .list
            .iter()
            .enumerate()
            .filter(|(_, w)| w.shown)
            .filter_map(|(i, w)| {
                let (x, z) = waypoints::convert(
                    &w.dimension,
                    f64::from(w.x) + 0.5,
                    f64::from(w.z) + 0.5,
                    dimension,
                )?;
                Some((i, x, f64::from(w.y) + 0.5, z))
            })
            .collect()
    }

    fn wp_command(&mut self, game: &Game<'_>, args: &str) -> String {
        let (verb, rest) = match args.trim().split_once(' ') {
            Some((v, r)) => (v, r.trim()),
            None => (args.trim(), ""),
        };
        let pick = |w: &waypoints::Waypoints, rest: &str| {
            if rest.is_empty() {
                return Err("Name a waypoint, or give its number from /wp list".to_string());
            }
            w.find(rest).ok_or_else(|| format!("No waypoint called {rest}"))
        };
        match verb {
            "" => {
                self.open = Some(gui::TAB_WAYPOINTS);
                String::new()
            }
            "add" | "set" => self.add_here(game, (!rest.is_empty()).then_some(rest)),
            "del" | "delete" | "remove" => match pick(&self.waypoints, rest) {
                Ok(i) => match self.waypoints.remove(i) {
                    Some(w) => {
                        self.waypoints_changed = true;
                        format!("Waypoint {} deleted", w.name)
                    }
                    None => "No such waypoint".into(),
                },
                Err(e) => e,
            },
            "show" | "hide" => match pick(&self.waypoints, rest) {
                Ok(i) => {
                    let shown = verb == "show";
                    self.waypoints.list[i].shown = shown;
                    self.waypoints_changed = true;
                    format!(
                        "Waypoint {} {}",
                        self.waypoints.list[i].name,
                        if shown { "shown" } else { "hidden" }
                    )
                }
                Err(e) => e,
            },
            "list" => {
                if self.waypoints.list.is_empty() {
                    return "No waypoints yet: /wp add <name>".into();
                }
                let all: Vec<String> = self
                    .waypoints
                    .list
                    .iter()
                    .enumerate()
                    .map(|(i, w)| {
                        format!(
                            "#{} {} ({} {} {} {}{})",
                            i + 1,
                            w.name,
                            w.x,
                            w.y,
                            w.z,
                            waypoints::dimension_name(&w.dimension),
                            if w.shown { "" } else { ", hidden" }
                        )
                    })
                    .collect();
                all.join("; ")
            }
            "clear" => {
                let n = self.waypoints.list.len();
                self.waypoints.list.clear();
                self.waypoints_changed = true;
                format!("Deleted {n} waypoints")
            }
            _ => "/wp [add <name> | del <name> | show <name> | hide <name> | list | clear]".into(),
        }
    }
}

/// Runs `f` on the runtime, loading it on first use.
fn with_rt<R>(game: &Game<'_>, f: impl FnOnce(&mut Rt) -> R) -> R {
    RT.with(|cell| {
        let mut cell = cell.borrow_mut();
        let rt = cell.get_or_insert_with(|| Rt::load(game));
        f(rt)
    })
}

/// Takes the runtime out for a hook that needs the game mutably; `put` puts it back.
fn take(game: &Game<'_>) -> Rt {
    RT.with(|cell| cell.borrow_mut().take())
        .unwrap_or_else(|| Rt::load(game))
}

fn put(rt: Rt) {
    RT.with(|cell| *cell.borrow_mut() = Some(rt));
}

/// Opens Aetherium's screen over the world at `tab`.
fn open_screen(game: &mut Game<'_>, tab: usize) {
    let Some(i) = game
        .settings
        .mods
        .installed
        .iter()
        .position(|m| m.id == gui::ID && m.native.is_some())
    else {
        return;
    };
    gui::with(|sh| sh.open_tab = Some(tab));
    game.capture(false);
    game.open_options_at(false, horizon_client::options_ui::Page::NativeMod(i));
    game.options_to_game = true;
}

/// Every frame: settings from the screen, the screen's requests, the death point, the zoom.
pub(super) fn frame(game: &mut Game<'_>, elapsed: Duration) {
    if !game.native_on(gui::ID) {
        return;
    }
    let mut rt = take(game);
    let playing = game.native_playing();
    let fov = game.settings.fov;
    let (cfg, generation, dirty, actions) = gui::with(|sh| {
        (
            sh.config,
            sh.config_gen,
            std::mem::take(&mut sh.config_dirty),
            std::mem::take(&mut sh.actions),
        )
    });
    if dirty {
        game.native_set_setting(gui::ID, "config", &cfg.to_text());
    }
    if generation != rt.config_gen || (fov - rt.fov).abs() > 1e-3 {
        rt.config = cfg;
        rt.config_gen = generation;
        rt.fov = fov;
        rt.zoom.config = zoom_config(&cfg, fov);
    }
    let mut chat: Vec<String> = Vec::new();
    for action in actions {
        match action {
            gui::Action::AddHere => chat.push(rt.add_here(game, None)),
            gui::Action::SetShown(name, shown) => {
                if let Some(w) = rt.waypoints.list.iter_mut().find(|w| w.name == name)
                    && w.shown != shown
                {
                    w.shown = shown;
                    rt.waypoints_changed = true;
                }
            }
            gui::Action::DeleteAll => {
                if !rt.waypoints.list.is_empty() {
                    let n = rt.waypoints.list.len();
                    rt.waypoints.list.clear();
                    rt.waypoints_changed = true;
                    chat.push(format!("Deleted {n} waypoints"));
                }
            }
        }
    }
    if std::mem::take(&mut rt.add_pressed) {
        chat.push(rt.add_here(game, None));
    }
    // The death point, when the death screen opens.
    let dead = game.screen == Screen::Dead;
    if dead
        && !rt.dead
        && rt.config.wp_death
        && let Some(p) = game.native_player()
    {
        let wp = waypoints::Waypoint {
            name: waypoints::DEATH.into(),
            dimension: p.dimension.to_owned(),
            x: block(p.x),
            y: block(p.y),
            z: block(p.z),
            shown: true,
        };
        let text = format!(
            "Death point saved at {} {} {} ({}). /wp del Death removes it.",
            wp.x,
            wp.y,
            wp.z,
            waypoints::dimension_name(&wp.dimension)
        );
        if rt.waypoints.set(wp).is_ok() {
            rt.waypoints_changed = true;
            chat.push(text);
        }
    }
    rt.dead = dead;
    if !playing {
        rt.held = [false; 3];
    }
    rt.zoom
        .frame(elapsed.as_secs_f64(), rt.held[0], rt.held[1], rt.held[2]);
    rt.view = rt.zoom.view();
    if std::mem::take(&mut rt.waypoints_changed) {
        game.native_set_setting(gui::ID, "waypoints", &rt.waypoints.to_text());
        rt.publish_waypoints();
    }
    let fps = game.native_frame_rate();
    let in_world = game.native_player().is_some();
    gui::with(|sh| {
        sh.fps = fps;
        sh.in_world = in_world;
    });
    let open = if playing { rt.open.take() } else { None };
    put(rt);
    for line in chat {
        game.native_chat(&line);
    }
    if let Some(tab) = open {
        open_screen(game, tab);
    }
}

/// A key went down or up while playing.
pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !game.native_on(gui::ID) || key == UNBOUND {
        return;
    }
    let bound = gui::KEYS.map(|[name, _, default]| game.native_mod_key(gui::ID, name, default));
    let playing = game.native_playing();
    with_rt(game, |rt| {
        for (i, b) in bound.iter().enumerate() {
            if b != key {
                continue;
            }
            match i {
                0..=2 => rt.held[i] = pressed && playing,
                3 if pressed && playing => rt.add_pressed = true,
                4 if pressed && playing => rt.open = Some(0),
                _ => {}
            }
        }
    });
}

/// Wheel steps while playing: the zoom takes them while zoomed.
pub(super) fn wheel(game: &Game<'_>, steps: f32, claimed: &mut bool) {
    if *claimed || !game.native_on(gui::ID) {
        return;
    }
    if with_rt(game, |rt| rt.zoom.scroll(f64::from(steps))) {
        *claimed = true;
    }
}

/// The zoom in effect.
pub(super) fn zoom(game: &Game<'_>, zoom: &mut Option<NativeZoom>) {
    if zoom.is_some() || !game.native_on(gui::ID) {
        return;
    }
    let view = RT.with(|cell| cell.borrow().as_ref().map(|rt| rt.view));
    if let Some(v) = view.filter(|v| v.factor > 1.0 + 1e-4) {
        *zoom = Some(NativeZoom {
            factor: v.factor as f32,
            smooth: false,
            turn: v.turn as f32,
        });
    }
}

/// The info overlay and the waypoint markers.
pub(super) fn hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    if !game.native_on(gui::ID) {
        return;
    }
    let Some(p) = game.native_player() else {
        return;
    };
    let (cfg, view, here, names) = with_rt(game, |rt| {
        let here = rt.here(p.dimension);
        let names: Vec<String> = here.iter().map(|h| rt.waypoints.list[h.0].name.clone()).collect();
        (rt.config, rt.view, here, names)
    });
    let fov = f64::from(game.settings.fov);
    if cfg.wp_markers && !here.is_empty() {
        let cam = waypoints::Camera {
            x: p.x,
            y: p.y + waypoints::EYE,
            z: p.z,
            yaw: f64::from(p.yaw),
            pitch: f64::from(p.pitch),
            fov: fov / view.factor.max(1.0),
            width: f64::from(batch.width),
            height: f64::from(batch.height),
        };
        for (k, &(i, x, y, z)) in here.iter().enumerate() {
            let Some(at) = cam.project(x, y, z, 10.0) else {
                continue;
            };
            let name = &names[k];
            let dist = ((x - cam.x).powi(2) + (y - cam.y).powi(2) + (z - cam.z).powi(2)).sqrt();
            let death = name == waypoints::DEATH;
            let colour = if death { RED } else { PALETTE[i % PALETTE.len()] };
            let (mx, my) = (at.x.round() as f32, at.y.round() as f32);
            let size = if at.edge { 3 } else { 4 };
            diamond(game, batch, mx, my, size + 1, OUTLINE);
            diamond(game, batch, mx, my, size, 0xFF00_0000 | colour);
            let label = format!("{} {}m", name, dist.round() as i64);
            let w = game.native_text_width(&label);
            let s = size as f32;
            let ty = if my < 20.0 { my + s + 3.0 } else { my - s - 11.0 };
            game.native_text(batch, &label, [mx - w / 2.0, ty], if death { RED } else { WHITE }, true);
        }
    }
    let lines = panel_lines(game, &cfg, &p, &here, &names);
    let c = corner(cfg.hud_corner);
    if lines.is_empty() || !game.native_corner_shown(c) {
        return;
    }
    let w = lines
        .iter()
        .map(|(l, _)| game.native_text_width(l))
        .fold(0.0, f32::max)
        + 4.0;
    let h = 10.0 * lines.len() as f32 + 2.0;
    let (x, y) = corners.place(c, w, h);
    game.native_fill(batch, [x, y, w, h], PANEL);
    for (n, (line, colour)) in lines.iter().enumerate() {
        game.native_text(batch, line, [x + 2.0, y + 2.0 + 10.0 * n as f32], *colour, true);
    }
}

/// A diamond `size` pixels from its centre to each tip, in one-pixel rows.
fn diamond(game: &Game<'_>, batch: &mut HudBatch, x: f32, y: f32, size: i32, argb: u32) {
    for dy in -size..=size {
        let half = (size - dy.abs()) as f32;
        game.native_fill(batch, [x - half, y + dy as f32, 2.0 * half + 1.0, 1.0], argb);
    }
}

fn panel_lines(
    game: &Game<'_>,
    cfg: &gui::Config,
    p: &super::native_hooks::NativePlayer,
    here: &[(usize, f64, f64, f64)],
    names: &[String],
) -> Vec<(String, u32)> {
    let mut lines = Vec::new();
    if cfg.hud {
        if cfg.hud_fps {
            let (fps, min, low) = game.native_frame_rate();
            let text = if cfg.hud_lows {
                format!("{fps} FPS | 1% {low} | min {min}")
            } else {
                format!("{fps} FPS")
            };
            lines.push((text, fps_colour(fps)));
        }
        let facing = waypoints::facing(f64::from(p.yaw));
        if cfg.hud_coords {
            let mut text = format!("XYZ {} {} {}", block(p.x), block(p.y), block(p.z));
            if cfg.hud_facing {
                text.push_str(&format!(" ({facing})"));
            }
            lines.push((text, WHITE));
        } else if cfg.hud_facing {
            lines.push((format!("Facing {facing}"), WHITE));
        }
        if cfg.hud_other_dim
            && let Some((name, x, z)) = waypoints::other_dimension(p.dimension, p.x, p.z)
        {
            let colour = if name == "Nether" { NETHER_RED } else { AQUA };
            lines.push((format!("{name} XZ {} {}", block(x), block(z)), colour));
        }
    }
    if cfg.wp_hud {
        let eye = (p.x, p.y + waypoints::EYE, p.z);
        let mut near: Vec<(f64, String, u32)> = here
            .iter()
            .zip(names)
            .map(|(&(_, x, y, z), name)| {
                let dist = ((x - eye.0).powi(2) + (y - eye.1).powi(2) + (z - eye.2).powi(2)).sqrt();
                let rel = waypoints::yaw_to(p.x, p.z, x, z) - f64::from(p.yaw);
                let colour = if name == waypoints::DEATH { RED } else { GOLD };
                let text = format!(
                    "{} {}m, {} o'clock",
                    name,
                    dist.round() as i64,
                    waypoints::clock(rel)
                );
                (dist, text, colour)
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        lines.extend(near.into_iter().take(HUD_WAYPOINTS).map(|(_, t, c)| (t, c)));
    }
    lines
}

/// `/aetherium` and `/wp`.
pub(super) fn command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let command = command.trim().trim_start_matches('/');
    let (name, args) = command.split_once(' ').unwrap_or((command, ""));
    if !matches!(name, "aetherium" | "wp") || !game.native_on(gui::ID) {
        return;
    }
    *ran = true;
    let mut rt = take(game);
    let reply = if name == "wp" {
        rt.wp_command(game, args)
    } else {
        match args.trim() {
            "" => {
                rt.open = Some(0);
                String::new()
            }
            "version" => format!("Aetherium {} (native) for Horizon 26.3", gui::VERSION),
            "reset" => {
                gui::with(|sh| {
                    let dark = sh.config.dark;
                    sh.config = gui::Config {
                        dark,
                        ..gui::Config::DEFAULT
                    };
                    sh.config_dirty = true;
                    sh.config_gen += 1;
                });
                "Aetherium settings reset to defaults".into()
            }
            _ => "/aetherium [version | reset] - no arguments opens the settings screen".into(),
        }
    };
    if std::mem::take(&mut rt.waypoints_changed) {
        game.native_set_setting(gui::ID, "waypoints", &rt.waypoints.to_text());
        rt.publish_waypoints();
    }
    put(rt);
    if !reply.is_empty() {
        game.native_chat(&reply);
    }
}

/// The commands chat completes.
pub(super) fn commands(commands: &mut Vec<&'static str>) {
    commands.extend(["aetherium", "wp"]);
}

/// Aetherium's row in Options > Mods, with its keys.
pub(super) fn listed(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: gui::ID.into(),
        title: format!("Aetherium {}", gui::VERSION),
        description: DESCRIPTION.into(),
        keys: gui::KEYS
            .iter()
            .map(|[n, l, d]| [(*n).to_owned(), (*l).to_owned(), (*d).to_owned()])
            .collect(),
    });
}
