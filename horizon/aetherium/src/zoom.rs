//! Aetherium's zoom, ported from the Java mod's `ZoomController` (which
//! follows Zume): a zoom *amount* from 1 (none) to 0 (strongest), eased with
//! `1 - (1 - t)^4`, turned into a field of view as
//! `minFov + (fov - minFov) * amount^2`.
//!
//! Horizon takes a zoom as a factor the field of view is divided by (at most
//! 10) and a mouse-speed multiplier, so the amount becomes
//! `fov / zoomed_fov`. Third-person zoom and the cinematic camera are not
//! offered by Horizon's mod interface, so this port is first-person only and
//! slows the mouse instead.

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

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn hold_zooms_to_default_and_back() {
        let mut z = Zoom::new(ZoomConfig::default());
        z.frame(0.016, false, false, false);
        assert_eq!(z.view(), View::NONE);
        z.frame(0.016, true, false, false);
        assert!(z.is_zooming());
        // Long enough for the 150 ms ease to finish.
        for _ in 0..20 {
            z.frame(0.016, true, false, false);
        }
        assert!(close(z.amount(), 0.5));
        // FOV 70: 1 + 69 * 0.25 = 18.25, factor 70 / 18.25.
        let v = z.view();
        assert!(close(v.factor, 70.0 / 18.25), "{v:?}");
        assert!(close(v.turn, 0.4 + 0.6 * 0.5));
        z.frame(0.016, false, false, false);
        assert!(!z.is_zooming());
        assert!(z.is_visible(), "eases out");
        for _ in 0..20 {
            z.frame(0.016, false, false, false);
        }
        assert!(!z.is_visible());
        assert_eq!(z.view(), View::NONE);
    }

    #[test]
    fn toggle_mode_flips_on_press() {
        let cfg = ZoomConfig {
            toggle: true,
            smooth_s: 0.0,
            ..ZoomConfig::default()
        };
        let mut z = Zoom::new(cfg);
        z.frame(0.016, true, false, false);
        assert!(z.is_zooming());
        z.frame(0.016, false, false, false);
        assert!(z.is_zooming(), "release keeps a toggled zoom");
        z.frame(0.016, true, false, false);
        assert!(!z.is_zooming());
    }

    #[test]
    fn easing_is_zumes_quartic() {
        let mut z = Zoom::new(ZoomConfig::default());
        z.frame(0.0, true, false, false);
        // Half way through the 150 ms ease: 1 - (1 - 0.5)^4 = 0.9375 of the way.
        z.frame(0.075, true, false, false);
        assert!(close(z.amount(), 1.0 - 0.5 * 0.9375), "{}", z.amount());
    }

    #[test]
    fn keys_and_wheel_change_the_zoom_within_limits() {
        let cfg = ZoomConfig {
            smooth_s: 0.0,
            ..ZoomConfig::default()
        };
        let mut z = Zoom::new(cfg);
        z.frame(0.016, true, false, false);
        assert!(close(z.amount(), 0.5));
        // Zoom In held 0.1 s: 20 * 0.0001 * 100 = 0.2 closer.
        z.frame(0.1, true, true, false);
        assert!(close(z.amount(), 0.3));
        assert!(z.scroll(1.0));
        z.frame(0.016, true, false, false);
        assert!(close(z.amount(), 0.3 - 0.08));
        // Far past the strongest zoom: the factor stops at Horizon's 10.
        z.frame(0.25, true, true, false);
        z.frame(0.25, true, true, false);
        assert!(close(z.amount(), 0.0));
        assert!(close(z.view().factor, MAX_FACTOR));
        // Zoom Out all the way: no zoom left, but still zooming.
        for _ in 0..10 {
            z.frame(0.25, true, false, true);
        }
        assert!(close(z.amount(), 1.0));
        assert!(close(z.view().factor, 1.0));
        assert!(!z.scroll(0.0));
    }

    #[test]
    fn wheel_ignored_when_off_or_not_zooming() {
        let mut z = Zoom::new(ZoomConfig::default());
        assert!(!z.scroll(1.0));
        z.config.scrolling = false;
        z.frame(0.016, true, false, false);
        assert!(!z.scroll(1.0));
        assert!(!z.wants_wheel());
    }

    #[test]
    fn hitches_and_bad_frames_are_bounded() {
        let mut z = Zoom::new(ZoomConfig::default());
        z.frame(f64::NAN, true, false, false);
        z.frame(100.0, true, false, false);
        assert!(close(z.amount(), 0.5));
        assert!(z.view().factor.is_finite());
    }
}
