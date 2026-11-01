//! Aetherium's options, kept in the mod's storage as `key=value` lines.
//!
//! Every option is a cycle of a few choices, the way Horizon's own option
//! buttons work: a click moves to the next one. Unknown or malformed lines
//! are ignored, so an older or newer settings file never stops the mod.

/// Where a panel sits on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
    fn parse(s: &str) -> Option<Corner> {
        Corner::ALL.into_iter().find(|c| c.key() == s)
    }
}

/// All options. Defaults follow Aetherium's Java defaults where the option
/// exists there (zoom: Zume's numbers).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    // Zoom
    pub zoom_toggle: bool,
    /// Percent of the way to the strongest zoom a fresh zoom starts at.
    pub zoom_default: u32,
    /// Ease-in/out time in milliseconds (0: instant).
    pub zoom_smooth_ms: u32,
    pub zoom_scroll: bool,
    /// Mouse speed at full zoom, percent of normal.
    pub zoom_mouse: u32,
    /// The player's field of view, which Horizon doesn't tell mods: used for
    /// Zume's zoom curve and for placing waypoint markers.
    pub fov: u32,
    // HUD
    pub hud: bool,
    pub hud_corner: Corner,
    pub hud_fps: bool,
    pub hud_lows: bool,
    pub hud_coords: bool,
    pub hud_other_dim: bool,
    pub hud_facing: bool,
    // Waypoints
    pub wp_markers: bool,
    pub wp_hud: bool,
    pub wp_death: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            zoom_toggle: false,
            zoom_default: 50,
            zoom_smooth_ms: 150,
            zoom_scroll: true,
            zoom_mouse: 40,
            fov: 70,
            hud: true,
            hud_corner: Corner::TopLeft,
            hud_fps: true,
            hud_lows: true,
            hud_coords: true,
            hud_other_dim: true,
            hud_facing: true,
            wp_markers: true,
            wp_hud: true,
            wp_death: true,
        }
    }
}

pub const ZOOM_DEFAULTS: [u32; 5] = [25, 50, 65, 75, 90];
pub const ZOOM_SMOOTH: [u32; 4] = [0, 100, 150, 300];
pub const ZOOM_MOUSE: [u32; 5] = [20, 40, 60, 80, 100];
pub const FOVS: [u32; 9] = [30, 50, 60, 70, 80, 90, 100, 110, 120];

/// One option the player can change: its storage key and button label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opt {
    ZoomToggle,
    ZoomDefault,
    ZoomSmooth,
    ZoomScroll,
    ZoomMouse,
    Fov,
    Hud,
    HudCorner,
    HudFps,
    HudLows,
    HudCoords,
    HudOtherDim,
    HudFacing,
    WpMarkers,
    WpHud,
    WpDeath,
}

impl Opt {
    pub const ALL: [Opt; 16] = [
        Opt::ZoomToggle,
        Opt::ZoomDefault,
        Opt::ZoomSmooth,
        Opt::ZoomScroll,
        Opt::ZoomMouse,
        Opt::Fov,
        Opt::Hud,
        Opt::HudCorner,
        Opt::HudFps,
        Opt::HudLows,
        Opt::HudCoords,
        Opt::HudOtherDim,
        Opt::HudFacing,
        Opt::WpMarkers,
        Opt::WpHud,
        Opt::WpDeath,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Opt::ZoomToggle => "zoom_toggle",
            Opt::ZoomDefault => "zoom_default",
            Opt::ZoomSmooth => "zoom_smooth",
            Opt::ZoomScroll => "zoom_scroll",
            Opt::ZoomMouse => "zoom_mouse",
            Opt::Fov => "fov",
            Opt::Hud => "hud",
            Opt::HudCorner => "hud_corner",
            Opt::HudFps => "hud_fps",
            Opt::HudLows => "hud_lows",
            Opt::HudCoords => "hud_coords",
            Opt::HudOtherDim => "hud_other_dim",
            Opt::HudFacing => "hud_facing",
            Opt::WpMarkers => "wp_markers",
            Opt::WpHud => "wp_hud",
            Opt::WpDeath => "wp_death",
        }
    }
    pub fn from_key(key: &str) -> Option<Opt> {
        Opt::ALL.into_iter().find(|o| o.key() == key)
    }
    /// The section heading the option sits under on the settings page.
    pub fn section(self) -> &'static str {
        match self {
            Opt::ZoomToggle
            | Opt::ZoomDefault
            | Opt::ZoomSmooth
            | Opt::ZoomScroll
            | Opt::ZoomMouse
            | Opt::Fov => "Zoom",
            Opt::Hud
            | Opt::HudCorner
            | Opt::HudFps
            | Opt::HudLows
            | Opt::HudCoords
            | Opt::HudOtherDim
            | Opt::HudFacing => "HUD",
            Opt::WpMarkers | Opt::WpHud | Opt::WpDeath => "Waypoints",
        }
    }
}

fn on(b: bool) -> &'static str {
    if b { "ON" } else { "OFF" }
}

/// The next value of a cycle; a value not in it starts the cycle over.
fn next_in(cycle: &[u32], v: u32) -> u32 {
    match cycle.iter().position(|&c| c == v) {
        Some(i) => cycle[(i + 1) % cycle.len()],
        None => cycle[0],
    }
}

impl Settings {
    /// The button text for `opt`.
    pub fn label(&self, opt: Opt) -> String {
        match opt {
            Opt::ZoomToggle => format!(
                "Zoom key: {}",
                if self.zoom_toggle { "Toggle" } else { "Hold" }
            ),
            Opt::ZoomDefault => format!("Starting zoom: {}%", self.zoom_default),
            Opt::ZoomSmooth => {
                if self.zoom_smooth_ms == 0 {
                    "Zoom animation: OFF".to_string()
                } else {
                    format!("Zoom animation: {} ms", self.zoom_smooth_ms)
                }
            }
            Opt::ZoomScroll => format!("Mouse wheel zoom: {}", on(self.zoom_scroll)),
            Opt::ZoomMouse => format!("Mouse speed when zoomed: {}%", self.zoom_mouse),
            Opt::Fov => format!("Your FOV setting: {}", self.fov),
            Opt::Hud => format!("Info HUD: {}", on(self.hud)),
            Opt::HudCorner => format!("HUD corner: {}", self.hud_corner.label()),
            Opt::HudFps => format!("FPS: {}", on(self.hud_fps)),
            Opt::HudLows => format!("1% low and minimum FPS: {}", on(self.hud_lows)),
            Opt::HudCoords => format!("Coordinates: {}", on(self.hud_coords)),
            Opt::HudOtherDim => format!("Nether/Overworld coordinates: {}", on(self.hud_other_dim)),
            Opt::HudFacing => format!("Facing direction: {}", on(self.hud_facing)),
            Opt::WpMarkers => format!("Waypoint markers on screen: {}", on(self.wp_markers)),
            Opt::WpHud => format!("Waypoints in the HUD: {}", on(self.wp_hud)),
            Opt::WpDeath => format!("Save death point: {}", on(self.wp_death)),
        }
    }

    /// Moves `opt` to its next choice.
    pub fn cycle(&mut self, opt: Opt) {
        match opt {
            Opt::ZoomToggle => self.zoom_toggle = !self.zoom_toggle,
            Opt::ZoomDefault => self.zoom_default = next_in(&ZOOM_DEFAULTS, self.zoom_default),
            Opt::ZoomSmooth => self.zoom_smooth_ms = next_in(&ZOOM_SMOOTH, self.zoom_smooth_ms),
            Opt::ZoomScroll => self.zoom_scroll = !self.zoom_scroll,
            Opt::ZoomMouse => self.zoom_mouse = next_in(&ZOOM_MOUSE, self.zoom_mouse),
            Opt::Fov => self.fov = next_in(&FOVS, self.fov),
            Opt::Hud => self.hud = !self.hud,
            Opt::HudCorner => {
                let i = Corner::ALL
                    .iter()
                    .position(|&c| c == self.hud_corner)
                    .unwrap_or(0);
                self.hud_corner = Corner::ALL[(i + 1) % Corner::ALL.len()];
            }
            Opt::HudFps => self.hud_fps = !self.hud_fps,
            Opt::HudLows => self.hud_lows = !self.hud_lows,
            Opt::HudCoords => self.hud_coords = !self.hud_coords,
            Opt::HudOtherDim => self.hud_other_dim = !self.hud_other_dim,
            Opt::HudFacing => self.hud_facing = !self.hud_facing,
            Opt::WpMarkers => self.wp_markers = !self.wp_markers,
            Opt::WpHud => self.wp_hud = !self.wp_hud,
            Opt::WpDeath => self.wp_death = !self.wp_death,
        }
    }

    /// `opt`'s value as stored.
    pub fn value(&self, opt: Opt) -> String {
        let b = |v: bool| if v { "true" } else { "false" }.to_string();
        match opt {
            Opt::ZoomToggle => b(self.zoom_toggle),
            Opt::ZoomDefault => self.zoom_default.to_string(),
            Opt::ZoomSmooth => self.zoom_smooth_ms.to_string(),
            Opt::ZoomScroll => b(self.zoom_scroll),
            Opt::ZoomMouse => self.zoom_mouse.to_string(),
            Opt::Fov => self.fov.to_string(),
            Opt::Hud => b(self.hud),
            Opt::HudCorner => self.hud_corner.key().to_string(),
            Opt::HudFps => b(self.hud_fps),
            Opt::HudLows => b(self.hud_lows),
            Opt::HudCoords => b(self.hud_coords),
            Opt::HudOtherDim => b(self.hud_other_dim),
            Opt::HudFacing => b(self.hud_facing),
            Opt::WpMarkers => b(self.wp_markers),
            Opt::WpHud => b(self.wp_hud),
            Opt::WpDeath => b(self.wp_death),
        }
    }

    /// Sets `opt` from text (`true`/`on`, a number, a corner key); errs on a
    /// value the option doesn't take.
    pub fn set(&mut self, opt: Opt, text: &str) -> Result<(), String> {
        let text = text.trim().to_ascii_lowercase();
        let flag = || match text.as_str() {
            "true" | "on" | "yes" | "1" => Ok(true),
            "false" | "off" | "no" | "0" => Ok(false),
            _ => Err(format!("{} takes on or off", opt.key())),
        };
        let num = |lo: u32, hi: u32| {
            text.parse::<u32>()
                .ok()
                .filter(|v| (lo..=hi).contains(v))
                .ok_or_else(|| format!("{} takes a number from {lo} to {hi}", opt.key()))
        };
        match opt {
            Opt::ZoomToggle => self.zoom_toggle = flag()?,
            Opt::ZoomDefault => self.zoom_default = num(5, 100)?,
            Opt::ZoomSmooth => self.zoom_smooth_ms = num(0, 1000)?,
            Opt::ZoomScroll => self.zoom_scroll = flag()?,
            Opt::ZoomMouse => self.zoom_mouse = num(5, 100)?,
            Opt::Fov => self.fov = num(30, 120)?,
            Opt::Hud => self.hud = flag()?,
            Opt::HudCorner => {
                self.hud_corner = Corner::parse(&text).ok_or_else(|| {
                    "hud_corner takes top_left, top_right, bottom_left or bottom_right".to_string()
                })?
            }
            Opt::HudFps => self.hud_fps = flag()?,
            Opt::HudLows => self.hud_lows = flag()?,
            Opt::HudCoords => self.hud_coords = flag()?,
            Opt::HudOtherDim => self.hud_other_dim = flag()?,
            Opt::HudFacing => self.hud_facing = flag()?,
            Opt::WpMarkers => self.wp_markers = flag()?,
            Opt::WpHud => self.wp_hud = flag()?,
            Opt::WpDeath => self.wp_death = flag()?,
        }
        Ok(())
    }

    /// The storage file: one `key=value` line per option.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for opt in Opt::ALL {
            out.push_str(opt.key());
            out.push('=');
            out.push_str(&self.value(opt));
            out.push('\n');
        }
        out
    }

    /// Reads a storage file; anything it doesn't understand keeps its default.
    pub fn from_text(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            if let Some(opt) = Opt::from_key(k.trim()) {
                let _ = s.set(opt, v);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_text() {
        let mut s = Settings::default();
        for opt in Opt::ALL {
            s.cycle(opt);
        }
        assert_ne!(s, Settings::default());
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }

    #[test]
    fn bad_lines_keep_defaults() {
        let s = Settings::from_text("fov=999\nzoom_default=abc\nnonsense\nhud=off\nwho=1");
        assert_eq!(s.fov, 70);
        assert_eq!(s.zoom_default, 50);
        assert!(!s.hud);
    }

    #[test]
    fn cycles_wrap_and_recover() {
        let mut s = Settings::default();
        s.zoom_default = 33;
        s.cycle(Opt::ZoomDefault);
        assert_eq!(s.zoom_default, ZOOM_DEFAULTS[0]);
        s.fov = 120;
        s.cycle(Opt::Fov);
        assert_eq!(s.fov, 30);
        s.hud_corner = Corner::BottomRight;
        s.cycle(Opt::HudCorner);
        assert_eq!(s.hud_corner, Corner::TopLeft);
    }

    #[test]
    fn every_key_is_unique_and_found() {
        for opt in Opt::ALL {
            assert_eq!(Opt::from_key(opt.key()), Some(opt));
            assert!(!s_label(opt).is_empty());
        }
        fn s_label(o: Opt) -> String {
            Settings::default().label(o)
        }
    }
}
