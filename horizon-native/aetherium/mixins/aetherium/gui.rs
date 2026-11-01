// Aetherium's own settings screen inside Horizon's Options: the themed screen of the Java mod
// (sidebar tabs, animated rows, hover popups, Apply and Done) drawn with Horizon's HUD batch.
// This module is added to `horizon-client-render/src/options_ui.rs` as `options_ui::aetherium`;
// the hooks in this bundle hand it Options' drawing and input while Aetherium's page is open.
// Horizon's settings are read and written directly; Aetherium's own settings and the waypoint
// list go through `SHARED`, which the game side (`aetherium_rt` in the game crate) saves.

use std::cell::{Cell, RefCell};
use std::ops::ControlFlow;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use super::{Act, Event, OptionsUi, Page, Row};
use crate::hud::{HudAtlas, HudBatch};
use crate::settings::Settings;
use crate::text::Text;

/// The mod's id (Options > Mods, its settings folder) and version.
pub const ID: &str = "aetherium";
pub const VERSION: &str = "1.0.0";

/// The mod's keys: name, label, default key. Options > Mods and this screen list them; the game
/// side reads the player's choice with `native_mod_key`.
pub const KEYS: [[&str; 3]; 5] = [
    ["zoom", "Zoom", "key.keyboard.c"],
    ["zoom_in", "Zoom in", "key.keyboard.equal"],
    ["zoom_out", "Zoom out", "key.keyboard.minus"],
    ["add_waypoint", "Add waypoint", "key.keyboard.b"],
    ["menu", "Open Aetherium", "key.keyboard.n"],
];
const UNBOUND: &str = "key.keyboard.unknown";

// ---------------------------------------------------------------------------------------------
// Shared state with the game side
// ---------------------------------------------------------------------------------------------

/// Aetherium's own settings, kept in `config` in the mod's data folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub hud: bool,
    pub hud_fps: bool,
    pub hud_lows: bool,
    pub hud_coords: bool,
    pub hud_facing: bool,
    pub hud_other_dim: bool,
    /// 0 top left, 1 top right, 2 bottom left, 3 bottom right.
    pub hud_corner: i32,
    pub zoom_toggle: bool,
    /// Percent of the way in a press zooms, 10..=90.
    pub zoom_default: i32,
    /// Milliseconds of the zoom's easing, 0..=500.
    pub zoom_smooth_ms: i32,
    pub zoom_scroll: bool,
    /// Mouse speed at full zoom, percent, 10..=100.
    pub zoom_mouse: i32,
    pub wp_markers: bool,
    pub wp_hud: bool,
    pub wp_death: bool,
    pub dark: bool,
}

impl Config {
    pub const DEFAULT: Config = Config {
        hud: true,
        hud_fps: true,
        hud_lows: false,
        hud_coords: true,
        hud_facing: true,
        hud_other_dim: false,
        hud_corner: 0,
        zoom_toggle: false,
        zoom_default: 50,
        zoom_smooth_ms: 150,
        zoom_scroll: true,
        zoom_mouse: 40,
        wp_markers: true,
        wp_hud: true,
        wp_death: true,
        dark: false,
    };

    /// `key=value` lines; unknown keys and bad values keep the defaults.
    pub fn from_text(text: &str) -> Config {
        let mut c = Config::DEFAULT;
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let Some(k) = Cfg::ALL.iter().find(|k| k.key() == key.trim()) else {
                continue;
            };
            let value = value.trim();
            let v = match value {
                "true" => 1,
                "false" => 0,
                _ => match value.parse::<i32>() {
                    Ok(v) => v,
                    Err(_) => continue,
                },
            };
            k.set(&mut c, v);
        }
        c
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for k in Cfg::ALL {
            let v = k.get(self);
            if k.is_bool() {
                out.push_str(&format!("{}={}\n", k.key(), v != 0));
            } else {
                out.push_str(&format!("{}={v}\n", k.key()));
            }
        }
        out
    }
}

/// A waypoint as the screen lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct WaypointRow {
    pub name: String,
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub shown: bool,
}

/// What the screen asks the game side to do with the waypoints.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddHere,
    SetShown(String, bool),
    DeleteAll,
}

pub struct Shared {
    pub config: Config,
    /// The game side read `config` from its file.
    pub loaded: bool,
    /// The screen changed `config`: the game side saves it.
    pub config_dirty: bool,
    /// Counts changes of `config`, so the game side re-reads it.
    pub config_gen: u64,
    /// Frames in the last second, the slowest frame, the 1% low.
    pub fps: (u32, u32, u32),
    pub in_world: bool,
    pub waypoints: Vec<WaypointRow>,
    pub waypoints_gen: u64,
    pub actions: Vec<Action>,
    /// The tab the screen opens on next (`/wp` opens Waypoints).
    pub open_tab: Option<usize>,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    config: Config::DEFAULT,
    loaded: false,
    config_dirty: false,
    config_gen: 0,
    fps: (0, 0, 0),
    in_world: false,
    waypoints: Vec::new(),
    waypoints_gen: 0,
    actions: Vec::new(),
    open_tab: None,
});

fn shared() -> MutexGuard<'static, Shared> {
    SHARED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Runs `f` on the state shared with the game side.
pub fn with<R>(f: impl FnOnce(&mut Shared) -> R) -> R {
    f(&mut shared())
}

/// The tab index of Waypoints (`/wp`).
pub const TAB_WAYPOINTS: usize = 4;

// ---------------------------------------------------------------------------------------------
// Settings rows
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cfg {
    Hud,
    HudFps,
    HudLows,
    HudCoords,
    HudFacing,
    HudOtherDim,
    HudCorner,
    ZoomToggle,
    ZoomDefault,
    ZoomSmooth,
    ZoomScroll,
    ZoomMouse,
    WpMarkers,
    WpHud,
    WpDeath,
    Dark,
}

impl Cfg {
    const ALL: [Cfg; 16] = [
        Cfg::Hud,
        Cfg::HudFps,
        Cfg::HudLows,
        Cfg::HudCoords,
        Cfg::HudFacing,
        Cfg::HudOtherDim,
        Cfg::HudCorner,
        Cfg::ZoomToggle,
        Cfg::ZoomDefault,
        Cfg::ZoomSmooth,
        Cfg::ZoomScroll,
        Cfg::ZoomMouse,
        Cfg::WpMarkers,
        Cfg::WpHud,
        Cfg::WpDeath,
        Cfg::Dark,
    ];

    fn key(self) -> &'static str {
        match self {
            Cfg::Hud => "hud",
            Cfg::HudFps => "hud_fps",
            Cfg::HudLows => "hud_lows",
            Cfg::HudCoords => "hud_coords",
            Cfg::HudFacing => "hud_facing",
            Cfg::HudOtherDim => "hud_other_dim",
            Cfg::HudCorner => "hud_corner",
            Cfg::ZoomToggle => "zoom_toggle",
            Cfg::ZoomDefault => "zoom_default",
            Cfg::ZoomSmooth => "zoom_smooth_ms",
            Cfg::ZoomScroll => "zoom_scroll",
            Cfg::ZoomMouse => "zoom_mouse",
            Cfg::WpMarkers => "wp_markers",
            Cfg::WpHud => "wp_hud",
            Cfg::WpDeath => "wp_death",
            Cfg::Dark => "dark",
        }
    }

    fn is_bool(self) -> bool {
        !matches!(
            self,
            Cfg::HudCorner | Cfg::ZoomDefault | Cfg::ZoomSmooth | Cfg::ZoomMouse
        )
    }

    fn get(self, c: &Config) -> i32 {
        let b = |v: bool| i32::from(v);
        match self {
            Cfg::Hud => b(c.hud),
            Cfg::HudFps => b(c.hud_fps),
            Cfg::HudLows => b(c.hud_lows),
            Cfg::HudCoords => b(c.hud_coords),
            Cfg::HudFacing => b(c.hud_facing),
            Cfg::HudOtherDim => b(c.hud_other_dim),
            Cfg::HudCorner => c.hud_corner,
            Cfg::ZoomToggle => b(c.zoom_toggle),
            Cfg::ZoomDefault => c.zoom_default,
            Cfg::ZoomSmooth => c.zoom_smooth_ms,
            Cfg::ZoomScroll => b(c.zoom_scroll),
            Cfg::ZoomMouse => c.zoom_mouse,
            Cfg::WpMarkers => b(c.wp_markers),
            Cfg::WpHud => b(c.wp_hud),
            Cfg::WpDeath => b(c.wp_death),
            Cfg::Dark => b(c.dark),
        }
    }

    fn set(self, c: &mut Config, v: i32) {
        let on = v != 0;
        match self {
            Cfg::Hud => c.hud = on,
            Cfg::HudFps => c.hud_fps = on,
            Cfg::HudLows => c.hud_lows = on,
            Cfg::HudCoords => c.hud_coords = on,
            Cfg::HudFacing => c.hud_facing = on,
            Cfg::HudOtherDim => c.hud_other_dim = on,
            Cfg::HudCorner => c.hud_corner = v.clamp(0, 3),
            Cfg::ZoomToggle => c.zoom_toggle = on,
            Cfg::ZoomDefault => c.zoom_default = v.clamp(10, 90),
            Cfg::ZoomSmooth => c.zoom_smooth_ms = v.clamp(0, 500),
            Cfg::ZoomScroll => c.zoom_scroll = on,
            Cfg::ZoomMouse => c.zoom_mouse = v.clamp(10, 100),
            Cfg::WpMarkers => c.wp_markers = on,
            Cfg::WpHud => c.wp_hud = on,
            Cfg::WpDeath => c.wp_death = on,
            Cfg::Dark => c.dark = on,
        }
    }
}

/// Horizon's own settings the screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hz {
    Preset,
    Clouds,
    Particles,
    Smooth,
    Shadows,
    Blend,
    Mipmaps,
    Gamma,
    Leaves,
    Transparency,
    Vignette,
    Fov,
    Bobbing,
    Render,
    Sim,
    MaxFps,
    Vsync,
    Entity,
    Builder,
    Weather,
    Blur,
    CloudRange,
}

impl Hz {
    fn get(self, s: &Settings) -> i32 {
        let b = |v: bool| i32::from(v);
        match self {
            Hz::Preset => super::preset_index(&s.graphics_preset) as i32,
            Hz::Clouds => match s.clouds.as_str() {
                "false" => 0,
                "fast" => 1,
                _ => 2,
            },
            Hz::Particles => i32::from(s.particles.min(2)),
            Hz::Smooth => b(s.smooth_lighting),
            Hz::Shadows => b(s.entity_shadows),
            Hz::Blend => s.biome_blend,
            Hz::Mipmaps => s.mipmap_levels,
            Hz::Gamma => (s.gamma * 100.0).round() as i32,
            Hz::Leaves => b(s.cutout_leaves),
            Hz::Transparency => b(s.improved_transparency),
            Hz::Vignette => b(s.vignette),
            Hz::Fov => s.fov.round() as i32,
            Hz::Bobbing => b(s.view_bobbing),
            Hz::Render => s.render_distance,
            Hz::Sim => s.simulation_distance,
            Hz::MaxFps => s.max_fps.min(260) as i32,
            Hz::Vsync => b(s.vsync),
            Hz::Entity => (s.entity_distance * 100.0).round() as i32,
            Hz::Builder => i32::from(s.prioritize_chunk_updates.min(2)),
            Hz::Weather => s.weather_radius,
            Hz::Blur => s.menu_blur,
            Hz::CloudRange => s.cloud_range,
        }
    }

    fn set(self, s: &mut Settings, v: i32) {
        let on = v != 0;
        match self {
            Hz::Preset => {
                s.graphics_preset = super::PRESETS[v.clamp(0, 3) as usize].into();
                super::apply_preset(s);
            }
            Hz::Clouds => s.clouds = ["false", "fast", "true"][v.clamp(0, 2) as usize].into(),
            Hz::Particles => s.particles = v.clamp(0, 2) as u8,
            Hz::Smooth => s.smooth_lighting = on,
            Hz::Shadows => s.entity_shadows = on,
            Hz::Blend => s.biome_blend = v.clamp(0, 7),
            Hz::Mipmaps => s.mipmap_levels = v.clamp(0, 4),
            Hz::Gamma => s.gamma = v.clamp(0, 100) as f32 / 100.0,
            Hz::Leaves => s.cutout_leaves = on,
            Hz::Transparency => s.improved_transparency = on,
            Hz::Vignette => s.vignette = on,
            Hz::Fov => s.fov = v.clamp(30, 110) as f32,
            Hz::Bobbing => s.view_bobbing = on,
            Hz::Render => s.render_distance = v.clamp(2, 64),
            Hz::Sim => s.simulation_distance = v.clamp(5, 32),
            Hz::MaxFps => s.max_fps = v.clamp(10, 260) as u32,
            Hz::Vsync => s.vsync = on,
            Hz::Entity => s.entity_distance = v.clamp(50, 500) as f32 / 100.0,
            Hz::Builder => s.prioritize_chunk_updates = v.clamp(0, 2) as u8,
            Hz::Weather => s.weather_radius = v.clamp(3, 10),
            Hz::Blur => s.menu_blur = v.clamp(0, 10),
            Hz::CloudRange => s.cloud_range = v.clamp(2, 128),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    Reset,
    Video,
    AddWaypoint,
    DeleteAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Info {
    Status,
    Game,
    Version,
    Api,
    GuiSize,
    Fps,
    Distance,
    Mods,
    Hooks,
}

/// What a row reads, writes or does.
#[derive(Clone, Debug, PartialEq)]
enum Bind {
    Cfg(Cfg),
    Hz(Hz),
    /// One of `KEYS`.
    Key(usize),
    Press(Press),
    Info(Info),
    /// A waypoint's shown switch, by name.
    Waypoint(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Toggle,
    Segmented,
    Dropdown,
    Slider,
    Info,
    Button,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fmt {
    Plain,
    Percent,
    Chunks,
    Ms,
    Fov,
    MaxFps,
    Gamma,
    Blend,
    Mip,
}

impl Fmt {
    fn apply(self, v: i32) -> String {
        match self {
            Fmt::Plain => v.to_string(),
            Fmt::Percent => format!("{v}%"),
            Fmt::Chunks => format!("{v} chunks"),
            Fmt::Ms => {
                if v <= 0 {
                    "Instant".into()
                } else {
                    format!("{v} ms")
                }
            }
            Fmt::Fov => match v {
                70 => "Normal".into(),
                110 => "Quake Pro".into(),
                _ => v.to_string(),
            },
            Fmt::MaxFps => {
                if v >= 260 {
                    "Unlimited".into()
                } else {
                    format!("{v} fps")
                }
            }
            Fmt::Gamma => match v {
                0 => "Moody".into(),
                100 => "Bright".into(),
                _ => format!("{v}%"),
            },
            Fmt::Blend => {
                if v <= 0 {
                    "Off".into()
                } else {
                    let n = 2 * v + 1;
                    format!("{n}x{n}")
                }
            }
            Fmt::Mip => {
                if v <= 0 {
                    "Off".into()
                } else {
                    v.to_string()
                }
            }
        }
    }
}

struct Setting {
    kind: Kind,
    label: String,
    desc: &'static str,
    bind: Bind,
    choices: &'static [&'static str],
    min: i32,
    max: i32,
    step: i32,
    fmt: Fmt,
    button: String,
    member: bool,
    available: bool,
    reason: &'static str,
    pending: i32,
    committed: i32,
    loaded: bool,
    hover: Anim,
    knob: Anim,
    open: Anim,
    slider: (i32, i32),
}

impl Setting {
    fn new(kind: Kind, label: &str, desc: &'static str, bind: Bind) -> Setting {
        Setting {
            kind,
            label: label.to_owned(),
            desc,
            bind,
            choices: &[],
            min: 0,
            max: 1,
            step: 1,
            fmt: Fmt::Plain,
            button: String::new(),
            member: false,
            available: true,
            reason: "",
            pending: 0,
            committed: 0,
            loaded: false,
            hover: Anim::new(0.0, 18.0),
            knob: Anim::new(0.0, 16.0),
            open: Anim::new(0.0, 16.0),
            slider: (0, 1),
        }
    }
    fn toggle(label: &str, desc: &'static str, bind: Bind) -> Setting {
        Setting::new(Kind::Toggle, label, desc, bind)
    }
    fn choice(
        kind: Kind,
        label: &str,
        desc: &'static str,
        choices: &'static [&'static str],
        bind: Bind,
    ) -> Setting {
        let mut s = Setting::new(kind, label, desc, bind);
        s.choices = choices;
        s.max = choices.len() as i32 - 1;
        s
    }
    fn slider(
        label: &str,
        desc: &'static str,
        (min, max, step): (i32, i32, i32),
        fmt: Fmt,
        bind: Bind,
    ) -> Setting {
        let mut s = Setting::new(Kind::Slider, label, desc, bind);
        s.min = min;
        s.max = max.max(min + 1);
        s.step = step.max(1);
        s.fmt = fmt;
        s
    }
    fn info(label: &str, desc: &'static str, info: Info) -> Setting {
        Setting::new(Kind::Info, label, desc, Bind::Info(info))
    }
    fn button(label: &str, desc: &'static str, text: &str, bind: Bind) -> Setting {
        let mut s = Setting::new(Kind::Button, label, desc, bind);
        s.button = text.to_owned();
        s
    }
    fn member(mut self) -> Setting {
        self.member = true;
        self
    }

    fn writable(&self) -> bool {
        matches!(self.bind, Bind::Cfg(_) | Bind::Hz(_) | Bind::Waypoint(_))
    }
    fn editable(&self) -> bool {
        self.writable() && self.available
    }
    fn dirty(&self) -> bool {
        self.writable() && self.pending != self.committed
    }
    fn clamp(&self, v: i32) -> i32 {
        let mut v = v.clamp(self.min, self.max);
        if self.kind == Kind::Slider && self.step > 1 {
            v = self.min + ((v - self.min) as f32 / self.step as f32).round() as i32 * self.step;
            v = v.clamp(self.min, self.max);
        }
        v
    }
    fn stage(&mut self, v: i32) -> bool {
        let next = self.clamp(v);
        if next == self.pending {
            return false;
        }
        self.pending = next;
        true
    }
    fn load(&mut self, ctx: &Ctx) {
        let value = match &self.bind {
            Bind::Cfg(k) => k.get(&ctx.cfg),
            Bind::Hz(h) => h.get(ctx.s),
            Bind::Waypoint(name) => ctx
                .wps
                .iter()
                .find(|w| w.name == *name)
                .map_or(0, |w| i32::from(w.shown)),
            _ => return,
        };
        self.committed = self.clamp(value);
        self.pending = self.committed;
        if !self.loaded {
            self.knob.snap(self.pending.max(0) as f32);
            self.loaded = true;
        }
    }
    fn formatted(&self) -> String {
        match self.kind {
            Kind::Slider => self.fmt.apply(self.pending),
            Kind::Segmented | Kind::Dropdown => self
                .choices
                .get(self.pending.max(0) as usize)
                .copied()
                .unwrap_or("Custom")
                .to_owned(),
            Kind::Toggle => if self.pending != 0 { "ON" } else { "OFF" }.into(),
            Kind::Info | Kind::Button => self.button.clone(),
        }
    }
    fn tooltip(&self) -> &'static str {
        if self.available {
            self.desc
        } else {
            self.reason
        }
    }
}

/// What rows read: Horizon's settings, Aetherium's and the game's state.
struct Ctx<'a> {
    s: &'a Settings,
    cfg: Config,
    wps: &'a [WaypointRow],
    in_world: bool,
}

struct PageDef {
    name: &'static str,
    title: &'static str,
    subtitle: &'static str,
    icon: &'static Bitmap,
    rows: Vec<Setting>,
}

const CORNERS: &[&str] = &["Top left", "Top right", "Bottom left", "Bottom right"];
const PRESET_NAMES: &[&str] = &["Fast", "Fancy", "Fabulous", "Custom"];

fn pages() -> Vec<PageDef> {
    let cfg = |k: Cfg| Bind::Cfg(k);
    let hz = |h: Hz| Bind::Hz(h);
    let general = vec![
        Setting::info("Status", "Aetherium is built into this Horizon as a native mod.", Info::Status),
        Setting::toggle(
            "Info overlay",
            "A small box with the frame rate, your coordinates and the way you face.",
            cfg(Cfg::Hud),
        ),
        Setting::choice(
            Kind::Dropdown,
            "Overlay position",
            "The corner of the screen the overlay sits in. F3 takes the top corners while it is open.",
            CORNERS,
            cfg(Cfg::HudCorner),
        ),
        Setting::toggle("Frame rate", "Frames per second, coloured by how smooth they are.", cfg(Cfg::HudFps)),
        Setting::toggle(
            "1% low and min",
            "The 1% low and the slowest frame of the last seconds, after the frame rate.",
            cfg(Cfg::HudLows),
        ),
        Setting::toggle("Coordinates", "Your block position.", cfg(Cfg::HudCoords)),
        Setting::toggle("Facing", "The direction you look, as a compass point.", cfg(Cfg::HudFacing)),
        Setting::toggle(
            "Nether/Overworld coordinates",
            "Where you would be in the other dimension: divided by 8 in the Overworld, times 8 in the Nether.",
            cfg(Cfg::HudOtherDim),
        ),
        Setting::button(
            "Reset Aetherium settings",
            "Stages Aetherium's own defaults on every tab. Nothing changes until Apply.",
            "Reset",
            Bind::Press(Press::Reset),
        ),
        Setting::button(
            "Vanilla video settings",
            "Applies your changes and opens Horizon's Video Settings.",
            "Open",
            Bind::Press(Press::Video),
        ),
    ];
    let quality = vec![
        Setting::choice(
            Kind::Segmented,
            "Graphics preset",
            "Stages a full set of options. Editing any of them afterwards switches to Custom.",
            PRESET_NAMES,
            hz(Hz::Preset),
        ),
        Setting::choice(
            Kind::Dropdown,
            "Clouds",
            "Fancy clouds are 3D; Fast clouds are flat.",
            &["Off", "Fast", "Fancy"],
            hz(Hz::Clouds),
        )
        .member(),
        Setting::choice(
            Kind::Segmented,
            "Particles",
            "How many particles are drawn. Fewer is faster in busy scenes.",
            &["All", "Less", "Min"],
            hz(Hz::Particles),
        )
        .member(),
        Setting::toggle("Smooth lighting", "Soft shadows between blocks.", hz(Hz::Smooth)).member(),
        Setting::toggle("Entity shadows", "Round shadows under mobs and items.", hz(Hz::Shadows)).member(),
        Setting::slider(
            "Biome blend",
            "How far grass, leaf and water colours blend across biome borders. Higher costs chunk build time.",
            (0, 7, 1),
            Fmt::Blend,
            hz(Hz::Blend),
        )
        .member(),
        Setting::slider(
            "Mipmap levels",
            "Smaller copies of textures for distant blocks: less shimmer far away.",
            (0, 4, 1),
            Fmt::Mip,
            hz(Hz::Mipmaps),
        )
        .member(),
        Setting::slider("Brightness", "Game brightness.", (0, 100, 1), Fmt::Gamma, hz(Hz::Gamma)),
        Setting::toggle(
            "Cutout leaves",
            "Leaves you can see through. Off draws them solid, which is faster in forests.",
            hz(Hz::Leaves),
        )
        .member(),
        Setting::toggle(
            "Improved transparency",
            "Extra passes so water, glass and clouds blend correctly behind each other.",
            hz(Hz::Transparency),
        )
        .member(),
        Setting::toggle("Vignette", "Darkened screen corners.", hz(Hz::Vignette)),
        Setting::toggle("View bobbing", "The camera bobs as you walk.", hz(Hz::Bobbing)),
        Setting::slider("FOV", "Field of view.", (30, 110, 1), Fmt::Fov, hz(Hz::Fov)),
    ];
    let performance = vec![
        Setting::slider(
            "Render distance",
            "Chunks drawn around you. The single biggest FPS cost.",
            (2, 64, 1),
            Fmt::Chunks,
            hz(Hz::Render),
        )
        .member(),
        Setting::slider(
            "Simulation distance",
            "Chunks around you where mobs move and crops grow.",
            (5, 32, 1),
            Fmt::Chunks,
            hz(Hz::Sim),
        )
        .member(),
        Setting::slider(
            "Max frame rate",
            "Caps the frame rate. A cap near your screen's refresh rate saves power and heat.",
            (10, 260, 10),
            Fmt::MaxFps,
            hz(Hz::MaxFps),
        ),
        Setting::toggle("VSync", "Waits for the screen's refresh: no tearing, a little more input delay.", hz(Hz::Vsync)),
        Setting::slider(
            "Entity distance",
            "How far away mobs and items are still drawn.",
            (50, 500, 25),
            Fmt::Percent,
            hz(Hz::Entity),
        )
        .member(),
        Setting::choice(
            Kind::Segmented,
            "Chunk builder",
            "Threaded builds chunks off the main thread; the blocking modes show block changes sooner at some FPS cost.",
            &["Threaded", "Semi", "Blocking"],
            hz(Hz::Builder),
        )
        .member(),
        Setting::slider(
            "Weather radius",
            "How far around you rain and snow are drawn.",
            (3, 10, 1),
            Fmt::Plain,
            hz(Hz::Weather),
        )
        .member(),
        Setting::slider("Menu blur", "Blur behind menus. Off is cheapest.", (0, 10, 1), Fmt::Plain, hz(Hz::Blur))
            .member(),
        Setting::slider(
            "Cloud range",
            "How far clouds reach.",
            (2, 128, 1),
            Fmt::Chunks,
            hz(Hz::CloudRange),
        )
        .member(),
    ];
    let zoom = vec![
        Setting::choice(
            Kind::Segmented,
            "Zoom mode",
            "Hold: zoom while the key is down. Toggle: one press zooms in, the next zooms out.",
            &["Hold", "Toggle"],
            cfg(Cfg::ZoomToggle),
        ),
        Setting::slider(
            "Default zoom",
            "How far in a press zooms before you use the wheel.",
            (10, 90, 5),
            Fmt::Percent,
            cfg(Cfg::ZoomDefault),
        ),
        Setting::slider(
            "Smoothness",
            "How long the zoom eases in and out. 0 is instant.",
            (0, 500, 10),
            Fmt::Ms,
            cfg(Cfg::ZoomSmooth),
        ),
        Setting::toggle(
            "Wheel zoom",
            "While zoomed, the mouse wheel zooms in and out instead of changing the hotbar slot.",
            cfg(Cfg::ZoomScroll),
        ),
        Setting::slider(
            "Mouse speed",
            "How fast the view turns at full zoom, so aiming stays steady.",
            (10, 100, 5),
            Fmt::Percent,
            cfg(Cfg::ZoomMouse),
        ),
        Setting::button("Zoom key", "Click, then press a key. Escape unbinds it.", "", Bind::Key(0)),
        Setting::button("Zoom in key", "Held while zoomed: zooms further in.", "", Bind::Key(1)),
        Setting::button("Zoom out key", "Held while zoomed: zooms back out.", "", Bind::Key(2)),
    ];
    let waypoints = Vec::new();
    let backend = vec![
        Setting::info("Game", "The game this build of Aetherium runs in.", Info::Game),
        Setting::info("Version", "Aetherium's version.", Info::Version),
        Setting::info("Graphics API", "The API Horizon renders with. A change applies at the next start.", Info::Api),
        Setting::info("GUI size", "The screen in GUI pixels, and the GUI scale.", Info::GuiSize),
        Setting::info("Frame rate", "Now, the 1% low and the slowest frame.", Info::Fps),
        Setting::info("Render distance", "Chunks drawn around you.", Info::Distance),
        Setting::info("Native mods", "Mods built into this Horizon.", Info::Mods),
        Setting::info("Hooks", "Where Aetherium plugs into Horizon.", Info::Hooks),
    ];
    vec![
        PageDef {
            name: "General",
            title: "General",
            subtitle: "Overlay, status and shortcuts",
            icon: &GEAR,
            rows: general,
        },
        PageDef {
            name: "Quality",
            title: "Quality",
            subtitle: "How the world looks",
            icon: &SPARKLES,
            rows: quality,
        },
        PageDef {
            name: "Performance",
            title: "Performance",
            subtitle: "Frame rate and draw distance",
            icon: &BARS,
            rows: performance,
        },
        PageDef {
            name: "Zoom",
            title: "Zoom",
            subtitle: "Smooth zoom with wheel control",
            icon: &LENS,
            rows: zoom,
        },
        PageDef {
            name: "Waypoints",
            title: "Waypoints",
            subtitle: "Markers that work on any server",
            icon: &PIN,
            rows: waypoints,
        },
        PageDef {
            name: "Backend",
            title: "Backend",
            subtitle: "Renderer and runtime",
            icon: &CUBE,
            rows: backend,
        },
    ]
}

fn dimension_name(id: &str) -> &str {
    match id {
        "minecraft:overworld" => "Overworld",
        "minecraft:the_nether" => "Nether",
        "minecraft:the_end" => "End",
        other => other.strip_prefix("minecraft:").unwrap_or(other),
    }
}

/// The Waypoints tab's rows for this list.
fn waypoint_rows(wps: &[WaypointRow]) -> Vec<Setting> {
    let mut rows = vec![
        Setting::button(
            "Add waypoint here",
            "Saves where you stand, in this dimension.",
            "Add",
            Bind::Press(Press::AddWaypoint),
        ),
        Setting::button("Add waypoint key", "Click, then press a key. Escape unbinds it.", "", Bind::Key(3)),
        Setting::button(
            "Open Aetherium key",
            "Opens this screen while playing. Click, then press a key.",
            "",
            Bind::Key(4),
        ),
        Setting::toggle(
            "Death waypoint",
            "Saves a waypoint called Death where you die.",
            Bind::Cfg(Cfg::WpDeath),
        ),
        Setting::toggle(
            "Waypoint markers",
            "A marker with the name and distance over each waypoint, at the screen edge when it is off screen.",
            Bind::Cfg(Cfg::WpMarkers),
        ),
        Setting::toggle(
            "Waypoints in the overlay",
            "The nearest waypoints in the Info overlay, with distance and direction.",
            Bind::Cfg(Cfg::WpHud),
        ),
    ];
    for w in wps {
        let label = format!(
            "{}  {} {} {} {}",
            w.name,
            w.x,
            w.y,
            w.z,
            dimension_name(&w.dimension)
        );
        rows.push(Setting::toggle(
            &label,
            "Shows or hides this waypoint. /wp del <name> deletes one.",
            Bind::Waypoint(w.name.clone()),
        ));
    }
    rows.push(Setting::button(
        "Delete all waypoints",
        "Deletes every waypoint. Press twice to confirm.",
        "Delete",
        Bind::Press(Press::DeleteAll),
    ));
    rows
}

// ---------------------------------------------------------------------------------------------
// Animation, colours, pixel art
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Anim {
    value: f32,
    target: f32,
    speed: f32,
}

impl Anim {
    const fn new(initial: f32, speed: f32) -> Anim {
        Anim {
            value: initial,
            target: initial,
            speed,
        }
    }
    fn set(&mut self, target: f32) {
        self.target = target;
    }
    fn snap(&mut self, to: f32) {
        self.value = to;
        self.target = to;
    }
    fn get(&self) -> f32 {
        self.value
    }
    fn tick(&mut self, seconds: f32) {
        if seconds <= 0.0 {
            return;
        }
        let k = 1.0 - (-self.speed * seconds.min(0.25)).exp();
        self.value += (self.target - self.value) * k;
        if (self.target - self.value).abs() < 0.0005 {
            self.value = self.target;
        }
    }
}

fn ease_out(t: f32) -> f32 {
    let inv = 1.0 - t.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

const BACKGROUND: usize = 0;
const BACKGROUND_DOT: usize = 1;
const SURFACE: usize = 2;
const SURFACE_BORDER: usize = 3;
const SURFACE_SHADOW: usize = 4;
const DIVIDER: usize = 5;
const INK: usize = 6;
const INK_SOFT: usize = 7;
const MUTED: usize = 8;
const FAINT: usize = 9;
const TAB_IDLE: usize = 10;
const TAB_HOVER: usize = 11;
const TAB_LIGHT_EDGE: usize = 12;
const TAB_DARK_EDGE: usize = 13;
const CONTROL_DARK: usize = 14;
const CONTROL_DARK_HOVER: usize = 15;
const CONTROL_TRACK: usize = 16;
const CONTROL_TRACK_EDGE: usize = 17;
const TOGGLE_ON: usize = 18;
const TOGGLE_OFF: usize = 19;
const ROW_HOVER: usize = 20;
const SCROLL_TRACK: usize = 21;
const SCROLL_THUMB: usize = 22;
const SCROLL_THUMB_ACTIVE: usize = 23;
const TITLE_SHADOW: usize = 24;

const LIGHT: [u32; 25] = [
    0xFFF3F4F6, 0xFFD9DCE1, 0xFFFFFFFF, 0xFFE5E7EB, 0x14000000, 0xFFF0F1F3, 0xFF111827, 0xFF374151,
    0xFF6B7280, 0xFF9CA3AF, 0xFFE5E7EB, 0xFFDCDFE4, 0xFFFFFFFF, 0xFFC4C8CF, 0xFF1F2937, 0xFF2B3647,
    0xFFE5E7EB, 0xFFD1D5DB, 0xFF22C55E, 0xFFD1D5DB, 0xFFF8F9FA, 0xFFEDEEF1, 0xFFB4B9C2, 0xFF6B7280,
    0x22000000,
];
const DARK: [u32; 25] = [
    0xFF0E1014, 0xFF262A33, 0xFF171A21, 0xFF2B303B, 0x50000000, 0xFF21252D, 0xFFF3F4F6, 0xFFD1D5DB,
    0xFF9CA3AF, 0xFF6B7280, 0xFF1D2129, 0xFF282D37, 0xFF2F3541, 0xFF0A0B0E, 0xFF3A4252, 0xFF4A5366,
    0xFF252A33, 0xFF343A47, 0xFF22C55E, 0xFF3A404C, 0xFF1C2028, 0xFF20242C, 0xFF4B5261, 0xFF8B93A3,
    0x66000000,
];

const BADGE_GOOD: u32 = 0xFF16A34A;
const BADGE_OK: u32 = 0xFFD97706;
const BADGE_BAD: u32 = 0xFFDC2626;
const ACCENT: u32 = 0xFF16A34A;
const SUN: u32 = 0xFFF59E0B;
const MOON: u32 = 0xFFC7D2FE;
const WHITE: u32 = 0xFFFFFFFF;

fn mix(from: u32, to: u32, t: f32) -> u32 {
    let k = t.clamp(0.0, 1.0);
    let ch = |shift: u32| {
        let a = ((from >> shift) & 0xFF) as f32;
        let b = ((to >> shift) & 0xFF) as f32;
        ((a + (b - a) * k) as u32) << shift
    };
    ch(24) | ch(16) | ch(8) | ch(0)
}

fn fade(argb: u32, factor: f32) -> u32 {
    let alpha = (((argb >> 24) & 0xFF) as f32 * factor.clamp(0.0, 1.0)) as u32;
    (alpha << 24) | (argb & 0x00FF_FFFF)
}

/// Java's ARGB to the HUD batch's ABGR (`hud::rgba`).
pub fn abgr(c: u32) -> u32 {
    (c & 0xFF00_FF00) | ((c >> 16) & 0xFF) | ((c & 0xFF) << 16)
}

struct Theme([u32; 25]);

impl Theme {
    fn new(t: f32) -> Theme {
        let mut c = [0; 25];
        for (i, slot) in c.iter_mut().enumerate() {
            *slot = mix(LIGHT[i], DARK[i], t);
        }
        Theme(c)
    }
    fn c(&self, i: usize) -> u32 {
        self.0[i]
    }
}

struct Bitmap(&'static [&'static str]);

impl Bitmap {
    fn width(&self) -> i32 {
        self.0.iter().map(|r| r.len()).max().unwrap_or(0) as i32
    }
    fn draw(&self, c: &mut Canvas, x: i32, y: i32, scale: i32, color: u32) {
        for (ry, row) in self.0.iter().enumerate() {
            let bytes = row.as_bytes();
            let mut col = 0;
            while col < bytes.len() {
                if bytes[col] != b'#' {
                    col += 1;
                    continue;
                }
                let start = col;
                while col < bytes.len() && bytes[col] == b'#' {
                    col += 1;
                }
                let px = x + start as i32 * scale;
                let py = y + ry as i32 * scale;
                c.fill(px, py, px + (col - start) as i32 * scale, py + scale, color);
            }
        }
    }
}

const GLYPH_A: Bitmap = Bitmap(&[".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]);
const GLYPH_E: Bitmap = Bitmap(&["#####", "#....", "#....", "####.", "#....", "#....", "#####"]);
const GLYPH_H: Bitmap = Bitmap(&["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]);
const GLYPH_I: Bitmap = Bitmap(&["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"]);
const GLYPH_M: Bitmap = Bitmap(&["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"]);
const GLYPH_R: Bitmap = Bitmap(&["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]);
const GLYPH_T: Bitmap = Bitmap(&["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]);
const GLYPH_U: Bitmap = Bitmap(&["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]);

fn glyph(ch: char) -> Option<&'static Bitmap> {
    Some(match ch {
        'A' => &GLYPH_A,
        'E' => &GLYPH_E,
        'H' => &GLYPH_H,
        'I' => &GLYPH_I,
        'M' => &GLYPH_M,
        'R' => &GLYPH_R,
        'T' => &GLYPH_T,
        'U' => &GLYPH_U,
        _ => return None,
    })
}

fn pixel_text(c: &mut Canvas, text: &str, x: i32, y: i32, scale: i32, color: u32) {
    let mut cursor = x;
    for ch in text.chars() {
        match glyph(ch) {
            Some(g) => {
                g.draw(c, cursor, y, scale, color);
                cursor += (g.width() + 1) * scale;
            }
            None => cursor += 4 * scale,
        }
    }
}

const GEAR: Bitmap = Bitmap(&[
    "....####....",
    "..#.####.#..",
    ".##########.",
    "..###..###..",
    "####....####",
    "###......###",
    "###......###",
    "####....####",
    "..###..###..",
    ".##########.",
    "..#.####.#..",
    "....####....",
]);
const SPARKLES: Bitmap = Bitmap(&[
    ".....#......",
    ".....#......",
    "....###.....",
    "..#######...",
    "....###.....",
    ".....#...#..",
    ".....#...#..",
    "........###.",
    "..#......#..",
    ".###.....#..",
    "..#.........",
    "............",
]);
const BARS: Bitmap = Bitmap(&[
    "............",
    ".........##.",
    ".........##.",
    "......##.##.",
    "......##.##.",
    "...##.##.##.",
    "...##.##.##.",
    "##.##.##.##.",
    "##.##.##.##.",
    "##.##.##.##.",
    "############",
    "............",
]);
const CUBE: Bitmap = Bitmap(&[
    ".....##.....",
    "...##..##...",
    ".##......##.",
    "#.##....##.#",
    "#...####...#",
    "#.....#....#",
    "#.....#....#",
    "#.....#....#",
    "#.....#....#",
    ".##...#..##.",
    "...##.#.##..",
    ".....###....",
]);
const PIN: Bitmap = Bitmap(&[
    "...######...",
    "..########..",
    ".####..####.",
    ".###....###.",
    ".###....###.",
    ".####..####.",
    "..########..",
    "..########..",
    "...######...",
    "....####....",
    ".....##.....",
    ".....##.....",
]);
const LENS: Bitmap = Bitmap(&[
    "...####.....",
    "..#....#....",
    ".#......#...",
    "#..##....#..",
    "#.#......#..",
    "#........#..",
    "#........#..",
    ".#......#...",
    "..#....##...",
    "...####.###.",
    ".........###",
    "..........##",
]);
const CHEVRON: Bitmap = Bitmap(&["#...#", ".#.#.", "..#.."]);
const SUN_SMALL: Bitmap = Bitmap(&[
    "...#...", ".#...#.", "..###..", "#.###.#", "..###..", ".#...#.", "...#...",
]);
const MOON_SMALL: Bitmap = Bitmap(&[
    "...###.", "..##...", ".##....", ".##....", ".##....", "..##...", "...###.",
]);
const CHECK: Bitmap = Bitmap(&["......#", ".....#.", "#...#..", ".#.#...", "..#...."]);

// ---------------------------------------------------------------------------------------------
// Canvas over the HUD batch
// ---------------------------------------------------------------------------------------------

struct Canvas<'a> {
    batch: &'a mut HudBatch,
    atlas: &'a HudAtlas,
    dx: i32,
    dy: i32,
    alpha: f32,
    clips: Vec<(usize, [i32; 4])>,
}

impl Canvas<'_> {
    fn apply(&self, argb: u32) -> u32 {
        if self.alpha >= 0.999 {
            return argb;
        }
        let a = (((argb >> 24) & 0xFF) as f32 * self.alpha + 0.5) as u32;
        (a.min(255) << 24) | (argb & 0x00FF_FFFF)
    }
    fn offset(&mut self, dx: i32, dy: i32) {
        self.dx = dx;
        self.dy = dy;
    }
    fn fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, argb: u32) {
        if x2 <= x1 || y2 <= y1 {
            return;
        }
        let color = self.apply(argb);
        if color >> 24 == 0 {
            return;
        }
        self.batch.fill(
            self.atlas,
            (x1 + self.dx) as f32,
            (y1 + self.dy) as f32,
            (x2 - x1) as f32,
            (y2 - y1) as f32,
            abgr(color),
        );
    }
    fn text(&mut self, text: &str, x: i32, y: i32, argb: u32) {
        if text.is_empty() {
            return;
        }
        let color = self.apply(argb);
        if color >> 24 < 8 {
            return;
        }
        let _ = self.batch.text(
            self.atlas,
            text,
            (x + self.dx) as f32,
            (y + self.dy) as f32,
            abgr(color),
            false,
        );
    }
    fn width(&self, text: &str) -> i32 {
        if text.is_empty() {
            0
        } else {
            self.atlas.font.width(text).ceil() as i32
        }
    }
    fn push_clip(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut r = [x1 + self.dx, y1 + self.dy, x2 + self.dx, y2 + self.dy];
        if let Some((_, p)) = self.clips.last() {
            r = [r[0].max(p[0]), r[1].max(p[1]), r[2].min(p[2]), r[3].min(p[3])];
        }
        r[2] = r[2].max(r[0]);
        r[3] = r[3].max(r[1]);
        self.clips.push((self.batch.vertices.len(), r));
    }
    fn pop_clip(&mut self) {
        if let Some((start, r)) = self.clips.pop() {
            self.batch
                .clip_since(start, r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
        }
    }
    fn round_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        self.fill(x1 + 1, y1, x2 - 1, y2, color);
        self.fill(x1, y1 + 1, x1 + 1, y2 - 1, color);
        self.fill(x2 - 1, y1 + 1, x2, y2 - 1, color);
    }
    /// `text` cut to `max` pixels with "...".
    fn fit(&self, text: &str, max: i32) -> String {
        if max <= 0 {
            return String::new();
        }
        if self.width(text) <= max {
            return text.to_owned();
        }
        let budget = max - self.width("...");
        let ends: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .skip(1)
            .chain(std::iter::once(text.len()))
            .collect();
        let (mut lo, mut hi) = (0usize, ends.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if self.width(&text[..ends[mid - 1]]) <= budget {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        if lo == 0 {
            String::new()
        } else {
            format!("{}...", &text[..ends[lo - 1]])
        }
    }
}

/// Words of `text` in lines at most `max` wide (`measure` gives a width).
fn wrap(text: &str, max: i32, measure: &dyn Fn(&str) -> i32) -> Vec<String> {
    let mut out = Vec::new();
    if max <= 0 {
        return out;
    }
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if measure(&candidate) <= max {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            out.push(std::mem::take(&mut line));
        }
        let mut rest = word.to_owned();
        while measure(&rest) > max && rest.chars().count() > 1 {
            let chars: Vec<(usize, char)> = rest.char_indices().collect();
            let mut cut = chars.len() - 1;
            while cut > 1 && measure(&rest[..chars[cut].0]) > max {
                cut -= 1;
            }
            let at = chars[cut].0;
            out.push(rest[..at].to_owned());
            rest = rest[at..].to_owned();
        }
        line = rest;
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn inside(x: i32, y: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    x >= x1 && x < x2 && y >= y1 && y < y2
}

// ---------------------------------------------------------------------------------------------
// The view
// ---------------------------------------------------------------------------------------------

const DOTS: usize = 28;
const THEME_W: i32 = 30;
const THEME_H: i32 = 14;
const FLING_FRICTION: f32 = 5.0;
const FLING_MIN: f32 = 40.0;
const TIP_DELAY: f32 = 0.35;
const TIP_PAD: i32 = 5;
const TIP_LINE_H: i32 = 10;

#[derive(Clone, Copy, Default)]
struct Layout {
    margin: i32,
    header_y: i32,
    header_h: i32,
    body_top: i32,
    body_bottom: i32,
    footer_y: i32,
    side_x: i32,
    side_w: i32,
    tab_h: i32,
    tab_gap: i32,
    compact: bool,
    px1: i32,
    py1: i32,
    px2: i32,
    py2: i32,
    rows_top: i32,
    rows_bottom: i32,
    row_h: i32,
    ctrl_w: i32,
    btn_y: i32,
    btn_h: i32,
    apply_x1: i32,
    apply_x2: i32,
    done_x1: i32,
    done_x2: i32,
    sb_x1: i32,
    sb_x2: i32,
}

impl Layout {
    fn ctrl_x2(&self) -> i32 {
        self.sb_x1 - 8
    }
    fn ctrl_x1(&self) -> i32 {
        self.ctrl_x2() - self.ctrl_w
    }
}

/// What an input asks of Options (the screen itself can't reach the settings there).
#[derive(Default)]
struct Out {
    sound: bool,
    apply: bool,
    done: bool,
    video: bool,
    bind: Option<usize>,
    dark: Option<bool>,
    actions: Vec<Action>,
}

struct View {
    /// The mod's index in `mods.installed`.
    index: usize,
    pages: Vec<PageDef>,
    selected: usize,
    width: i32,
    height: i32,
    last: Instant,
    clock: f32,
    open: Anim,
    tab: Anim,
    content: Anim,
    scroll: Anim,
    toast: Anim,
    apply_hover: Anim,
    done_hover: Anim,
    tab_hover: Vec<Anim>,
    theme: Anim,
    theme_hover: Anim,
    sb_hover: Anim,
    tip: Anim,
    scroll_target: f32,
    velocity: f32,
    press: Option<(i32, i32)>,
    drag_start: (f32, f32),
    last_drag: (f32, Instant),
    thumb_drag: bool,
    thumb_grab: f32,
    dark: bool,
    max_scroll: i32,
    dragging: Option<usize>,
    scroll_dragging: bool,
    dropdown: Option<usize>,
    closing: Option<usize>,
    hovered: Option<usize>,
    tip_for: Option<usize>,
    tip_shown: Option<usize>,
    tip_hold: f32,
    toast_text: String,
    fps_shown: (u32, u32, u32),
    fps_timer: f32,
    dots: [(f32, f32, f32, i32); DOTS],
    l: Layout,
    theme_box: [i32; 4],
    wps: Vec<WaypointRow>,
    wp_gen: u64,
    confirm_delete: bool,
    preview_preset: bool,
}

impl View {
    fn new(index: usize, ctx: &Ctx, tab: usize, wp_gen: u64) -> View {
        let pages = pages();
        let tabs = pages.len();
        let mut seed: u32 = 0x00AE_7E21;
        let mut next = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        let mut dots = [(0.0, 0.0, 0.0, 1); DOTS];
        for d in dots.iter_mut() {
            *d = (next(), next(), 0.006 + next() * 0.014, 1 + (next() * 2.0) as i32);
        }
        let selected = tab.min(tabs - 1);
        let mut view = View {
            index,
            pages,
            selected,
            width: 320,
            height: 240,
            last: Instant::now(),
            clock: 0.0,
            open: Anim::new(0.0, 9.0),
            tab: Anim::new(selected as f32, 16.0),
            content: Anim::new(1.0, 11.0),
            scroll: Anim::new(0.0, 18.0),
            toast: Anim::new(0.0, 2.2),
            apply_hover: Anim::new(0.0, 18.0),
            done_hover: Anim::new(0.0, 18.0),
            tab_hover: vec![Anim::new(0.0, 18.0); tabs],
            theme: Anim::new(if ctx.cfg.dark { 1.0 } else { 0.0 }, 10.0),
            theme_hover: Anim::new(0.0, 18.0),
            sb_hover: Anim::new(0.0, 16.0),
            tip: Anim::new(0.0, 16.0),
            scroll_target: 0.0,
            velocity: 0.0,
            press: None,
            drag_start: (0.0, 0.0),
            last_drag: (0.0, Instant::now()),
            thumb_drag: false,
            thumb_grab: 0.0,
            dark: ctx.cfg.dark,
            max_scroll: 0,
            dragging: None,
            scroll_dragging: false,
            dropdown: None,
            closing: None,
            hovered: None,
            tip_for: None,
            tip_shown: None,
            tip_hold: 0.0,
            toast_text: String::new(),
            fps_shown: (0, 0, 0),
            fps_timer: 0.0,
            dots,
            l: Layout::default(),
            theme_box: [-1, 0, -1, 0],
            wps: ctx.wps.to_vec(),
            wp_gen,
            confirm_delete: false,
            preview_preset: false,
        };
        view.open.set(1.0);
        view.pages[TAB_WAYPOINTS].rows = waypoint_rows(ctx.wps);
        view.reload(ctx);
        view
    }

    fn reload(&mut self, ctx: &Ctx) {
        for page in &mut self.pages {
            for row in &mut page.rows {
                row.load(ctx);
            }
        }
    }

    fn rows(&self) -> &[Setting] {
        &self.pages[self.selected].rows
    }

    fn row_top(&self, i: usize) -> i32 {
        self.l.rows_top + i as i32 * self.l.row_h - self.scroll.get().round() as i32
    }

    fn tab_top(&self, i: usize) -> i32 {
        self.l.body_top + i as i32 * (self.l.tab_h + self.l.tab_gap)
    }

    fn layout(&mut self) {
        let (w, h) = (self.width, self.height);
        let mut l = Layout {
            margin: if w >= 480 { 10 } else { 5 },
            header_h: 20,
            ..Layout::default()
        };
        l.header_y = l.margin;
        l.footer_y = h - l.margin - 9;
        l.body_top = l.header_y + l.header_h + 4;
        l.body_bottom = l.footer_y - 5;
        l.compact = w < 400;
        l.side_w = if l.compact { 26 } else { (w / 6).clamp(84, 104) };
        l.side_x = l.margin;
        let count = self.pages.len().max(1) as i32;
        l.tab_gap = 3;
        l.tab_h = ((l.body_bottom - l.body_top - (count - 1) * l.tab_gap) / count).clamp(14, 24);
        l.px1 = l.side_x + l.side_w + 6;
        l.px2 = w - l.margin;
        l.py1 = l.body_top;
        l.py2 = l.body_bottom;
        l.row_h = 20;
        l.btn_h = 16;
        l.btn_y = l.py2 - 6 - l.btn_h;
        l.rows_top = l.py1 + 34;
        l.rows_bottom = l.btn_y - 5;
        l.sb_x2 = l.px2 - 4;
        l.sb_x1 = l.sb_x2 - 6;
        l.ctrl_w = ((l.ctrl_x2() - l.px1 - 10) * 48 / 100).clamp(70, 150);
        l.done_x2 = l.px2 - 8;
        l.done_x1 = l.done_x2 - 46;
        l.apply_x2 = l.done_x1 - 5;
        l.apply_x1 = l.apply_x2 - 46;
        let content = self.rows().len() as i32 * l.row_h;
        self.max_scroll = (content - (l.rows_bottom - l.rows_top)).max(0);
        self.l = l;
    }

    fn clamp_scroll(&mut self) {
        self.scroll_target = self.scroll_target.clamp(0.0, self.max_scroll as f32);
    }

    fn row_at(&self, x: i32, y: i32) -> Option<usize> {
        let l = &self.l;
        if x < l.px1 || x >= l.sb_x1 - 4 || y < l.rows_top || y >= l.rows_bottom {
            return None;
        }
        let index = (y - l.rows_top + self.scroll.get().round() as i32) / l.row_h;
        (index >= 0 && (index as usize) < self.rows().len()).then_some(index as usize)
    }

    fn thumb_height(&self) -> i32 {
        let track = self.l.rows_bottom - self.l.rows_top;
        let content = track + self.max_scroll;
        track.min((track * track / content.max(1)).max(18))
    }

    fn thumb_top(&self) -> i32 {
        let travel = (self.l.rows_bottom - self.l.rows_top) - self.thumb_height();
        let position = if self.max_scroll <= 0 {
            0.0
        } else {
            (self.scroll.get() / self.max_scroll as f32).clamp(0.0, 1.0)
        };
        self.l.rows_top + (travel as f32 * position).round() as i32
    }

    fn over_scrollbar(&self, x: i32, y: i32) -> bool {
        x >= self.l.sb_x1 - 4 && x < self.l.px2 && y >= self.l.rows_top && y < self.l.rows_bottom
    }

    fn any_dirty(&self) -> bool {
        self.pages.iter().any(|p| p.rows.iter().any(Setting::dirty))
    }

    fn preset_pending(&self) -> Option<i32> {
        self.pages
            .iter()
            .flat_map(|p| p.rows.iter())
            .find(|r| r.bind == Bind::Hz(Hz::Preset))
            .map(|r| r.pending)
    }

    // ------------------------------------------------------------------ per frame

    /// Brings rows up to date with the game: the waypoint list, a picked preset's values, keys
    /// and what is available.
    fn sync(&mut self, ctx: &Ctx, gen: u64, ui: &OptionsUi) {
        if gen != self.wp_gen {
            self.wp_gen = gen;
            self.wps = ctx.wps.to_vec();
            let mut rows = waypoint_rows(ctx.wps);
            // Keep staged switches of waypoints still there.
            for row in &mut rows {
                row.load(ctx);
                if let Some(old) = self.pages[TAB_WAYPOINTS]
                    .rows
                    .iter()
                    .find(|o| o.bind == row.bind && o.label == row.label)
                {
                    row.pending = old.pending;
                    row.knob = old.knob;
                    row.hover = old.hover;
                }
            }
            self.pages[TAB_WAYPOINTS].rows = rows;
            self.confirm_delete = false;
            if self.selected == TAB_WAYPOINTS {
                self.dropdown = None;
                self.closing = None;
                self.dragging = None;
                self.hovered = None;
                self.tip_for = None;
                self.tip_shown = None;
            }
        }
        if std::mem::take(&mut self.preview_preset)
            && let Some(p) = self.preset_pending().filter(|p| (0..3).contains(p))
        {
            // The preset's options, staged so the rows show what Apply will set.
            let mut preview = ctx.s.clone();
            Hz::Preset.set(&mut preview, p);
            let pctx = Ctx {
                s: &preview,
                cfg: ctx.cfg,
                wps: ctx.wps,
                in_world: ctx.in_world,
            };
            for page in &mut self.pages {
                for row in &mut page.rows {
                    if row.member
                        && let Bind::Hz(h) = row.bind
                    {
                        row.stage(h.get(pctx.s));
                    }
                }
            }
        }
        let waiting = ui.selected_native_key.filter(|_| ui.selected_key.is_some());
        let wp_count = self.wps.len();
        for page in &mut self.pages {
            for row in &mut page.rows {
                match &row.bind {
                    Bind::Key(k) => {
                        let [name, _, default] = KEYS[*k];
                        let chosen = ctx
                            .s
                            .mods
                            .keys
                            .get(ID)
                            .and_then(|m| m.get(name))
                            .map_or(default, String::as_str);
                        let label = if chosen == UNBOUND {
                            "None".to_owned()
                        } else {
                            let name = crate::keybinds::display_name(chosen).plain();
                            if name.starts_with("key.") {
                                crate::lang::get(&name)
                            } else {
                                name
                            }
                        };
                        let waiting_here = waiting.is_some_and(|(i, slot)| {
                            i == self.index && key_slot(ctx.s, i, name) == Some(slot)
                        });
                        row.button = if waiting_here {
                            format!("> {label} <")
                        } else {
                            label
                        };
                    }
                    Bind::Press(Press::AddWaypoint) => {
                        row.available = ctx.in_world;
                        row.reason = "Join a world first.";
                    }
                    Bind::Press(Press::DeleteAll) => {
                        row.available = wp_count > 0;
                        row.reason = "There are no waypoints.";
                        row.button = if self.confirm_delete { "Confirm" } else { "Delete" }.into();
                    }
                    _ => {}
                }
            }
        }
    }

    fn info_text(&self, info: Info, s: &Settings, fps: (u32, u32, u32)) -> String {
        match info {
            Info::Status => "Running".into(),
            Info::Game => "Horizon, Minecraft 26.3".into(),
            Info::Version => format!("Aetherium {VERSION} (native)"),
            Info::Api => {
                let name = |a: crate::graphics::GraphicsApi| match a {
                    crate::graphics::GraphicsApi::Auto => "Auto",
                    crate::graphics::GraphicsApi::Vulkan => "Vulkan",
                    crate::graphics::GraphicsApi::Dx12 => "DirectX 12",
                    crate::graphics::GraphicsApi::OpenGl => "OpenGL",
                };
                if s.horizon.api == s.startup_api {
                    name(s.startup_api).into()
                } else {
                    format!("{} (next start: {})", name(s.startup_api), name(s.horizon.api))
                }
            }
            Info::GuiSize => {
                let scale = if s.gui_scale == 0 {
                    "Auto".to_owned()
                } else {
                    s.gui_scale.to_string()
                };
                format!("{} x {}, scale {scale}", self.width, self.height)
            }
            Info::Fps => format!("{} FPS, 1% {}, min {}", fps.0, fps.2, fps.1),
            Info::Distance => format!("{} chunks", s.render_distance),
            Info::Mods => format!(
                "{} installed",
                s.mods.installed.iter().filter(|m| m.native.is_some()).count()
            ),
            Info::Hooks => "Options, HUD, keys, zoom, chat".into(),
        }
    }

    fn tick(&mut self, dt: f32, mx: i32, my: i32, fps: (u32, u32, u32)) {
        self.open.tick(dt);
        self.theme.set(if self.dark { 1.0 } else { 0.0 });
        self.theme.tick(dt);
        let [tx1, ty1, tx2, ty2] = self.theme_box;
        self.theme_hover
            .set(if tx1 >= 0 && inside(mx, my, tx1, ty1, tx2, ty2) { 1.0 } else { 0.0 });
        self.theme_hover.tick(dt);
        let over_sb = self.max_scroll > 0 && self.over_scrollbar(mx, my);
        self.sb_hover
            .set(if self.thumb_drag || over_sb { 1.0 } else { 0.0 });
        self.sb_hover.tick(dt);
        if self.velocity != 0.0 && !self.scroll_dragging && !self.thumb_drag {
            self.scroll_target += self.velocity * dt;
            self.velocity *= (-FLING_FRICTION * dt).exp();
            if self.velocity.abs() < FLING_MIN {
                self.velocity = 0.0;
            }
            if self.scroll_target <= 0.0 || self.scroll_target >= self.max_scroll as f32 {
                self.velocity = 0.0;
            }
            self.clamp_scroll();
            self.scroll.snap(self.scroll_target);
        }
        self.tab.set(self.selected as f32);
        self.tab.tick(dt);
        self.content.tick(dt);
        self.scroll.set(self.scroll_target);
        self.scroll.tick(dt);
        self.toast.set(0.0);
        self.toast.tick(dt);
        let l = self.l;
        self.apply_hover.set(
            if inside(mx, my, l.apply_x1, l.btn_y, l.apply_x2, l.btn_y + l.btn_h) { 1.0 } else { 0.0 },
        );
        self.apply_hover.tick(dt);
        self.done_hover.set(
            if inside(mx, my, l.done_x1, l.btn_y, l.done_x2, l.btn_y + l.btn_h) { 1.0 } else { 0.0 },
        );
        self.done_hover.tick(dt);
        for i in 0..self.tab_hover.len() {
            let top = self.tab_top(i);
            let over = i != self.selected && inside(mx, my, l.side_x, top, l.side_x + l.side_w, top + l.tab_h);
            self.tab_hover[i].set(if over { 1.0 } else { 0.0 });
            self.tab_hover[i].tick(dt);
        }
        self.fps_timer -= dt;
        if self.fps_timer <= 0.0 {
            self.fps_timer = 0.25;
            self.fps_shown = fps;
        }
        self.hovered = None;
        if self.dropdown.is_none() && !self.thumb_drag {
            self.hovered = self.row_at(mx, my);
        }
        if self.dragging.is_some() {
            self.hovered = self.dragging;
        }
        let (hovered, dropdown) = (self.hovered, self.dropdown);
        for (i, row) in self.pages[self.selected].rows.iter_mut().enumerate() {
            let hot = Some(i) == hovered && row.available && row.kind != Kind::Info;
            row.hover.set(if hot { 1.0 } else { 0.0 });
            row.hover.tick(dt);
            if row.pending >= 0 {
                row.knob.set(row.pending as f32);
            }
            row.knob.tick(dt);
            row.open.set(if Some(i) == dropdown { 1.0 } else { 0.0 });
            row.open.tick(dt);
        }
        if let Some(c) = self.closing
            && self.rows().get(c).is_none_or(|r| r.open.get() <= 0.01)
        {
            self.closing = None;
        }
        self.tick_tooltip(dt);
    }

    fn tick_tooltip(&mut self, dt: f32) {
        let allowed = self.hovered.filter(|&i| {
            self.dragging.is_none()
                && self.dropdown.is_none()
                && !self.scroll_dragging
                && !self.thumb_drag
                && self.velocity == 0.0
                && !self.rows()[i].tooltip().is_empty()
        });
        match allowed {
            None => {
                self.tip_for = None;
                self.tip_hold = 0.0;
            }
            Some(i) if Some(i) != self.tip_for => {
                let open = self.tip.get() > 0.5;
                self.tip_for = Some(i);
                self.tip_hold = if open { TIP_DELAY } else { 0.0 };
            }
            Some(_) => self.tip_hold += dt,
        }
        let show = self.tip_for.is_some() && self.tip_hold >= TIP_DELAY;
        if show {
            self.tip_shown = self.tip_for;
        }
        self.tip.set(if show { 1.0 } else { 0.0 });
        self.tip.tick(dt);
    }

    // ------------------------------------------------------------------ drawing

    fn render(&mut self, c: &mut Canvas, mx: i32, my: i32, dt: f32, s: &Settings, fps: (u32, u32, u32)) {
        self.clock += dt;
        self.layout();
        self.tick(dt, mx, my, fps);
        let t = Theme::new(self.theme.get());
        let open = ease_out(self.open.get());
        let lift = ((1.0 - open) * 10.0).round() as i32;
        // The whole screen first: nothing of the world or Horizon's page shows through.
        c.alpha = 1.0;
        c.offset(0, 0);
        c.fill(0, 0, self.width, self.height, t.c(BACKGROUND) | 0xFF00_0000);
        self.draw_dots(c, dt, open, &t);
        c.alpha = open;
        c.offset(0, lift);
        self.draw_header(c, &t);
        self.draw_sidebar(c, &t);
        self.draw_panel(c, mx, my, lift, &t, s, fps);
        self.draw_footer(c, &t, s);
        self.draw_dropdown(c, mx, my, &t);
        self.draw_tooltip(c, &t);
        c.alpha = 1.0;
        c.offset(0, 0);
    }

    fn draw_dots(&mut self, c: &mut Canvas, dt: f32, open: f32, t: &Theme) {
        let color = fade(t.c(BACKGROUND_DOT), open);
        for (i, d) in self.dots.iter_mut().enumerate() {
            d.1 -= d.2 * dt;
            if d.1 < 0.0 {
                d.1 += 1.0;
            }
            let x = (d.0 * self.width as f32 + ((self.clock * 0.6 + i as f32).sin() * 4.0)) as i32;
            let y = (d.1 * self.height as f32) as i32;
            c.fill(x, y, x + d.3, y + d.3, color);
        }
    }

    fn draw_header(&mut self, c: &mut Canvas, t: &Theme) {
        let l = self.l;
        let scale = if self.height >= 260 && !l.compact { 2 } else { 1 };
        let title_y = l.header_y + (l.header_h - 7 * scale) / 2;
        let title_x = l.margin + 2;
        pixel_text(c, "AETHERIUM", title_x + scale, title_y + scale, scale, t.c(TITLE_SHADOW));
        pixel_text(c, "AETHERIUM", title_x, title_y, scale, t.c(INK));
        let fps = self.fps_shown.0;
        let fps_text = format!("FPS: {fps}");
        let badge_w = c.width(&fps_text) + 14;
        let badge_h = 14;
        let x2 = self.width - l.margin;
        let x1 = x2 - badge_w;
        let y1 = l.header_y + (l.header_h - badge_h) / 2;
        let color = if fps >= 50 {
            BADGE_GOOD
        } else if fps >= 30 {
            BADGE_OK
        } else {
            BADGE_BAD
        };
        c.round_rect(x1, y1, x2, y1 + badge_h, color);
        let pulse = 0.55 + 0.45 * (self.clock * 2.0).sin().abs();
        c.fill(x1 + 4, y1 + 6, x1 + 6, y1 + 8, fade(WHITE, pulse));
        c.text(&fps_text, x1 + 9, y1 + 3, WHITE);
        self.draw_theme_switch(c, x1 - 6, t);
    }

    fn draw_theme_switch(&mut self, c: &mut Canvas, right: i32, t: &Theme) {
        let x2 = right;
        let x1 = x2 - THEME_W;
        let y1 = self.l.header_y + (self.l.header_h - THEME_H) / 2;
        let y2 = y1 + THEME_H;
        self.theme_box = [x1, y1, x2, y2];
        let k = self.theme.get();
        let hover = self.theme_hover.get();
        c.round_rect(x1, y1, x2, y2, mix(t.c(SURFACE_BORDER), t.c(CONTROL_TRACK_EDGE), hover));
        c.round_rect(x1 + 1, y1 + 1, x2 - 1, y2 - 1, mix(t.c(TAB_IDLE), t.c(TAB_HOVER), hover));
        SUN_SMALL.draw(c, x1 + 3, y1 + 4, 1, fade(t.c(MUTED), 0.55));
        MOON_SMALL.draw(c, x2 - 10, y1 + 4, 1, fade(t.c(MUTED), 0.55));
        let knob_w = 14;
        let knob_x = x1 + 1 + (k.clamp(0.0, 1.0) * (THEME_W - 2 - knob_w) as f32).round() as i32;
        c.round_rect(knob_x, y1 + 1, knob_x + knob_w, y2 - 1, mix(WHITE, 0xFF3A_4252, k));
        if k < 0.98 {
            SUN_SMALL.draw(c, knob_x + 4, y1 + 4, 1, fade(SUN, 1.0 - k));
        }
        if k > 0.02 {
            MOON_SMALL.draw(c, knob_x + 3, y1 + 4, 1, fade(MOON, k));
        }
    }

    fn draw_sidebar(&self, c: &mut Canvas, t: &Theme) {
        let l = self.l;
        for i in 0..self.pages.len() {
            let top = self.tab_top(i);
            let (x1, x2, bottom) = (l.side_x, l.side_x + l.side_w, top + l.tab_h);
            c.fill(x1, top, x2, bottom, mix(t.c(TAB_IDLE), t.c(TAB_HOVER), self.tab_hover[i].get()));
            c.fill(x1, top, x2, top + 1, t.c(TAB_LIGHT_EDGE));
            c.fill(x1, top, x1 + 1, bottom, t.c(TAB_LIGHT_EDGE));
            c.fill(x1, bottom - 1, x2, bottom, t.c(TAB_DARK_EDGE));
            c.fill(x2 - 1, top, x2, bottom, t.c(TAB_DARK_EDGE));
        }
        let sel = (l.body_top as f32 + self.tab.get() * (l.tab_h + l.tab_gap) as f32).round() as i32;
        c.fill(l.side_x, sel, l.side_x + l.side_w, sel + l.tab_h, t.c(INK));
        c.fill(l.side_x + 1, sel + 1, l.side_x + l.side_w - 1, sel + l.tab_h - 1, t.c(SURFACE));
        c.fill(l.side_x + 1, sel + l.tab_h - 1, l.side_x + l.side_w - 1, sel + l.tab_h, t.c(INK));
        for (i, page) in self.pages.iter().enumerate() {
            let top = self.tab_top(i);
            let active = i == self.selected;
            let ink = if active { t.c(INK) } else { t.c(INK_SOFT) };
            let icon_x = if l.compact {
                l.side_x + (l.side_w - 12) / 2
            } else {
                l.side_x + 6
            };
            page.icon.draw(c, icon_x, top + (l.tab_h - 12) / 2, 1, ink);
            if !l.compact {
                let name = c.fit(page.name, l.side_w - 26);
                c.text(&name, l.side_x + 22, top + (l.tab_h - 8) / 2, ink);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_panel(
        &self,
        c: &mut Canvas,
        mx: i32,
        my: i32,
        lift: i32,
        t: &Theme,
        s: &Settings,
        fps: (u32, u32, u32),
    ) {
        let l = self.l;
        c.fill(l.px1 + 2, l.py2, l.px2 + 1, l.py2 + 2, t.c(SURFACE_SHADOW));
        c.fill(l.px1, l.py1, l.px2, l.py2, t.c(SURFACE_BORDER));
        c.fill(l.px1 + 1, l.py1 + 1, l.px2 - 1, l.py2 - 1, t.c(SURFACE));
        let page = &self.pages[self.selected];
        let base = c.alpha;
        let content = ease_out(self.content.get());
        let slide = ((1.0 - content) * 14.0).round() as i32;
        c.alpha = base * content;
        c.offset(slide, lift);
        let text_x = l.px1 + 10;
        c.text(page.title, text_x, l.py1 + 8, t.c(INK));
        c.text(page.title, text_x + 1, l.py1 + 8, t.c(INK));
        let subtitle = c.fit(page.subtitle, l.px2 - text_x - 10);
        c.text(&subtitle, text_x, l.py1 + 20, t.c(MUTED));
        c.offset(0, lift);
        c.alpha = base;
        c.fill(l.px1 + 8, l.py1 + 31, l.px2 - 8, l.py1 + 32, t.c(SURFACE_BORDER));
        c.push_clip(l.px1 + 1, l.rows_top, l.px2 - 1, l.rows_bottom);
        c.alpha = base * content;
        c.offset(slide, lift);
        for (i, row) in page.rows.iter().enumerate() {
            let top = self.row_top(i);
            if top + l.row_h < l.rows_top || top > l.rows_bottom {
                continue;
            }
            self.draw_row(c, row, i, top, mx, my, t, s, fps);
        }
        c.offset(0, lift);
        c.alpha = base;
        c.pop_clip();
        if self.max_scroll > 0 {
            self.draw_scrollbar(c, t);
        }
        self.draw_button_bar(c, t);
    }

    fn draw_scrollbar(&self, c: &mut Canvas, t: &Theme) {
        let l = self.l;
        let hover = self.sb_hover.get();
        let inset = ((1.0 - hover) * 1.0).round() as i32;
        let (x1, x2) = (l.sb_x1 + inset, l.sb_x2 - inset);
        c.round_rect(l.sb_x1, l.rows_top, l.sb_x2, l.rows_bottom, t.c(SCROLL_TRACK));
        let thumb_y = self.thumb_top();
        let thumb_h = self.thumb_height();
        let color = mix(t.c(SCROLL_THUMB), t.c(SCROLL_THUMB_ACTIVE), hover);
        c.round_rect(x1, thumb_y, x2, thumb_y + thumb_h, color);
        if l.sb_x2 - l.sb_x1 >= 6 && thumb_h >= 14 {
            let mid = thumb_y + thumb_h / 2;
            let grip = fade(t.c(SURFACE), 0.55 + 0.35 * hover);
            for i in -1..=1 {
                c.fill(x1 + 2, mid + i * 2, x2 - 2, mid + i * 2 + 1, grip);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &self,
        c: &mut Canvas,
        row: &Setting,
        index: usize,
        top: i32,
        mx: i32,
        my: i32,
        t: &Theme,
        s: &Settings,
        fps: (u32, u32, u32),
    ) {
        let l = self.l;
        let hover = row.hover.get();
        if hover > 0.01 {
            c.fill(l.px1 + 1, top, l.sb_x1 - 3, top + l.row_h, fade(t.c(ROW_HOVER), hover));
        }
        c.fill(l.px1 + 10, top + l.row_h - 1, l.ctrl_x2(), top + l.row_h, t.c(DIVIDER));
        let base = c.alpha;
        if !row.available {
            c.alpha = base * 0.45;
        }
        let (cx1, cx2) = (l.ctrl_x1(), l.ctrl_x2());
        let text_y = top + (l.row_h - 8) / 2;
        let label_limit = if matches!(row.kind, Kind::Info | Kind::Button | Kind::Toggle) {
            cx2 - 60
        } else {
            cx1 - 6
        } - (l.px1 + 10);
        let label = c.fit(&row.label, label_limit);
        c.text(&label, l.px1 + 10, text_y, t.c(INK));
        match row.kind {
            Kind::Toggle => self.draw_toggle(c, row, top, cx2, t),
            Kind::Segmented => self.draw_segmented(c, row, top, cx1, cx2, mx, my, t),
            Kind::Dropdown => self.draw_dropdown_header(c, row, index, top, cx1, cx2, mx, my, t),
            Kind::Slider => self.draw_slider(c, row, index, top, cx1, cx2, t),
            Kind::Info => {
                let value = match &row.bind {
                    Bind::Info(info) => self.info_text(*info, s, fps),
                    _ => String::new(),
                };
                let room = cx2 - (l.px1 + 10) - c.width(&row.label) - 12;
                let text = c.fit(&value, room);
                let w = c.width(&text);
                c.text(&text, cx2 - w, text_y, t.c(MUTED));
            }
            Kind::Button => {
                let w = c.width(&row.button) + 16;
                let h = 16;
                let x1 = cx2 - w;
                let y1 = top + (l.row_h - h) / 2;
                let over = row.available && inside(mx, my, x1, y1, cx2, y1 + h);
                c.fill(x1, y1, cx2, y1 + h, t.c(INK));
                c.fill(
                    x1 + 1,
                    y1 + 1,
                    cx2 - 1,
                    y1 + h - 1,
                    if over { t.c(TAB_IDLE) } else { t.c(SURFACE) },
                );
                c.text(&row.button, x1 + 8, y1 + 4, t.c(INK));
            }
        }
        c.alpha = base;
    }

    fn draw_toggle(&self, c: &mut Canvas, row: &Setting, top: i32, cx2: i32, t: &Theme) {
        let (pill_w, pill_h) = (40, 14);
        let x1 = cx2 - pill_w;
        let y1 = top + (self.l.row_h - pill_h) / 2;
        let k = row.knob.get().clamp(0.0, 1.0);
        c.round_rect(x1, y1, cx2, y1 + pill_h, mix(t.c(TOGGLE_OFF), t.c(TOGGLE_ON), k));
        let knob_x = x1 + 2 + (k * (pill_w - 14) as f32).round() as i32;
        c.round_rect(knob_x, y1 + 2, knob_x + 10, y1 + 12, WHITE);
        if k >= 0.5 {
            c.text("ON", x1 + 5, y1 + 3, fade(WHITE, (k - 0.5) * 2.0));
        } else {
            let w = c.width("OFF");
            c.text("OFF", cx2 - 5 - w, y1 + 3, fade(t.c(INK_SOFT), (0.5 - k) * 2.0));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_segmented(
        &self,
        c: &mut Canvas,
        row: &Setting,
        top: i32,
        cx1: i32,
        cx2: i32,
        mx: i32,
        my: i32,
        t: &Theme,
    ) {
        let h = 16;
        let y1 = top + (self.l.row_h - h) / 2;
        let count = row.choices.len().max(1) as i32;
        let seg = (cx2 - cx1) / count;
        c.fill(cx1, y1, cx2, y1 + h, t.c(CONTROL_TRACK_EDGE));
        c.fill(cx1 + 1, y1 + 1, cx2 - 1, y1 + h - 1, t.c(CONTROL_TRACK));
        if row.available && self.dropdown.is_none() && inside(mx, my, cx1, y1, cx2, y1 + h) {
            let hover_index = ((mx - cx1) / seg.max(1)).min(count - 1);
            if hover_index != row.pending {
                let hx = cx1 + hover_index * seg;
                let end = if hover_index == count - 1 { cx2 - 1 } else { hx + seg - 1 };
                c.fill(hx + 1, y1 + 1, end, y1 + h - 1, t.c(TAB_HOVER));
            }
        }
        if row.pending >= 0 {
            let position = row.knob.get().clamp(0.0, (count - 1) as f32);
            let sx = cx1 + (position * seg as f32).round() as i32;
            let ex = if row.pending == count - 1 && position > count as f32 - 1.01 {
                cx2 - 1
            } else {
                sx + seg - 1
            };
            c.fill(sx + 1, y1 + 1, ex, y1 + h - 1, t.c(CONTROL_DARK));
        }
        for (i, choice) in row.choices.iter().enumerate() {
            let x = cx1 + i as i32 * seg;
            let text = c.fit(choice, seg - 4);
            let tx = x + (seg - c.width(&text)) / 2;
            let color = if i as i32 == row.pending { WHITE } else { t.c(INK_SOFT) };
            c.text(&text, tx, y1 + 4, color);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_dropdown_header(
        &self,
        c: &mut Canvas,
        row: &Setting,
        index: usize,
        top: i32,
        cx1: i32,
        cx2: i32,
        mx: i32,
        my: i32,
        t: &Theme,
    ) {
        let h = 16;
        let y1 = top + (self.l.row_h - h) / 2;
        let over = row.available && inside(mx, my, cx1, y1, cx2, y1 + h);
        let color = if over || self.dropdown == Some(index) {
            t.c(CONTROL_DARK_HOVER)
        } else {
            t.c(CONTROL_DARK)
        };
        c.round_rect(cx1, y1, cx2, y1 + h, color);
        let text = c.fit(&row.formatted(), cx2 - cx1 - 22);
        c.text(&text, cx1 + 6, y1 + 4, WHITE);
        CHEVRON.draw(c, cx2 - 11, y1 + 7, 1, WHITE);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_slider(&self, c: &mut Canvas, row: &Setting, index: usize, top: i32, cx1: i32, cx2: i32, t: &Theme) {
        let value = row.formatted();
        let value_w = c
            .width(&value)
            .max(30)
            .max(c.width(&row.fmt.apply(row.min)))
            .max(c.width(&row.fmt.apply(row.max)));
        let (x1, x2) = (cx1, (cx2 - value_w - 6).max(cx1 + 20));
        let vw = c.width(&value);
        c.text(&value, cx2 - vw, top + (self.l.row_h - 8) / 2, t.c(INK_SOFT));
        let mid = top + self.l.row_h / 2;
        c.fill(x1, mid - 2, x2, mid + 2, t.c(CONTROL_TRACK));
        let k = ((row.knob.get() - row.min as f32) / (row.max - row.min) as f32).clamp(0.0, 1.0);
        let knob_x = x1 + (k * (x2 - x1) as f32).round() as i32;
        c.fill(x1, mid - 2, knob_x, mid + 2, t.c(CONTROL_DARK));
        let active = self.dragging == Some(index) || row.hover.get() > 0.5;
        let half = if active { 4 } else { 3 };
        c.fill(knob_x - half, mid - 6, knob_x + half, mid + 6, t.c(CONTROL_DARK));
        c.fill(knob_x - half + 1, mid - 5, knob_x + half - 1, mid + 5, WHITE);
    }

    fn draw_button_bar(&self, c: &mut Canvas, t: &Theme) {
        let l = self.l;
        let dirty = self.any_dirty();
        let text_y = l.btn_y + (l.btn_h - 8) / 2;
        let ah = self.apply_hover.get();
        c.fill(
            l.apply_x1,
            l.btn_y,
            l.apply_x2,
            l.btn_y + l.btn_h,
            if dirty { t.c(INK) } else { t.c(SURFACE_BORDER) },
        );
        c.fill(
            l.apply_x1 + 1,
            l.btn_y + 1,
            l.apply_x2 - 1,
            l.btn_y + l.btn_h - 1,
            mix(t.c(SURFACE), t.c(TAB_IDLE), if dirty { ah } else { 0.0 }),
        );
        let aw = c.width("Apply");
        c.text(
            "Apply",
            l.apply_x1 + (l.apply_x2 - l.apply_x1 - aw) / 2,
            text_y,
            if dirty { t.c(INK) } else { t.c(FAINT) },
        );
        c.round_rect(
            l.done_x1,
            l.btn_y,
            l.done_x2,
            l.btn_y + l.btn_h,
            mix(t.c(CONTROL_DARK), t.c(CONTROL_DARK_HOVER), self.done_hover.get()),
        );
        let dw = c.width("Done");
        c.text("Done", l.done_x1 + (l.done_x2 - l.done_x1 - dw) / 2, text_y, WHITE);
        let desc_x = l.px1 + 10;
        let limit = l.apply_x1 - desc_x - 8;
        let toast = self.toast.get();
        if toast > 0.03 {
            let color = fade(ACCENT, (toast * 2.0).min(1.0));
            CHECK.draw(c, desc_x, text_y + 1, 1, color);
            let text = c.fit(&self.toast_text, limit - 10);
            c.text(&text, desc_x + 10, text_y, color);
            return;
        }
        if dirty {
            let text = c.fit("Unsaved changes - press Apply or Done.", limit);
            c.text(&text, desc_x, text_y, t.c(MUTED));
        }
    }

    fn draw_footer(&self, c: &mut Canvas, t: &Theme, s: &Settings) {
        let l = self.l;
        let api = self.info_text(Info::Api, s, (0, 0, 0));
        let right = format!("Rendering API: {api}");
        let left = format!("Aetherium Mod v{VERSION} \u{b7} Horizon \u{b7} Minecraft 26.3");
        let right_w = c.width(&right).min((self.width - 2 * l.margin) / 2);
        let right_fit = c.fit(&right, right_w);
        let rw = c.width(&right_fit);
        c.text(&right_fit, self.width - l.margin - rw, l.footer_y, t.c(MUTED));
        let left_fit = c.fit(&left, self.width - 2 * l.margin - right_w - 12);
        c.text(&left_fit, l.margin, l.footer_y, t.c(MUTED));
    }

    fn draw_tooltip(&self, c: &mut Canvas, t: &Theme) {
        let a = ease_out(self.tip.get());
        let Some(index) = self.tip_shown.filter(|_| a > 0.02) else {
            return;
        };
        let Some(row) = self.rows().get(index) else {
            return;
        };
        let text = row.tooltip();
        if text.is_empty() {
            return;
        }
        let l = self.l;
        let available = self.width - 2 * l.margin;
        let box_w = available.min((l.px2 - l.px1 - 20).clamp(120, 240));
        let lines = wrap(text, box_w - 2 * TIP_PAD - 4, &|s: &str| c.width(s));
        if lines.is_empty() {
            return;
        }
        let box_h = lines.len() as i32 * TIP_LINE_H + 2 * TIP_PAD - 1;
        let top = self.row_top(index);
        let x1 = (l.px1 + 10).min(self.width - l.margin - box_w).max(l.margin);
        let mut y1 = top + l.row_h + 2;
        if y1 + box_h > self.height - l.margin {
            y1 = top - 2 - box_h;
        }
        y1 = y1.min(self.height - l.margin - box_h).max(l.margin);
        y1 += ((1.0 - a) * 4.0).round() as i32;
        let (x2, y2) = (x1 + box_w, y1 + box_h);
        let before = c.alpha;
        c.alpha = before * a;
        c.fill(x1 + 1, y1 + 3, x2 + 2, y2 + 3, t.c(SURFACE_SHADOW));
        c.fill(x1 + 2, y1 + 2, x2 + 1, y2 + 1, t.c(SURFACE_SHADOW));
        c.round_rect(x1, y1, x2, y2, mix(t.c(SURFACE_BORDER), t.c(MUTED), 0.35));
        c.fill(x1 + 1, y1 + 1, x2 - 1, y2 - 1, t.c(SURFACE));
        c.fill(
            x1 + 2,
            y1 + 3,
            x1 + 4,
            y2 - 3,
            if row.available { ACCENT } else { BADGE_OK },
        );
        let mut ty = y1 + TIP_PAD;
        for line in &lines {
            c.text(line, x1 + TIP_PAD + 3, ty, t.c(INK_SOFT));
            ty += TIP_LINE_H;
        }
        c.alpha = before;
    }

    fn dropdown_box(&self, index: usize) -> Option<[i32; 5]> {
        let row = self.rows().get(index)?;
        let top = self.row_top(index);
        let header_h = 16;
        let list_h = row.choices.len() as i32 * 14 + 2;
        let below = top + (self.l.row_h + header_h) / 2 + 1;
        let (y1, upward) = if below + list_h > self.height - 2 {
            (top + (self.l.row_h - header_h) / 2 - 1 - list_h, 1)
        } else {
            (below, 0)
        };
        Some([self.l.ctrl_x1(), y1, self.l.ctrl_x2(), y1 + list_h, upward])
    }

    fn draw_dropdown(&self, c: &mut Canvas, mx: i32, my: i32, t: &Theme) {
        let Some(index) = self.dropdown.or(self.closing) else {
            return;
        };
        let Some(row) = self.rows().get(index) else {
            return;
        };
        let open = ease_out(row.open.get());
        if open <= 0.01 {
            return;
        }
        let Some(b) = self.dropdown_box(index) else {
            return;
        };
        let item_h = 14;
        let visible = ((b[3] - b[1]) as f32 * open).round() as i32;
        let (clip_y1, clip_y2) = if b[4] == 1 {
            (b[3] - visible, b[3])
        } else {
            (b[1], b[1] + visible)
        };
        c.push_clip(b[0], clip_y1, b[2] + 2, clip_y2 + 2);
        c.fill(b[0] + 1, b[3], b[2] + 1, b[3] + 2, t.c(SURFACE_SHADOW));
        c.fill(b[0], b[1], b[2], b[3], t.c(INK));
        c.fill(b[0] + 1, b[1] + 1, b[2] - 1, b[3] - 1, t.c(SURFACE));
        for (i, choice) in row.choices.iter().enumerate() {
            let y = b[1] + 1 + i as i32 * item_h;
            if self.dropdown.is_some() && inside(mx, my, b[0], y, b[2], y + item_h) {
                c.fill(b[0] + 1, y, b[2] - 1, y + item_h, t.c(TAB_IDLE));
            }
            let chosen = i as i32 == row.pending;
            if chosen {
                CHECK.draw(c, b[0] + 5, y + 5, 1, ACCENT);
            }
            let text = c.fit(choice, b[2] - b[0] - 22);
            c.text(&text, b[0] + 15, y + 3, if chosen { t.c(INK) } else { t.c(INK_SOFT) });
        }
        c.pop_clip();
    }

    // ------------------------------------------------------------------ input

    fn select_tab(&mut self, index: usize, out: &mut Out) {
        if index == self.selected || index >= self.pages.len() {
            return;
        }
        self.selected = index;
        self.velocity = 0.0;
        self.press = None;
        self.thumb_drag = false;
        self.content.snap(0.0);
        self.content.set(1.0);
        self.scroll_target = 0.0;
        self.scroll.snap(0.0);
        self.dragging = None;
        self.dropdown = None;
        self.closing = None;
        self.hovered = None;
        self.tip_for = None;
        self.tip_shown = None;
        self.confirm_delete = false;
        out.sound = true;
    }

    fn close_dropdown(&mut self) {
        self.closing = self.dropdown.take();
    }

    /// After a row's value was staged: a preset's own options follow it, and editing one of
    /// them makes the preset Custom.
    fn edited(&mut self, index: usize) {
        let row = &self.pages[self.selected].rows[index];
        let member = row.member;
        let preset = row.bind == Bind::Hz(Hz::Preset);
        if preset {
            self.preview_preset = row.pending < 3;
        }
        if member {
            for page in &mut self.pages {
                for r in &mut page.rows {
                    if r.bind == Bind::Hz(Hz::Preset) {
                        r.stage(3);
                    }
                }
            }
        }
    }

    fn update_slider(&mut self, index: usize, x: i32) {
        let row = &mut self.pages[self.selected].rows[index];
        let span = (row.slider.1 - row.slider.0).max(1);
        let t = ((x - row.slider.0) as f32 / span as f32).clamp(0.0, 1.0);
        let value = row.min + (t * (row.max - row.min) as f32).round() as i32;
        if row.stage(value) {
            self.edited(index);
        }
    }

    fn slider_span(&self, index: usize) -> (i32, i32) {
        let Some(row) = self.rows().get(index) else {
            return (0, 1);
        };
        // The track as draw_slider lays it out, without a canvas: values are at most ~60 px.
        let (cx1, cx2) = (self.l.ctrl_x1(), self.l.ctrl_x2());
        let longest = [row.fmt.apply(row.min), row.fmt.apply(row.max), row.formatted()]
            .iter()
            .map(|s| s.chars().count() as i32 * 6)
            .max()
            .unwrap_or(0)
            .max(30);
        (cx1, (cx2 - longest - 6).max(cx1 + 20))
    }

    fn drag_thumb_to(&mut self, y: f32, immediate: bool) {
        let travel = (self.l.rows_bottom - self.l.rows_top) - self.thumb_height();
        if travel <= 0 {
            return;
        }
        let position = (y - self.thumb_grab - self.l.rows_top as f32) / travel as f32;
        self.scroll_target = position.clamp(0.0, 1.0) * self.max_scroll as f32;
        self.clamp_scroll();
        if immediate {
            self.scroll.snap(self.scroll_target);
        }
    }

    fn mouse_clicked(&mut self, x: i32, y: i32, out: &mut Out) {
        self.velocity = 0.0;
        if let Some(d) = self.dropdown {
            if let Some(b) = self.dropdown_box(d)
                && inside(x, y, b[0], b[1], b[2], b[3])
            {
                let index = (y - b[1] - 1) / 14;
                let len = self.rows()[d].choices.len() as i32;
                if (0..len).contains(&index) {
                    if self.pages[self.selected].rows[d].stage(index) {
                        self.edited(d);
                    }
                    out.sound = true;
                }
            }
            self.close_dropdown();
            return;
        }
        let [tx1, ty1, tx2, ty2] = self.theme_box;
        if tx1 >= 0 && inside(x, y, tx1, ty1, tx2, ty2) {
            self.dark = !self.dark;
            out.dark = Some(self.dark);
            out.sound = true;
            return;
        }
        let l = self.l;
        for i in 0..self.pages.len() {
            let top = self.tab_top(i);
            if inside(x, y, l.side_x, top, l.side_x + l.side_w, top + l.tab_h) {
                self.select_tab(i, out);
                return;
            }
        }
        if inside(x, y, l.apply_x1, l.btn_y, l.apply_x2, l.btn_y + l.btn_h) {
            out.apply = true;
            out.sound = true;
            return;
        }
        if inside(x, y, l.done_x1, l.btn_y, l.done_x2, l.btn_y + l.btn_h) {
            out.done = true;
            out.sound = true;
            return;
        }
        if self.max_scroll > 0 && self.over_scrollbar(x, y) {
            let (thumb_y, thumb_h) = (self.thumb_top(), self.thumb_height());
            if y >= thumb_y && y < thumb_y + thumb_h {
                self.thumb_grab = (y - thumb_y) as f32;
            } else {
                self.thumb_grab = thumb_h as f32 / 2.0;
                self.drag_thumb_to(y as f32, false);
            }
            self.thumb_drag = true;
            return;
        }
        if x >= l.px1 && x < l.px2 && y >= l.rows_top && y < l.rows_bottom {
            if let Some(i) = self.row_at(x, y) {
                let span = self.slider_span(i);
                let row = &mut self.pages[self.selected].rows[i];
                row.slider = span;
                if row.available
                    && row.kind == Kind::Slider
                    && row.editable()
                    && x >= span.0 - 6
                    && x <= span.1 + 6
                {
                    self.dragging = Some(i);
                    self.update_slider(i, x);
                    return;
                }
                // Buttons act at once: they open screens and wait for keys.
                if row.available && row.kind == Kind::Button && x >= l.ctrl_x2() - 90 && x < l.ctrl_x2() {
                    self.press_button(i, out);
                    return;
                }
            }
            self.press = Some((x, y));
            self.drag_start = (y as f32, self.scroll_target);
            self.last_drag = (y as f32, Instant::now());
        }
    }

    fn press_button(&mut self, index: usize, out: &mut Out) {
        out.sound = true;
        match self.rows()[index].bind.clone() {
            Bind::Press(Press::Reset) => {
                for page in &mut self.pages {
                    for row in &mut page.rows {
                        if let Bind::Cfg(k) = row.bind
                            && k != Cfg::Dark
                        {
                            row.stage(k.get(&Config::DEFAULT));
                        }
                    }
                }
                self.toast_text = "Defaults staged - press Apply".into();
                self.toast.snap(1.0);
            }
            Bind::Press(Press::Video) => out.video = true,
            Bind::Press(Press::AddWaypoint) => out.actions.push(Action::AddHere),
            Bind::Press(Press::DeleteAll) => {
                if self.confirm_delete {
                    self.confirm_delete = false;
                    out.actions.push(Action::DeleteAll);
                } else {
                    self.confirm_delete = true;
                }
            }
            Bind::Key(k) => out.bind = Some(k),
            _ => {}
        }
    }

    fn click_control(&mut self, index: usize, x: i32) {
        let (cx1, cx2) = (self.l.ctrl_x1(), self.l.ctrl_x2());
        let row = &mut self.pages[self.selected].rows[index];
        if !row.editable() {
            return;
        }
        match row.kind {
            Kind::Toggle => {
                let next = if row.pending == 0 { 1 } else { 0 };
                if row.stage(next) {
                    self.edited(index);
                }
            }
            Kind::Segmented if x >= cx1 && x < cx2 => {
                let count = row.choices.len().max(1) as i32;
                let pick = ((x - cx1) * count / (cx2 - cx1).max(1)).min(count - 1);
                if row.stage(pick) {
                    self.edited(index);
                }
            }
            Kind::Dropdown if x >= cx1 && x < cx2 => {
                self.dropdown = Some(index);
                self.closing = None;
            }
            _ => {}
        }
    }

    fn mouse_dragged(&mut self, x: i32, y: i32) {
        if let Some(i) = self.dragging {
            self.update_slider(i, x);
            return;
        }
        if self.thumb_drag {
            self.drag_thumb_to(y as f32, true);
            return;
        }
        if let Some((px, py)) = self.press
            && ((y - py).abs() > 4 || (x - px).abs() > 8)
        {
            self.press = None;
            self.scroll_dragging = self.max_scroll > 0;
        }
        if self.scroll_dragging {
            let yf = y as f32;
            self.scroll_target = self.drag_start.1 - (yf - self.drag_start.0);
            self.clamp_scroll();
            self.scroll.snap(self.scroll_target);
            let now = Instant::now();
            let dt = now.duration_since(self.last_drag.1).as_secs_f32();
            if dt > 0.0005 {
                let instant = (self.last_drag.0 - yf) / dt;
                self.velocity = self.velocity * 0.6 + instant * 0.4;
                self.last_drag = (yf, now);
            }
        }
    }

    fn mouse_released(&mut self, out: &mut Out) {
        if let Some((x, y)) = self.press.take() {
            if let Some(i) = self.row_at(x, y)
                && self.rows()[i].available
            {
                let kind = self.rows()[i].kind;
                self.click_control(i, x);
                if matches!(kind, Kind::Toggle | Kind::Segmented | Kind::Dropdown) {
                    out.sound = true;
                }
            }
            self.velocity = 0.0;
        } else if self.scroll_dragging {
            let stale = self.last_drag.1.elapsed().as_secs_f32() > 0.08;
            if stale || self.velocity.abs() < FLING_MIN * 2.0 {
                self.velocity = 0.0;
            }
            self.velocity = self.velocity.clamp(-4000.0, 4000.0);
        }
        self.dragging = None;
        self.scroll_dragging = false;
        self.thumb_drag = false;
    }

    fn mouse_scrolled(&mut self, amount: f32) {
        if self.dropdown.is_some() {
            return;
        }
        self.velocity = 0.0;
        self.scroll_target -= amount * self.l.row_h as f32 * 1.5;
        self.clamp_scroll();
    }

    /// Writes staged values: the preset first, then the rest. Returns how many changed.
    fn apply(&mut self, s: &mut Settings, events: &mut Vec<Event>) -> usize {
        let custom = self.preset_pending() == Some(3);
        let mut cfg = with(|sh| sh.config);
        let mut actions = Vec::new();
        let (mut count, mut hz, mut member, mut config) = (0, false, false, false);
        for preset_pass in [true, false] {
            for page in &mut self.pages {
                for row in &mut page.rows {
                    if !row.dirty() || (row.bind == Bind::Hz(Hz::Preset)) != preset_pass {
                        continue;
                    }
                    match &row.bind {
                        Bind::Hz(h) => {
                            h.set(s, row.pending);
                            hz = true;
                            member |= row.member;
                        }
                        Bind::Cfg(k) => {
                            k.set(&mut cfg, row.pending);
                            config = true;
                        }
                        Bind::Waypoint(name) => {
                            actions.push(Action::SetShown(name.clone(), row.pending != 0));
                        }
                        _ => {}
                    }
                    row.committed = row.pending;
                    count += 1;
                }
            }
        }
        if count == 0 {
            return 0;
        }
        if custom && member {
            s.graphics_preset = "custom".into();
        }
        if hz {
            events.push(Event::Changed);
        }
        with(|sh| {
            if config {
                sh.config = cfg;
                sh.config_dirty = true;
                sh.config_gen += 1;
            }
            sh.actions.extend(actions);
        });
        let ctx = Ctx {
            s,
            cfg,
            wps: &self.wps,
            in_world: true,
        };
        let mut pages = std::mem::take(&mut self.pages);
        for page in &mut pages {
            for row in &mut page.rows {
                row.load(&ctx);
            }
        }
        self.pages = pages;
        self.toast_text = if count == 1 {
            "Applied 1 change".into()
        } else {
            format!("Applied {count} changes")
        };
        self.toast.snap(1.0);
        count
    }
}

// ---------------------------------------------------------------------------------------------
// The hooks' entry points
// ---------------------------------------------------------------------------------------------

thread_local! {
    static VIEW: RefCell<Option<View>> = const { RefCell::new(None) };
    /// `list_rows` asks itself for Video's rows while adding the button.
    static LISTING: Cell<bool> = const { Cell::new(false) };
}

/// Aetherium's index in `mods.installed` while its page is the one open and it is on.
fn active(ui: &OptionsUi, s: &Settings) -> Option<usize> {
    let Page::NativeMod(i) = ui.page() else {
        return None;
    };
    let m = s.mods.installed.get(i)?;
    (m.id == ID && m.native.is_some() && s.mods.is_on(i)).then_some(i)
}

/// Where key `name` is in the mod's page (`selected_native_key`'s second number).
fn key_slot(s: &Settings, i: usize, name: &str) -> Option<usize> {
    let page = s.mods.installed.get(i)?.native.as_ref()?;
    page.keys.iter().position(|[n, ..]| n == name)
}

/// Does what an input asked of Options.
fn finish(view: &mut View, ui: &mut OptionsUi, s: &mut Settings, out: Out) -> Vec<Event> {
    let mut events = Vec::new();
    if out.sound {
        events.push(Event::ButtonClick);
    }
    if let Some(dark) = out.dark {
        with(|sh| {
            sh.config.dark = dark;
            sh.config_dirty = true;
            sh.config_gen += 1;
        });
    }
    if !out.actions.is_empty() {
        with(|sh| sh.actions.extend(out.actions));
    }
    if let Some(k) = out.bind
        && let Some(slot) = key_slot(s, view.index, KEYS[k][0])
    {
        ui.selected_key = Some(super::NATIVE_KEY);
        ui.selected_native_key = Some((view.index, slot));
    }
    if out.apply || out.done || out.video {
        view.apply(s, &mut events);
    }
    if out.done {
        events.extend(ui.back(s));
    } else if out.video {
        ui.push(Page::Video);
    }
    events
}

/// Video Settings gets an "Aetherium Settings..." button under Horizon's own two.
pub(super) fn video_rows(ui: &OptionsUi, s: &Settings) -> ControlFlow<Vec<Row>> {
    if ui.page() != Page::Video || !ui.settings_search.value.is_empty() || LISTING.with(Cell::get) {
        return ControlFlow::Continue(());
    }
    let Some(i) = s
        .mods
        .installed
        .iter()
        .position(|m| m.id == ID && m.native.is_some())
    else {
        return ControlFlow::Continue(());
    };
    if !s.mods.is_on(i) {
        return ControlFlow::Continue(());
    }
    LISTING.with(|l| l.set(true));
    let mut rows = ui.list_rows(s);
    LISTING.with(|l| l.set(false));
    let button = super::button(
        0.0,
        0.0,
        310.0,
        Text::literal("Aetherium Settings..."),
        Act::ModSettings(i),
    );
    rows.insert(rows.len().min(1), Row::Buttons(button, None));
    ControlFlow::Break(rows)
}

/// Draws Aetherium's screen instead of the stock page.
pub(super) fn draw(
    ui: &mut OptionsUi,
    s: &Settings,
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    cursor: (f32, f32),
) -> ControlFlow<()> {
    let Some(i) = active(ui, s) else {
        return ControlFlow::Continue(());
    };
    let (cfg, fps, in_world, gen, tab) = with(|sh| {
        (
            sh.config,
            sh.fps,
            sh.in_world,
            sh.waypoints_gen,
            sh.open_tab,
        )
    });
    VIEW.with(|cell| {
        let mut cell = cell.borrow_mut();
        let now = Instant::now();
        let stale = cell
            .as_ref()
            .is_none_or(|v| v.index != i || now.duration_since(v.last).as_secs_f32() > 0.25);
        let wps = if stale || cell.as_ref().is_some_and(|v| v.wp_gen != gen) {
            with(|sh| sh.waypoints.clone())
        } else {
            Vec::new()
        };
        if stale {
            with(|sh| sh.open_tab = None);
            let ctx = Ctx {
                s,
                cfg,
                wps: &wps,
                in_world,
            };
            *cell = Some(View::new(i, &ctx, tab.unwrap_or(0), gen));
        }
        let Some(view) = cell.as_mut() else {
            return;
        };
        let dt = now.duration_since(view.last).as_secs_f32().min(0.25);
        view.last = now;
        view.width = batch.width.max(1.0) as i32;
        view.height = batch.height.max(1.0) as i32;
        let current = if wps.is_empty() && view.wp_gen == gen {
            std::mem::take(&mut view.wps)
        } else {
            wps
        };
        let ctx = Ctx {
            s,
            cfg,
            wps: &current,
            in_world,
        };
        view.sync(&ctx, gen, ui);
        view.wps = current;
        let mut c = Canvas {
            batch,
            atlas,
            dx: 0,
            dy: 0,
            alpha: 1.0,
            clips: Vec::new(),
        };
        view.render(&mut c, cursor.0.floor() as i32, cursor.1.floor() as i32, dt, s, fps);
        while !c.clips.is_empty() {
            c.pop_clip();
        }
    });
    ControlFlow::Break(())
}

/// Runs `f` on the open view of Aetherium's page, else lets Options handle the input.
fn on_view(
    ui: &mut OptionsUi,
    s: &mut Settings,
    f: impl FnOnce(&mut View, &mut OptionsUi, &mut Settings) -> Vec<Event>,
) -> ControlFlow<Vec<Event>> {
    let Some(i) = active(ui, s) else {
        return ControlFlow::Continue(());
    };
    VIEW.with(|cell| {
        let mut cell = cell.borrow_mut();
        match cell.as_mut().filter(|v| v.index == i) {
            Some(view) => ControlFlow::Break(f(view, ui, s)),
            None => ControlFlow::Break(Vec::new()),
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn click(
    ui: &mut OptionsUi,
    s: &mut Settings,
    x: f32,
    y: f32,
    drag: bool,
    right: bool,
) -> ControlFlow<Vec<Event>> {
    if ui.selected_key.is_some() {
        // Waiting for a key: Options binds the mouse button, as on Key Binds.
        return if drag && active(ui, s).is_some() {
            ControlFlow::Break(Vec::new())
        } else {
            ControlFlow::Continue(())
        };
    }
    on_view(ui, s, |view, ui, s| {
        let (x, y) = (x.floor() as i32, y.floor() as i32);
        let mut out = Out::default();
        if drag {
            view.mouse_dragged(x, y);
        } else if !right {
            view.mouse_clicked(x, y, &mut out);
        }
        finish(view, ui, s, out)
    })
}

pub(super) fn release(ui: &mut OptionsUi) -> ControlFlow<Vec<Event>> {
    let Page::NativeMod(i) = ui.page() else {
        return ControlFlow::Continue(());
    };
    if ui.selected_key.is_some() {
        return ControlFlow::Continue(());
    }
    VIEW.with(|cell| {
        let mut cell = cell.borrow_mut();
        match cell
            .as_mut()
            .filter(|v| v.index == i && v.last.elapsed().as_secs_f32() <= 0.25)
        {
            Some(view) => {
                let mut out = Out::default();
                view.mouse_released(&mut out);
                ControlFlow::Break(if out.sound {
                    vec![Event::ButtonClick]
                } else {
                    Vec::new()
                })
            }
            None => ControlFlow::Continue(()),
        }
    })
}

pub(super) fn wheel(ui: &mut OptionsUi, s: &mut Settings, amount: f32) -> ControlFlow<Vec<Event>> {
    on_view(ui, s, |view, _, _| {
        view.mouse_scrolled(amount);
        Vec::new()
    })
}

pub(super) fn key(
    ui: &mut OptionsUi,
    s: &mut Settings,
    key: winit::keyboard::KeyCode,
) -> ControlFlow<Vec<Event>> {
    use winit::keyboard::KeyCode as K;
    if ui.selected_key.is_some() {
        return ControlFlow::Continue(());
    }
    on_view(ui, s, |view, ui, s| {
        let mut out = Out::default();
        match key {
            K::Escape => {
                if view.dropdown.is_some() {
                    view.close_dropdown();
                    return Vec::new();
                }
                out.done = true;
            }
            K::Enter | K::NumpadEnter => out.apply = true,
            K::ArrowDown | K::PageDown => {
                let next = (view.selected + 1).min(view.pages.len() - 1);
                view.select_tab(next, &mut out);
            }
            K::ArrowUp | K::PageUp => {
                let next = view.selected.saturating_sub(1);
                view.select_tab(next, &mut out);
            }
            _ => return Vec::new(),
        }
        finish(view, ui, s, out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip_and_bad_lines() {
        let mut c = Config::DEFAULT;
        c.hud_corner = 3;
        c.zoom_default = 70;
        c.dark = true;
        let back = Config::from_text(&format!("{}junk\nzoom_mouse=x\nhud_corner=9\n", c.to_text()));
        assert_eq!(back.zoom_default, 70);
        assert!(back.dark);
        assert_eq!(back.hud_corner, 3);
        assert_eq!(back.zoom_mouse, Config::DEFAULT.zoom_mouse);
    }

    #[test]
    fn colours_swap_red_and_blue() {
        assert_eq!(abgr(0xFF11_2233), 0xFF33_2211);
        assert_eq!(mix(0xFF00_0000, 0xFFFF_FFFF, 1.0), 0xFFFF_FFFF);
        assert_eq!(fade(0xFF12_3456, 0.0), 0x0012_3456);
    }

    #[test]
    fn wrap_breaks_long_words() {
        let measure = |s: &str| s.chars().count() as i32 * 6;
        let lines = wrap("aaa bbbbbbbbbbbbbbbbbbbb c", 60, &measure);
        assert!(lines.iter().all(|l| measure(l) <= 60));
        assert_eq!(lines.concat().replace(' ', ""), "aaabbbbbbbbbbbbbbbbbbbbc");
    }
}
