# Aetherium: native mod for Horizon (desktop)

Aetherium's settings screen, zoom, waypoints and info overlay as a
[horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle for
[Horizon](https://github.com/horizon-rust/horizon) (Minecraft Java 26.3 in Rust). The Horizon launcher builds it into the
game. Unlike the sandboxed `.hmod` in `../horizon/`, a native mod can draw its own screens, so this one has the real
Aetherium GUI (a port of `common/src/main/java/com/aetherium/gui/`).

| Path | What it is |
|---|---|
| `aetherium/compatibility.json` | The Horizon and horizon-mixins commits it is built on (pinned: it hooks Options itself) |
| `aetherium/mixins/aetherium/horizon-mixin.toml` | The bundle: two modules and the HEAD hooks |
| `aetherium/mixins/aetherium/gui.rs` | The screen, added to `horizon-client-render/src/options_ui.rs` as `options_ui::aetherium` |
| `aetherium/mixins/aetherium/runtime.rs` | The game side (`game::aetherium_rt`); generated, do not edit |
| `aetherium/src/runtime_main.rs` | The source of `runtime.rs`, before `tools/assemble.py` inlines the `.hmod`'s zoom and waypoints |
| `aetherium/mixins/aetherium/*.rs` | One small helper per hook |
| `tools/native.py` | Apply on Horizon, `cargo check`, package the `.hnmod` (Horizon's own tool, for pinned mods) |
| `ci/run.sh` | What CI runs; results in `ci/out/` |

```sh
python3 horizon-native/tools/assemble.py                 # after editing runtime_main.rs or ../horizon/aetherium/src
python3 horizon-native/tools/native.py check aetherium   # apply on the pinned Horizon, cargo check (Rust 1.99.1)
python3 horizon-native/tools/native.py package aetherium # horizon-native/target/aetherium-1.0.0.hnmod
```

`[hnmod-release]` in a commit message publishes the `.hnmod` as the release `hnmod-v<version>+26.3`.
