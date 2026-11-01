//! Aetherium for Horizon (Minecraft Java 26.3 rewritten in Rust), built as a
//! sandboxed `.hmod` for Vasysik/horizon-android and desktop Horizon.
//!
//! What Horizon's mod interface lets a sandboxed mod do, Aetherium does here:
//! - Zoom as in the Java mod (Zume's curve and easing): a Zoom key (hold or
//!   toggle), Zoom In / Zoom Out keys, the mouse wheel while zooming, and a
//!   slower mouse while zoomed.
//! - An info HUD: FPS with the 1% low and minimum, coordinates, facing and
//!   the matching Nether/Overworld coordinates.
//! - Waypoints: saved with a key or `/wp add`, shown as markers on screen and
//!   in the HUD with distance and a clock direction, converted between the
//!   Nether and the Overworld, and an automatic death point.
//! - A settings page in Options > Mods and an `/aetherium` screen.
//!
//! Horizon draws the world itself; a sandboxed mod can't replace its renderer,
//! so the Java mod's renderer and performance features have no place here.

mod settings;
mod waypoints;
mod zoom;

use horizon_mod_sdk::{
    Mod, Registration, chat, game, hud,
    input::{self, Event},
    render::{self, Point, QuadShape, Shape, TextShape},
    storage,
    ui::{self, ButtonWidget, ListWidget, Screen, TextWidget, Widget},
};
use settings::{Corner, Opt, Settings};
use waypoints::{Camera, Waypoint, Waypoints};
use zoom::{View, Zoom, ZoomConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Keys, as `register-key` names them.
const KEY_ZOOM: &str = "zoom";
const KEY_ZOOM_IN: &str = "zoom_in";
const KEY_ZOOM_OUT: &str = "zoom_out";
const KEY_ADD_WAYPOINT: &str = "add_waypoint";
const KEY_WAYPOINTS: &str = "waypoints";

// Widget ids.
const W_LIST: u32 = 1;
const W_ADD: u32 = 10;
const W_SHOW: u32 = 11;
const W_DELETE: u32 = 12;
const W_DELETE_ALL: u32 = 13;
const W_BACK: u32 = 14;
const W_OPT_BASE: u32 = 100;
const W_OPEN_WAYPOINTS: u32 = 200;
const W_TEXT: u32 = 900;

// Colours.
const GOLD: u32 = 0xFFAA00;
const RED: u32 = 0xFF5555;
const GREEN: u32 = 0x55FF55;
const YELLOW: u32 = 0xFFFF55;
const WHITE: u32 = 0xFFFFFF;
const AQUA: u32 = 0x55FFFF;
const NETHER_RED: u32 = 0xFF7755;
/// Marker colours, cycled by a waypoint's place in the list.
const PALETTE: [u32; 6] = [0x55FFFF, 0xFFAA00, 0x55FF55, 0xFF55FF, 0xFFFF55, 0x5555FF];

/// Waypoint rows shown in the HUD at most.
const HUD_WAYPOINTS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Main,
    Waypoints,
}

struct Aetherium {
    settings: Settings,
    zoom: Zoom,
    held_zoom: bool,
    held_in: bool,
    held_out: bool,
    wheel: bool,
    view: View,
    waypoints: Waypoints,
    /// Health at the last tick, to see a death.
    last_health: Option<f32>,
    /// The dimension at the last tick.
    dimension: String,
    in_world: bool,
    /// The HUD panel last sent and its corner.
    panel: Vec<(String, u32)>,
    panel_corner: Option<Corner>,
    /// A summary of the marker shapes last sent ("" for none).
    markers: String,
    /// Some shown waypoint is in this dimension (or its Nether/Overworld twin).
    markers_here: bool,
    screen: Option<Page>,
    selected: Option<usize>,
    confirm_delete_all: bool,
}

fn stored(key: &str) -> Option<String> {
    storage::get(key).and_then(|v| String::from_utf8(v).ok())
}

fn zoom_config(s: &Settings) -> ZoomConfig {
    ZoomConfig {
        toggle: s.zoom_toggle,
        default_zoom: f64::from(s.zoom_default) / 100.0,
        smooth_s: f64::from(s.zoom_smooth_ms) / 1000.0,
        scrolling: s.zoom_scroll,
        mouse_floor: f64::from(s.zoom_mouse) / 100.0,
        fov: f64::from(s.fov),
    }
}

fn hud_corner(c: Corner) -> hud::Corner {
    match c {
        Corner::TopLeft => hud::Corner::TopLeft,
        Corner::TopRight => hud::Corner::TopRight,
        Corner::BottomLeft => hud::Corner::BottomLeft,
        Corner::BottomRight => hud::Corner::BottomRight,
    }
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

fn block(v: f64) -> i32 {
    v.floor() as i32
}

impl Aetherium {
    fn save_settings(&self) {
        let _ = storage::set("settings", self.settings.to_text().as_bytes());
    }

    fn save_waypoints(&self) {
        let _ = storage::set("waypoints", self.waypoints.to_text().as_bytes());
    }

    fn apply_settings(&mut self) {
        self.zoom.config = zoom_config(&self.settings);
        // Redraw everything with the new options.
        self.panel.clear();
        self.panel_corner = None;
        self.markers_here = self.compute_markers_here();
    }

    // ------------------------------------------------------------ zoom

    fn show_view(&mut self) {
        let view = self.zoom.view();
        let moved = (view.factor - self.view.factor).abs() > 1e-4
            || (view.turn - self.view.turn).abs() > 1e-4;
        if moved {
            render::set_zoom(render::Zoom {
                factor: view.factor as f32,
                smooth: false,
                turn: view.turn as f32,
            });
            self.view = view;
        }
        let wants = self.zoom.wants_wheel();
        if wants != self.wheel {
            input::claim_wheel(wants);
            self.wheel = wants;
        }
    }

    // ------------------------------------------------------------ waypoints

    fn add_waypoint_here(&mut self, name: Option<&str>) -> Result<String, String> {
        let p = game::get_player().ok_or("Not in a world")?;
        let name = name
            .map(waypoints::clean_name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| self.waypoints.next_name());
        let wp = Waypoint {
            name: name.clone(),
            dimension: p.dimension.clone(),
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
        let i = self.waypoints.set(wp)?;
        self.selected = Some(i);
        self.save_waypoints();
        self.markers_here = self.compute_markers_here();
        self.panel.clear();
        Ok(text)
    }

    /// Shown waypoints in the current dimension, converted to it, with their
    /// index: (index, x, y, z) at the block's centre.
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

    fn compute_markers_here(&self) -> bool {
        self.in_world && !self.here(&self.dimension).is_empty()
    }

    fn clear_markers(&mut self) {
        if !self.markers.is_empty() {
            render::set_hud(&[]);
            self.markers.clear();
        }
    }

    /// The on-screen markers, once a frame while there is something to show.
    fn draw_markers(&mut self) {
        if !self.settings.wp_markers || !self.markers_here {
            self.clear_markers();
            return;
        }
        let Some(p) = game::get_player() else {
            self.clear_markers();
            return;
        };
        let (w, h) = render::gui_size();
        let cam = Camera {
            x: p.x,
            y: p.y + waypoints::EYE,
            z: p.z,
            yaw: f64::from(p.yaw),
            pitch: f64::from(p.pitch),
            fov: f64::from(self.settings.fov) / self.view.factor.max(1.0),
            width: f64::from(w),
            height: f64::from(h),
        };
        let mut shapes = Vec::new();
        let mut summary = String::new();
        for (i, x, y, z) in self.here(&p.dimension) {
            let Some(at) = cam.project(x, y, z, 10.0) else {
                continue;
            };
            let wp = &self.waypoints.list[i];
            let dist = ((x - cam.x).powi(2) + (y - cam.y).powi(2) + (z - cam.z).powi(2)).sqrt();
            let death = wp.name == waypoints::DEATH;
            let colour = if death {
                RED
            } else {
                PALETTE[i % PALETTE.len()]
            };
            let label = format!("{} {}m", wp.name, dist.round() as i64);
            let (mx, my) = ((at.x * 2.0).round() / 2.0, (at.y * 2.0).round() / 2.0);
            summary.push_str(&format!("{mx},{my},{label};"));
            let s = if at.edge { 3.0 } else { 4.0 };
            let pt = |px: f64, py: f64| Point {
                x: px as f32,
                y: py as f32,
            };
            shapes.push(Shape::Quad(QuadShape {
                corners: (
                    pt(mx, my - s - 1.0),
                    pt(mx + s + 1.0, my),
                    pt(mx, my + s + 1.0),
                    pt(mx - s - 1.0, my),
                ),
                color: 0xC000_0000,
            }));
            shapes.push(Shape::Quad(QuadShape {
                corners: (pt(mx, my - s), pt(mx + s, my), pt(mx, my + s), pt(mx - s, my)),
                color: 0xFF00_0000 | colour,
            }));
            // The label above the marker, below it at the top edge.
            let ty = if my < 20.0 { my + s + 3.0 } else { my - s - 11.0 };
            shapes.push(Shape::Text(TextShape {
                text: label,
                at: pt(mx, ty),
                color: if death { RED } else { WHITE },
                centered: true,
                shadow: true,
            }));
        }
        if summary != self.markers {
            render::set_hud(&shapes);
            self.markers = summary;
        }
    }

    // ------------------------------------------------------------ HUD

    fn panel_lines(&self, p: &game::Player) -> Vec<(String, u32)> {
        let s = &self.settings;
        let mut lines = Vec::new();
        if s.hud {
            if s.hud_fps {
                let f = render::frame_stats();
                let text = if s.hud_lows {
                    format!("{} FPS | 1% {} | min {}", f.fps, f.low, f.min)
                } else {
                    format!("{} FPS", f.fps)
                };
                lines.push((text, fps_colour(f.fps)));
            }
            let facing = waypoints::facing(f64::from(p.yaw));
            if s.hud_coords {
                let mut text = format!("XYZ {} {} {}", block(p.x), block(p.y), block(p.z));
                if s.hud_facing {
                    text.push_str(&format!(" ({facing})"));
                }
                lines.push((text, WHITE));
            } else if s.hud_facing {
                lines.push((format!("Facing {facing}"), WHITE));
            }
            if s.hud_other_dim
                && let Some((name, x, z)) = waypoints::other_dimension(&p.dimension, p.x, p.z)
            {
                let colour = if name == "Nether" { NETHER_RED } else { AQUA };
                lines.push((format!("{name} XZ {} {}", block(x), block(z)), colour));
            }
        }
        if s.wp_hud {
            let eye = (p.x, p.y + waypoints::EYE, p.z);
            let mut near: Vec<(f64, String, u32)> = self
                .here(&p.dimension)
                .into_iter()
                .map(|(i, x, y, z)| {
                    let wp = &self.waypoints.list[i];
                    let dist = ((x - eye.0).powi(2) + (y - eye.1).powi(2) + (z - eye.2).powi(2))
                        .sqrt();
                    let rel = waypoints::yaw_to(p.x, p.z, x, z) - f64::from(p.yaw);
                    let colour = if wp.name == waypoints::DEATH { RED } else { GOLD };
                    let text = format!(
                        "{} {}m, {} o'clock",
                        wp.name,
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

    fn show_panel(&mut self, lines: Vec<(String, u32)>) {
        let corner = self.settings.hud_corner;
        if lines == self.panel && self.panel_corner == Some(corner) {
            return;
        }
        if lines.is_empty() {
            hud::clear();
        } else {
            let out: Vec<hud::Line> = lines
                .iter()
                .map(|(text, color)| hud::Line {
                    text: text.clone(),
                    color: *color,
                })
                .collect();
            hud::set_panel(hud_corner(corner), &out);
        }
        self.panel = lines;
        self.panel_corner = Some(corner);
    }

    fn leave_world(&mut self) {
        self.in_world = false;
        self.last_health = None;
        self.markers_here = false;
        self.held_zoom = false;
        self.held_in = false;
        self.held_out = false;
        self.zoom.reset();
        self.show_view();
        self.clear_markers();
        if !self.panel.is_empty() {
            hud::clear();
            self.panel.clear();
        }
        self.panel_corner = None;
    }

    // ------------------------------------------------------------ widgets

    fn option_widgets(&self) -> Vec<Widget> {
        let mut out = Vec::new();
        let mut section = "";
        for (i, opt) in Opt::ALL.into_iter().enumerate() {
            if opt.section() != section {
                section = opt.section();
                out.push(Widget::Text(TextWidget {
                    id: W_TEXT + i as u32,
                    text: format!("- {section} -"),
                    color: GOLD,
                }));
            }
            out.push(Widget::Button(ButtonWidget {
                id: W_OPT_BASE + i as u32,
                label: self.settings.label(opt),
            }));
        }
        out
    }

    fn main_widgets(&self) -> Vec<Widget> {
        let mut out = vec![Widget::Text(TextWidget {
            id: W_TEXT + 50,
            text: format!("Aetherium {VERSION} for Horizon 26.3. Keys: Options > Mods > Aetherium."),
            color: 0,
        })];
        out.extend(self.option_widgets());
        out.push(Widget::Button(ButtonWidget {
            id: W_OPEN_WAYPOINTS,
            label: format!("Waypoints ({})...", self.waypoints.list.len()),
        }));
        out
    }

    fn waypoint_widgets(&self) -> Vec<Widget> {
        let p = game::get_player();
        let rows: Vec<String> = self
            .waypoints
            .list
            .iter()
            .map(|w| {
                let mut row = format!(
                    "{}  {} {} {}  {}",
                    w.name,
                    w.x,
                    w.y,
                    w.z,
                    waypoints::dimension_name(&w.dimension)
                );
                if let Some(p) = &p
                    && let Some((x, z)) = waypoints::convert(
                        &w.dimension,
                        f64::from(w.x) + 0.5,
                        f64::from(w.z) + 0.5,
                        &p.dimension,
                    )
                {
                    let d = ((x - p.x).powi(2) + (z - p.z).powi(2)).sqrt();
                    row.push_str(&format!("  {}m", d.round() as i64));
                }
                if !w.shown {
                    row.push_str("  (hidden)");
                }
                row
            })
            .collect();
        let selected = self
            .selected
            .filter(|&i| i < self.waypoints.list.len())
            .map(|i| i as u32);
        let picked = selected.map(|i| &self.waypoints.list[i as usize]);
        let mut out = vec![Widget::Text(TextWidget {
            id: W_TEXT + 60,
            text: if rows.is_empty() {
                "No waypoints yet. Add one here, with its key, or /wp add <name>.".into()
            } else {
                "Pick a waypoint, then a button. /wp add <name> saves a named one.".into()
            },
            color: 0,
        })];
        if !rows.is_empty() {
            out.push(Widget::List(ListWidget {
                id: W_LIST,
                rows,
                selected,
            }));
        }
        out.push(Widget::Button(ButtonWidget {
            id: W_ADD,
            label: "Add a waypoint where I stand".into(),
        }));
        if let Some(w) = picked {
            out.push(Widget::Button(ButtonWidget {
                id: W_SHOW,
                label: format!(
                    "{}: {}",
                    w.name,
                    if w.shown { "Shown (click to hide)" } else { "Hidden (click to show)" }
                ),
            }));
            out.push(Widget::Button(ButtonWidget {
                id: W_DELETE,
                label: format!("Delete {}", w.name),
            }));
        }
        if !self.waypoints.list.is_empty() {
            out.push(Widget::Button(ButtonWidget {
                id: W_DELETE_ALL,
                label: if self.confirm_delete_all {
                    "Click again to delete ALL waypoints".into()
                } else {
                    "Delete all waypoints".into()
                },
            }));
        }
        out.push(Widget::Button(ButtonWidget {
            id: W_BACK,
            label: "Aetherium settings...".into(),
        }));
        out
    }

    fn open(&mut self, page: Page) {
        let (title, widgets) = match page {
            Page::Main => ("Aetherium", self.main_widgets()),
            Page::Waypoints => ("Aetherium waypoints", self.waypoint_widgets()),
        };
        ui::open_screen(&Screen {
            title: title.into(),
            widgets,
        });
        self.screen = Some(page);
    }

    fn refresh(&mut self) {
        ui::set_settings(&self.option_widgets());
        if let Some(page) = self.screen {
            self.open(page);
        }
    }

    // ------------------------------------------------------------ commands

    fn aetherium_command(&mut self, args: &str) -> Result<Option<String>, String> {
        let mut words = args.split_whitespace();
        match words.next() {
            None => {
                self.open(Page::Main);
                Ok(None)
            }
            Some("set") => {
                let key = words.next().ok_or("/aetherium set <option> <value>")?;
                let value = words.collect::<Vec<_>>().join(" ");
                let opt = Opt::from_key(key).ok_or_else(|| unknown_option(key))?;
                self.settings.set(opt, &value)?;
                self.save_settings();
                self.apply_settings();
                self.refresh();
                Ok(Some(format!("{} = {}", opt.key(), self.settings.value(opt))))
            }
            Some("get") => {
                let key = words.next().ok_or("/aetherium get <option>")?;
                let opt = Opt::from_key(key).ok_or_else(|| unknown_option(key))?;
                Ok(Some(format!("{} = {}", opt.key(), self.settings.value(opt))))
            }
            Some("reset") => {
                self.settings = Settings::default();
                self.save_settings();
                self.apply_settings();
                self.refresh();
                Ok(Some("Aetherium settings reset to defaults".into()))
            }
            Some("version") => Ok(Some(format!("Aetherium {VERSION} for Horizon 26.3"))),
            Some(_) => Err("/aetherium [set <option> <value> | get <option> | reset | version]".into()),
        }
    }

    fn wp_command(&mut self, args: &str) -> Result<Option<String>, String> {
        let (verb, rest) = match args.trim().split_once(' ') {
            Some((v, r)) => (v, r.trim()),
            None => (args.trim(), ""),
        };
        let pick = |w: &Waypoints, rest: &str| {
            if rest.is_empty() {
                return Err("Name a waypoint, or give its number from /wp list".to_string());
            }
            w.find(rest)
                .ok_or_else(|| format!("No waypoint called {rest}"))
        };
        match verb {
            "" => {
                self.open(Page::Waypoints);
                Ok(None)
            }
            "add" | "set" => {
                let text = self.add_waypoint_here((!rest.is_empty()).then_some(rest))?;
                self.refresh_waypoint_screen();
                Ok(Some(text))
            }
            "del" | "delete" | "remove" => {
                let i = pick(&self.waypoints, rest)?;
                let w = self.waypoints.remove(i).ok_or("No such waypoint")?;
                self.after_waypoint_change();
                Ok(Some(format!("Waypoint {} deleted", w.name)))
            }
            "show" | "hide" => {
                let i = pick(&self.waypoints, rest)?;
                let shown = verb == "show";
                self.waypoints.list[i].shown = shown;
                let name = self.waypoints.list[i].name.clone();
                self.after_waypoint_change();
                Ok(Some(format!(
                    "Waypoint {name} {}",
                    if shown { "shown" } else { "hidden" }
                )))
            }
            "list" => {
                if self.waypoints.list.is_empty() {
                    return Ok(Some("No waypoints yet: /wp add <name>".into()));
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
                Ok(Some(all.join("; ")))
            }
            "clear" => {
                let n = self.waypoints.list.len();
                self.waypoints.list.clear();
                self.after_waypoint_change();
                Ok(Some(format!("Deleted {n} waypoints")))
            }
            _ => Err("/wp [add <name> | del <name> | show <name> | hide <name> | list | clear]".into()),
        }
    }

    fn after_waypoint_change(&mut self) {
        self.selected = self.selected.filter(|&i| i < self.waypoints.list.len());
        self.confirm_delete_all = false;
        self.save_waypoints();
        self.markers_here = self.compute_markers_here();
        self.panel.clear();
        self.refresh_waypoint_screen();
    }

    fn refresh_waypoint_screen(&mut self) {
        if self.screen == Some(Page::Waypoints) {
            self.open(Page::Waypoints);
        }
    }
}

fn unknown_option(key: &str) -> String {
    let keys: Vec<&str> = Opt::ALL.iter().map(|o| o.key()).collect();
    format!("Unknown option {key}. Options: {}", keys.join(", "))
}

impl Mod for Aetherium {
    fn init() -> Result<(Self, Registration), String> {
        input::register_key(KEY_ZOOM, "Zoom", "key.keyboard.z")?;
        input::register_key(KEY_ZOOM_IN, "Zoom in", "key.keyboard.equal")?;
        input::register_key(KEY_ZOOM_OUT, "Zoom out", "key.keyboard.minus")?;
        input::register_key(KEY_ADD_WAYPOINT, "Add waypoint", "key.keyboard.b")?;
        input::register_key(KEY_WAYPOINTS, "Waypoints screen", "key.keyboard.n")?;
        let settings = stored("settings")
            .map(|t| Settings::from_text(&t))
            .unwrap_or_default();
        let waypoints = stored("waypoints")
            .map(|t| Waypoints::from_text(&t))
            .unwrap_or_default();
        let state = Aetherium {
            zoom: Zoom::new(zoom_config(&settings)),
            settings,
            held_zoom: false,
            held_in: false,
            held_out: false,
            wheel: false,
            view: View::NONE,
            waypoints,
            last_health: None,
            dimension: String::new(),
            in_world: false,
            panel: Vec::new(),
            panel_corner: None,
            markers: String::new(),
            markers_here: false,
            screen: None,
            selected: None,
            confirm_delete_all: false,
        };
        ui::set_settings(&state.option_widgets());
        horizon_mod_sdk::info(&format!("Aetherium {VERSION} for Horizon loaded"));
        let hooks = Registration::new()
            .command("aetherium")
            .command("wp")
            .on_tick()
            .on_frame();
        Ok((state, hooks))
    }

    fn tick(&mut self, _tick: u64) {
        let Some(p) = game::get_player() else {
            if self.in_world {
                self.leave_world();
            }
            return;
        };
        if !self.in_world || p.dimension != self.dimension {
            self.in_world = true;
            self.dimension.clone_from(&p.dimension);
            self.markers_here = self.compute_markers_here();
        }
        // A death: health reached 0 this tick.
        let dead = p.health <= 0.0;
        if dead && self.last_health.is_some_and(|h| h > 0.0) && self.settings.wp_death {
            let wp = Waypoint {
                name: waypoints::DEATH.into(),
                dimension: p.dimension.clone(),
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
            if self.waypoints.set(wp).is_ok() {
                self.save_waypoints();
                self.markers_here = self.compute_markers_here();
                chat::show(&text);
                self.refresh_waypoint_screen();
            }
        }
        self.last_health = Some(p.health);
        let lines = self.panel_lines(&p);
        self.show_panel(lines);
    }

    fn frame(&mut self, seconds: f32) {
        for event in input::poll() {
            match event {
                Event::KeyDown(k) => match k.as_str() {
                    KEY_ZOOM => self.held_zoom = true,
                    KEY_ZOOM_IN => self.held_in = true,
                    KEY_ZOOM_OUT => self.held_out = true,
                    KEY_ADD_WAYPOINT => {
                        match self.add_waypoint_here(None) {
                            Ok(text) => chat::show(&text),
                            Err(e) => chat::show(&e),
                        }
                        self.refresh_waypoint_screen();
                    }
                    KEY_WAYPOINTS => self.open(Page::Waypoints),
                    _ => {}
                },
                Event::KeyUp(k) => match k.as_str() {
                    KEY_ZOOM => self.held_zoom = false,
                    KEY_ZOOM_IN => self.held_in = false,
                    KEY_ZOOM_OUT => self.held_out = false,
                    _ => {}
                },
                Event::Wheel(steps) => {
                    self.zoom.scroll(f64::from(steps));
                }
                _ => {}
            }
        }
        self.zoom.frame(
            f64::from(seconds),
            self.held_zoom,
            self.held_in,
            self.held_out,
        );
        self.show_view();
        if self.in_world {
            self.draw_markers();
        }
    }

    fn command(&mut self, name: &str, args: &str) -> Result<Option<String>, String> {
        match name {
            "aetherium" => self.aetherium_command(args),
            "wp" => self.wp_command(args),
            _ => Err("Unknown command".into()),
        }
    }

    fn ui_select(&mut self, id: u32, row: u32) {
        match id {
            W_LIST => {
                self.selected = Some(row as usize);
                self.confirm_delete_all = false;
            }
            W_ADD => {
                if let Err(e) = self.add_waypoint_here(None) {
                    chat::show(&e);
                }
                self.confirm_delete_all = false;
            }
            W_SHOW => {
                if let Some(w) = self.selected.and_then(|i| self.waypoints.list.get_mut(i)) {
                    w.shown = !w.shown;
                }
                self.after_waypoint_change();
            }
            W_DELETE => {
                if let Some(i) = self.selected {
                    self.waypoints.remove(i);
                }
                self.selected = None;
                self.after_waypoint_change();
            }
            W_DELETE_ALL => {
                if self.confirm_delete_all {
                    self.waypoints.list.clear();
                    self.selected = None;
                    self.after_waypoint_change();
                } else {
                    self.confirm_delete_all = true;
                }
            }
            W_BACK => {
                self.confirm_delete_all = false;
                self.open(Page::Main);
                return;
            }
            W_OPEN_WAYPOINTS => {
                self.open(Page::Waypoints);
                return;
            }
            id if (W_OPT_BASE..W_OPT_BASE + Opt::ALL.len() as u32).contains(&id) => {
                let opt = Opt::ALL[(id - W_OPT_BASE) as usize];
                self.settings.cycle(opt);
                self.save_settings();
                self.apply_settings();
            }
            _ => return,
        }
        self.refresh();
    }

    fn ui_closed(&mut self) {
        self.screen = None;
        self.confirm_delete_all = false;
    }

    fn ui_settings_opened(&mut self) {
        ui::set_settings(&self.option_widgets());
    }
}

horizon_mod_sdk::export_mod!(Aetherium);
