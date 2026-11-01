//! Waypoints: named block positions per dimension, kept in the mod's storage
//! one per line, with the maths to show them: Nether/Overworld conversion,
//! distance, a clock direction and where a point falls on the screen.

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

#[cfg(test)]
mod tests {
    use super::*;

    fn wp(name: &str, dim: &str, x: i32, z: i32) -> Waypoint {
        Waypoint {
            name: name.into(),
            dimension: dim.into(),
            x,
            y: 64,
            z,
            shown: true,
        }
    }

    #[test]
    fn storage_round_trip_and_bad_lines() {
        let mut w = Waypoints::default();
        w.set(wp("Home", OVERWORLD, 10, -20)).unwrap();
        w.set(wp("Fort\tress", NETHER, 1, 2)).unwrap();
        let text = w.to_text();
        let back = Waypoints::from_text(&format!("{text}junk\nA\tB\tx\t1\t2\t1\n"));
        assert_eq!(back.list.len(), 2);
        assert_eq!(back.list[0], w.list[0]);
        assert_eq!(back.list[1].name, "Fort ress");
    }

    #[test]
    fn find_by_name_or_number_and_replace() {
        let mut w = Waypoints::default();
        w.set(wp("Home", OVERWORLD, 0, 0)).unwrap();
        w.set(wp("Mine", OVERWORLD, 5, 5)).unwrap();
        assert_eq!(w.find("home"), Some(0));
        assert_eq!(w.find("#2"), Some(1));
        assert_eq!(w.find("2"), Some(1));
        assert_eq!(w.find("3"), None);
        w.set(wp("HOME", OVERWORLD, 9, 9)).unwrap();
        assert_eq!(w.list.len(), 2);
        assert_eq!(w.list[0].x, 9);
        assert_eq!(w.next_name(), "Waypoint 1");
    }

    #[test]
    fn full_list_refuses() {
        let mut w = Waypoints::default();
        for i in 0..MAX_WAYPOINTS {
            w.set(wp(&format!("w{i}"), OVERWORLD, 0, 0)).unwrap();
        }
        assert!(w.set(wp("one more", OVERWORLD, 0, 0)).is_err());
    }

    #[test]
    fn nether_conversion() {
        assert_eq!(convert(OVERWORLD, 80.0, -16.0, NETHER), Some((10.0, -2.0)));
        assert_eq!(convert(NETHER, 10.0, -2.0, OVERWORLD), Some((80.0, -16.0)));
        assert_eq!(convert(OVERWORLD, 1.0, 1.0, END), None);
        assert_eq!(convert(END, 1.0, 1.0, END), Some((1.0, 1.0)));
    }

    #[test]
    fn directions_follow_minecraft_yaw() {
        // South is +Z at yaw 0, west is -X at yaw 90.
        assert!((yaw_to(0.0, 0.0, 0.0, 10.0) - 0.0).abs() < 1e-9);
        assert!((yaw_to(0.0, 0.0, -10.0, 0.0) - 90.0).abs() < 1e-9);
        assert!((wrap(yaw_to(0.0, 0.0, 10.0, 0.0)) + 90.0).abs() < 1e-9);
        assert_eq!(clock(0.0), 12);
        assert_eq!(clock(90.0), 3);
        assert_eq!(clock(-90.0), 9);
        assert_eq!(clock(180.0), 6);
        assert_eq!(clock(-179.0), 6);
        assert_eq!(facing(0.0), "South");
        assert_eq!(facing(90.0), "West");
        assert_eq!(facing(-180.0), "North");
        assert_eq!(facing(-90.0), "East");
        assert_eq!(facing(314.0), "East");
    }

    #[test]
    fn projection_centre_right_and_behind() {
        let cam = Camera {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            fov: 90.0,
            width: 400.0,
            height: 200.0,
        };
        // Straight ahead (south): the centre.
        let p = cam.project(0.0, 0.0, 10.0, 8.0).unwrap();
        assert!((p.x - 200.0).abs() < 1e-6 && (p.y - 100.0).abs() < 1e-6 && !p.edge);
        // Facing south, west (-X) is on the right. At 45 degrees right with a
        // 90 degree vertical FOV: tan 45 = 1 → half the height right of centre.
        let p = cam.project(-10.0, 0.0, 10.0, 8.0).unwrap();
        assert!((p.x - 300.0).abs() < 1e-6, "{p:?}");
        // Up is up on screen.
        let p = cam.project(0.0, 5.0, 10.0, 8.0).unwrap();
        assert!(p.y < 100.0);
        // Behind and to the left (east, +X): pushed to the left edge.
        let p = cam.project(10.0, 0.0, -10.0, 8.0).unwrap();
        assert!(p.edge && (p.x - 8.0).abs() < 1e-6, "{p:?}");
        // Straight behind: the bottom edge.
        let p = cam.project(0.0, 0.0, -10.0, 8.0).unwrap();
        assert!(p.edge && (p.y - 192.0).abs() < 1e-6, "{p:?}");
        // No screen yet: nothing.
        let none = Camera { width: 0.0, ..cam };
        assert!(none.project(0.0, 0.0, 10.0, 8.0).is_none());
    }

    #[test]
    fn projection_looking_down() {
        let cam = Camera {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 90.0,
            fov: 70.0,
            width: 400.0,
            height: 200.0,
        };
        // Pitch 90 looks straight down: the point below is the centre.
        let p = cam.project(0.0, -10.0, 0.0, 8.0).unwrap();
        assert!((p.x - 200.0).abs() < 1e-6 && (p.y - 100.0).abs() < 1e-6, "{p:?}");
    }

    #[test]
    fn names_are_cleaned() {
        assert_eq!(clean_name("  a\tb\nc  "), "a b c");
        assert_eq!(clean_name(&"x".repeat(50)).len(), MAX_NAME);
        assert_eq!(dimension_name("mymod:deep_dark"), "deep dark");
    }
}
