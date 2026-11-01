# Aetherium for Horizon (Minecraft Java 26.3 in Rust)

`aetherium/` is a Rust mod for [Horizon](https://github.com/horizon-rust/horizon), the Rust rewrite of Minecraft Java 26.3. It is built as a sandboxed `.hmod` (WebAssembly component) for [Vasysik/horizon-android](https://github.com/Vasysik/horizon-android).

- `aetherium/src/zoom.rs`: Aetherium's Java `ZoomController` ported to Rust (first person).
- `aetherium/src/waypoints.rs`: waypoints, Nether conversion and screen projection.
- `aetherium/src/settings.rs`: options and their storage.
- `aetherium/src/lib.rs`: the Horizon mod (HUD, markers, keys, screens, commands).
- `aetherium/tests/*.toml`: scenario tests run by `cargo horizon-mod test` in Horizon's own mod sandbox.
- `ci/run.sh`: the CI build. It places the mod at `mods/aetherium` in the pinned Horizon source (tag v0.7.6), runs the unit tests, `cargo horizon-mod build`, `check` and `test`.

The workflow is `.github/workflows/horizon.yml`. A commit with `[horizon-release]` publishes `horizon-v<version>+26.3`. See `release_notes.md` for features, install steps and limits.
