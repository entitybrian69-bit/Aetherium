### What it is
A native Horizon port of Aetherium, written in Rust and built as a sandboxed `.hmod` (a WebAssembly component). It runs on Horizon Android 0.7.6 (64-bit APKs: arm64-v8a or x86_64) and on desktop Horizon builds with the same mod interface. **It is not a Java mod.** Fabric, Forge and NeoForge jars don't run on Horizon.

### Features
- **Zoom**, ported from Aetherium's Java zoom (Zume's curve and easing):
  - Zoom key: hold, or toggle.
  - Zoom In / Zoom Out keys, plus the mouse wheel while zooming.
  - A slower mouse while zoomed.
  - Starting zoom and animation time can be changed.
- **Info HUD**:
  - FPS with the 1% low and minimum, colour coded.
  - Coordinates and facing.
  - Nether/Overworld coordinates.
  - Any corner of the screen.
- **Waypoints**:
  - Add one with a key, `/wp add <name>` or the Waypoints screen.
  - Markers on screen with name and distance; off-screen ones stick to the edge.
  - Nearest waypoints in the HUD with distance and a clock direction.
  - Overworld waypoints show at 1/8 in the Nether (and the reverse).
  - Automatic **death point**.
- **Settings** in Options > Mods > Aetherium (keys are listed there and can be rebound), plus the `/aetherium` screen and `/aetherium set <option> <value>`.

### Install on Horizon Android
1. Open the launcher's **Mods** tab, paste the download link into the search field and press **Import .hmod**. You can also download the file and use **Import file**.
2. Enable it in **Installed**, then check its capabilities in the game's **Mods** menu.
3. On a touchscreen, add custom keypad buttons for the keys Z (zoom), = and - (zoom in and out), B (add waypoint) and N (waypoints screen), or use the commands.

### Keys (rebindable)
| Key | Action |
|---|---|
| Z | Zoom |
| = | Zoom in |
| - | Zoom out |
| B | Add waypoint |
| N | Waypoints screen |

If Horizon's own **Zoom** mod is also on, both use Z. Turn one off or rebind it.

### Not portable to Horizon
Horizon is its own Rust engine with its own renderer. Its sandboxed mod interface gives no access to rendering, the camera's fog and sky, particles, entities, the hand or inventories. These Aetherium features therefore can't exist here:
- The renderer and the performance features.
- Fog, sky, particle and scenery options.
- Freelook, the hand editor, Mouse Tweaks and sorting.
- The crosshair, damage and saturation features.
- The themed settings GUI. Horizon draws mod screens with its own widgets.

Horizon already ships graphics presets, distant terrain and a minimap.

### Limits
- Horizon doesn't tell mods your FOV. Set **Your FOV setting** in Aetherium's options to match yours; zoom strength and marker placement use it. Markers drift slightly while sprinting changes the FOV.
- Waypoints are saved per dimension, not per world or server, because Horizon doesn't tell mods which world is open.
