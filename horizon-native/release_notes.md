**Aetherium 1.0.0, native mod for Horizon** (Minecraft Java 26.3 rewritten in Rust), **desktop only** (Windows / Linux / macOS Horizon launcher).

This is Aetherium's real settings screen inside Horizon: the AETHERIUM header, sidebar tabs, animated toggles, sliders, segmented buttons and dropdowns, hover popups, light/dark switch, Apply and Done.

### Install
1. In the Horizon launcher, go to **Mods → Add from GitHub** and paste this release's link,
   or run `horizon-launcher mods add-github https://github.com/entitybrian69-bit/aetherium/releases/tag/hnmod-v1.0.0%2B26.3 --trust-native-code`.
2. Press **Build**. The launcher builds Horizon with the mod, on the Horizon commit it was made for.
3. If you also have the sandboxed `aetherium` .hmod installed, remove it first: both use the id `aetherium`.

### Where to find it
- **Options → Video Settings → Aetherium Settings...**
- **N** while playing, or `/aetherium`. `/wp` opens the Waypoints tab.

### Tabs
- **General**: info overlay (FPS, 1% low/min, coordinates, facing, Nether/Overworld coordinates, corner), reset, Horizon's Video Settings.
- **Quality**: graphics preset, clouds, particles, smooth lighting, entity shadows, biome blend, mipmaps, brightness, cutout leaves, improved transparency, vignette, view bobbing, FOV.
- **Performance**: render and simulation distance, max frame rate, VSync, entity distance, chunk builder, weather radius, menu blur, cloud range.
- **Zoom**: hold or toggle, default zoom, smoothness, wheel zoom, mouse speed while zoomed, keys.
- **Waypoints**: add here, death waypoint, on-screen markers, nearest waypoints in the overlay, show/hide each, delete all, keys.
- **Backend**: graphics API, GUI size, frame rate, render distance, native mods.

### Default keys (change them on the Zoom and Waypoints tabs, or in Options → Mods)
| Key | Action |
|---|---|
| C | Zoom (hold) |
| = / - | Zoom in / out while zoomed (the wheel works too) |
| B | Add a waypoint where you stand |
| N | Open Aetherium |

Settings and waypoints are kept in `horizon-mods/.horizon-mod-data/aetherium/`. Waypoints made with the .hmod carry over.
