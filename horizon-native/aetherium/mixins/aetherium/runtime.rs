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

// ---- zoom.rs of the .hmod (horizon/aetherium/src/zoom.rs), copied by tools/assemble.py.
#[allow(dead_code)]
mod zoom {
    // Aetherium's zoom, ported from the Java mod's `ZoomController` (which
    // follows Zume): a zoom *amount* from 1 (none) to 0 (strongest), eased with
    // `1 - (1 - t)^4`, turned into a field of view as
    // `minFov + (fov - minFov) * amount^2`.
    //
    // Horizon takes a zoom as a factor the field of view is divided by (at most
    // 10) and a mouse-speed multiplier, so the amount becomes
    // `fov / zoomed_fov`. Third-person zoom and the cinematic camera are not
    // offered by Horizon's mod interface, so this port is first-person only and
    // slows the mouse instead.

    /// Easing exponent of the zoom animation (Zume's default).
    pub const ANIMATION_EXPONENT: f64 = 4.0;
    /// Exponent of the amount-to-FOV curve (Zume's default).
    pub const ZOOM_EXPONENT: f64 = 2.0;
    /// The narrowest field of view the curve reaches (Zume's default).
    pub const MIN_FOV: f64 = 1.0;
    /// Zoom speed (Zume's default "speed" of 20).
    const SPEED: f64 = 20.0;
    /// Amount per mouse-wheel step, per unit of speed.
    const SCROLL_STEP_PER_SPEED: f64 = 0.004;
    /// Amount per millisecond a Zoom In/Out key is held, per unit of speed.
    const KEY_RATE_PER_SPEED_MS: f64 = 0.0001;
    /// A longer frame (a hitch, a pause) counts as this long.
    const MAX_FRAME_S: f64 = 0.25;
    /// Horizon's limits on a zoom.
    pub const MAX_FACTOR: f64 = 10.0;
    pub const MIN_TURN: f64 = 0.05;

    /// The zoom options this controller reads.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ZoomConfig {
        pub toggle: bool,
        /// 0..1: how far a fresh zoom goes.
        pub default_zoom: f64,
        /// Seconds an ease takes (0: instant).
        pub smooth_s: f64,
        pub scrolling: bool,
        /// 0..1: mouse speed at the strongest zoom.
        pub mouse_floor: f64,
        /// The player's field of view, degrees.
        pub fov: f64,
    }

    impl Default for ZoomConfig {
        fn default() -> Self {
            ZoomConfig {
                toggle: false,
                default_zoom: 0.5,
                smooth_s: 0.15,
                scrolling: true,
                mouse_floor: 0.4,
                fov: 70.0,
            }
        }
    }

    /// What Horizon is told: the factor the FOV is divided by and the mouse speed.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct View {
        pub factor: f64,
        pub turn: f64,
    }

    impl View {
        pub const NONE: View = View {
            factor: 1.0,
            turn: 1.0,
        };
    }

    #[derive(Clone, Debug)]
    pub struct Zoom {
        pub config: ZoomConfig,
        now: f64,
        from: f64,
        target: f64,
        start: f64,
        end: f64,
        zooming: bool,
        was_zooming: bool,
        was_held: bool,
        scroll: f64,
    }

    fn clamp01(v: f64) -> f64 {
        v.clamp(0.0, 1.0)
    }

    impl Zoom {
        pub fn new(config: ZoomConfig) -> Zoom {
            Zoom {
                config,
                now: 0.0,
                from: 1.0,
                target: 1.0,
                start: 0.0,
                end: 0.0,
                zooming: false,
                was_zooming: false,
                was_held: false,
                scroll: 0.0,
            }
        }

        /// Forgets everything (leaving a world).
        pub fn reset(&mut self) {
            let config = self.config;
            let now = self.now;
            *self = Zoom::new(config);
            self.now = now;
        }

        /// Mouse-wheel steps (positive: away from the player, zooming in). True
        /// when zoom used them.
        pub fn scroll(&mut self, steps: f64) -> bool {
            if !self.config.scrolling || !self.zooming || steps == 0.0 {
                return false;
            }
            self.scroll += steps;
            true
        }

        /// Wants the mouse wheel now (zooming with wheel zoom on).
        pub fn wants_wheel(&self) -> bool {
            self.zooming && self.config.scrolling
        }

        pub fn is_zooming(&self) -> bool {
            self.zooming
        }

        /// Zooming, or still easing back out.
        pub fn is_visible(&self) -> bool {
            self.zooming || self.now < self.end
        }

        /// One rendered frame of `dt` seconds with the keys' state.
        pub fn frame(&mut self, dt: f64, held: bool, zoom_in: bool, zoom_out: bool) {
            let elapsed = if dt.is_finite() {
                dt.clamp(0.0, MAX_FRAME_S)
            } else {
                0.0
            };
            self.now += elapsed;
            let now = self.now;
            if self.config.toggle {
                if held && !self.was_held {
                    self.zooming = !self.zooming;
                }
            } else {
                self.zooming = held;
            }
            if self.zooming {
                if !self.was_zooming {
                    let to = 1.0 - clamp01(self.config.default_zoom);
                    self.animate(now, 1.0, to);
                }
                if self.config.scrolling && self.scroll != 0.0 {
                    let to = self.target - self.scroll * SPEED * SCROLL_STEP_PER_SPEED;
                    self.animate_to(now, to);
                } else if zoom_in != zoom_out && elapsed > 0.0 {
                    let step = SPEED * KEY_RATE_PER_SPEED_MS * elapsed * 1000.0;
                    let to = self.target + if zoom_in { -step } else { step };
                    self.animate_to(now, to);
                }
            } else if self.was_zooming {
                self.animate_to(now, 1.0);
            }
            self.scroll = 0.0;
            self.was_held = held;
            self.was_zooming = self.zooming;
        }

        /// The current, eased amount: 1 none, 0 strongest.
        pub fn amount(&self) -> f64 {
            if self.now < self.end {
                let span = self.end - self.start;
                let t = if span <= 0.0 {
                    1.0
                } else {
                    clamp01((self.now - self.start) / span)
                };
                let eased = 1.0 - (1.0 - t).powf(ANIMATION_EXPONENT);
                return self.from + (self.target - self.from) * eased;
            }
            self.target
        }

        /// The field of view Zume's curve gives for the current amount.
        pub fn zoomed_fov(&self) -> f64 {
            let fov = self.config.fov.max(MIN_FOV + 1.0);
            MIN_FOV + (fov - MIN_FOV) * self.amount().powf(ZOOM_EXPONENT)
        }

        /// What Horizon should show now.
        pub fn view(&self) -> View {
            if !self.is_visible() {
                return View::NONE;
            }
            let fov = self.config.fov.max(MIN_FOV + 1.0);
            let factor = (fov / self.zoomed_fov().max(MIN_FOV)).clamp(1.0, MAX_FACTOR);
            let turn = if self.zooming {
                let floor = clamp01(self.config.mouse_floor);
                (floor + (1.0 - floor) * self.amount()).clamp(MIN_TURN, 1.0)
            } else {
                1.0
            };
            View { factor, turn }
        }

        fn animate_to(&mut self, now: f64, to: f64) {
            let from = self.amount();
            self.animate(now, from, to);
        }

        fn animate(&mut self, now: f64, start: f64, to: f64) {
            let to = clamp01(to);
            if self.config.smooth_s <= 0.0 {
                self.from = to;
                self.target = to;
                self.start = 0.0;
                self.end = 0.0;
                return;
            }
            self.from = clamp01(start);
            self.target = to;
            self.start = now;
            self.end = now + self.config.smooth_s;
        }
    }
}

// ---- waypoints.rs of the .hmod (horizon/aetherium/src/waypoints.rs), copied by tools/assemble.py.
#[allow(dead_code)]
mod waypoints {
    // Waypoints: named block positions per dimension, kept in the mod's storage
    // one per line, with the maths to show them: Nether/Overworld conversion,
    // distance, a clock direction and where a point falls on the screen.

    pub const OVERWORLD: &str = "minecraft:overworld";
    pub const NETHER: &str = "minecraft:the_nether";
    pub const END: &str = "minecraft:the_end";
    /// The name of the automatic death point (one at a time).
    pub const DEATH: &str = "Death";
    pub const MAX_WAYPOINTS: usize = 64;
    pub const MAX_NAME: usize = 32;
    /// Eye height of a standing player, blocks.
    pub const EYE: f64 = 1.62;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Waypoint {
        pub name: String,
        pub dimension: String,
        pub x: i32,
        pub y: i32,
        pub z: i32,
        /// Shown on screen and in the HUD.
        pub shown: bool,
    }

    /// A short, readable name for a dimension id.
    pub fn dimension_name(id: &str) -> String {
        match id {
            OVERWORLD => "Overworld".into(),
            NETHER => "Nether".into(),
            END => "End".into(),
            other => other.rsplit(':').next().unwrap_or(other).replace('_', " "),
        }
    }

    /// Cleans a name for storage: one line, no tabs, at most `MAX_NAME` chars.
    pub fn clean_name(name: &str) -> String {
        let s: String = name
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        s.chars().take(MAX_NAME).collect()
    }

    /// Where a waypoint of `wp_dim` at (x, z) lies in `here`: the same place, or
    /// the matching Nether/Overworld place (8 to 1), or nowhere.
    pub fn convert(wp_dim: &str, x: f64, z: f64, here: &str) -> Option<(f64, f64)> {
        if wp_dim == here {
            return Some((x, z));
        }
        match (wp_dim, here) {
            (OVERWORLD, NETHER) => Some((x / 8.0, z / 8.0)),
            (NETHER, OVERWORLD) => Some((x * 8.0, z * 8.0)),
            _ => None,
        }
    }

    /// Block coordinates of the other of Nether and Overworld (the HUD line).
    pub fn other_dimension(dim: &str, x: f64, z: f64) -> Option<(&'static str, f64, f64)> {
        match dim {
            OVERWORLD => Some(("Nether", x / 8.0, z / 8.0)),
            NETHER => Some(("Overworld", x * 8.0, z * 8.0)),
            _ => None,
        }
    }

    /// Minecraft's yaw (degrees, 0 south, 90 west) of the direction from
    /// (px, pz) to (tx, tz).
    pub fn yaw_to(px: f64, pz: f64, tx: f64, tz: f64) -> f64 {
        (-(tx - px)).atan2(tz - pz).to_degrees()
    }

    /// An angle wrapped into (-180, 180].
    pub fn wrap(deg: f64) -> f64 {
        let mut d = deg % 360.0;
        if d > 180.0 {
            d -= 360.0;
        } else if d <= -180.0 {
            d += 360.0;
        }
        d
    }

    /// The direction of a target as a clock face: 12 is straight ahead, 3 to
    /// the right, 6 behind.
    pub fn clock(relative_deg: f64) -> u32 {
        let r = wrap(relative_deg);
        let hour = ((r / 30.0).round() as i32).rem_euclid(12);
        if hour == 0 { 12 } else { hour as u32 }
    }

    /// Facing as the game's F3 names it, from yaw.
    pub fn facing(yaw: f64) -> &'static str {
        let i = ((yaw / 90.0 + 0.5).floor() as i64).rem_euclid(4);
        ["South", "West", "North", "East"][i as usize]
    }

    /// The camera, to place world points on the screen.
    #[derive(Clone, Copy, Debug)]
    pub struct Camera {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f64,
        pub pitch: f64,
        /// Vertical field of view in degrees, zoom included.
        pub fov: f64,
        /// The screen in GUI pixels.
        pub width: f64,
        pub height: f64,
    }

    /// Where a world point is drawn.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct OnScreen {
        pub x: f64,
        pub y: f64,
        /// The point is outside the view and was pushed to the screen's edge.
        pub edge: bool,
    }

    impl Camera {
        /// The point (tx, ty, tz) on the screen; one outside the view (or behind)
        /// is pushed to the edge, in its direction, `margin` pixels in.
        pub fn project(&self, tx: f64, ty: f64, tz: f64, margin: f64) -> Option<OnScreen> {
            if self.width <= 2.0 * margin || self.height <= 2.0 * margin {
                return None;
            }
            let (sy, cy) = self.yaw.to_radians().sin_cos();
            let (sp, cp) = self.pitch.to_radians().sin_cos();
            // Forward, right and up of Minecraft's camera.
            let f = [-sy * cp, -sp, cy * cp];
            let r = [-cy, 0.0, -sy];
            let u = [
                r[1] * f[2] - r[2] * f[1],
                r[2] * f[0] - r[0] * f[2],
                r[0] * f[1] - r[1] * f[0],
            ];
            let d = [tx - self.x, ty - self.y, tz - self.z];
            let dot = |a: [f64; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
            let (zc, xc, yc) = (dot(f), dot(r), dot(u));
            let half = (self.width / 2.0, self.height / 2.0);
            let tan = (self.fov.clamp(1.0, 179.0).to_radians() / 2.0).tan();
            if zc > 0.05 {
                // Vertical FOV: one unit of tan covers half the screen's height.
                let x = half.0 + xc / zc / tan * half.1;
                let y = half.1 - yc / zc / tan * half.1;
                let inside = x >= margin
                    && x <= self.width - margin
                    && y >= margin
                    && y <= self.height - margin;
                if inside {
                    return Some(OnScreen { x, y, edge: false });
                }
                return Some(push_to_edge(x - half.0, y - half.1, half, margin));
            }
            // Behind: point the way to turn; straight behind goes to the bottom.
            let (dx, dy) = if xc.abs() < 1e-9 && yc.abs() < 1e-9 {
                (0.0, 1.0)
            } else {
                (xc, -yc.min(0.0) + 1e-3)
            };
            Some(push_to_edge(dx, dy.abs(), half, margin))
        }
    }

    /// From the screen's centre along (dx, dy) to the edge, `margin` inside.
    fn push_to_edge(dx: f64, dy: f64, half: (f64, f64), margin: f64) -> OnScreen {
        let (hw, hh) = (half.0 - margin, half.1 - margin);
        let sx = if dx.abs() < 1e-12 {
            f64::INFINITY
        } else {
            hw / dx.abs()
        };
        let sy = if dy.abs() < 1e-12 {
            f64::INFINITY
        } else {
            hh / dy.abs()
        };
        let s = sx.min(sy);
        let s = if s.is_finite() { s } else { 0.0 };
        OnScreen {
            x: half.0 + dx * s,
            y: half.1 + dy * s,
            edge: true,
        }
    }

    /// The saved list.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Waypoints {
        pub list: Vec<Waypoint>,
    }

    impl Waypoints {
        /// Reads the storage file; broken lines are skipped.
        pub fn from_text(text: &str) -> Waypoints {
            let mut list = Vec::new();
            for line in text.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() != 6 {
                    continue;
                }
                let (Ok(x), Ok(y), Ok(z)) = (
                    parts[2].parse::<i32>(),
                    parts[3].parse::<i32>(),
                    parts[4].parse::<i32>(),
                ) else {
                    continue;
                };
                let name = clean_name(parts[0]);
                if name.is_empty() || parts[1].is_empty() || list.len() >= MAX_WAYPOINTS {
                    continue;
                }
                list.push(Waypoint {
                    name,
                    dimension: parts[1].to_string(),
                    x,
                    y,
                    z,
                    shown: parts[5] != "0",
                });
            }
            Waypoints { list }
        }

        pub fn to_text(&self) -> String {
            let mut out = String::new();
            for w in &self.list {
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\n",
                    clean_name(&w.name),
                    w.dimension,
                    w.x,
                    w.y,
                    w.z,
                    if w.shown { 1 } else { 0 }
                ));
            }
            out
        }

        /// A name not taken yet: `Waypoint 3`.
        pub fn next_name(&self) -> String {
            (1..)
                .map(|i| format!("Waypoint {i}"))
                .find(|n| self.find(n).is_none())
                .unwrap_or_else(|| "Waypoint".into())
        }

        /// The index of the waypoint called `name` (any case), or numbered `#n`/`n`
        /// as `/wp list` shows them.
        pub fn find(&self, name: &str) -> Option<usize> {
            let name = name.trim();
            if let Some(i) = self
                .list
                .iter()
                .position(|w| w.name.eq_ignore_ascii_case(name))
            {
                return Some(i);
            }
            let n = name.strip_prefix('#').unwrap_or(name).parse::<usize>().ok()?;
            (1..=self.list.len()).contains(&n).then(|| n - 1)
        }

        /// Adds or moves the waypoint called `name`; errs when the list is full.
        pub fn set(&mut self, w: Waypoint) -> Result<usize, String> {
            if let Some(i) = self.find(&w.name) {
                self.list[i] = w;
                return Ok(i);
            }
            if self.list.len() >= MAX_WAYPOINTS {
                return Err(format!("You have {MAX_WAYPOINTS} waypoints already: delete one first"));
            }
            self.list.push(w);
            Ok(self.list.len() - 1)
        }

        pub fn remove(&mut self, i: usize) -> Option<Waypoint> {
            (i < self.list.len()).then(|| self.list.remove(i))
        }
    }
}
