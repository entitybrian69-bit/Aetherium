#!/usr/bin/env bash
# Builds and tests Aetherium for Horizon against the pinned Horizon Android
# source (HZ), the way Horizon builds its own official mods: the mod sits in
# the Horizon workspace as mods/aetherium and `cargo horizon-mod` turns it
# into a sandboxed .hmod (a WebAssembly component plus its manifest).
# Writes logs and a status line per stage to horizon/ci/out/.
set -u
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
OUT="$ROOT/horizon/ci/out"
rm -rf "$OUT"; mkdir -p "$OUT"
: "${HZ:?HZ must point at the Horizon source}"
VERSION=$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$ROOT/horizon/aetherium/horizon-mod.toml" | head -1)
echo "$VERSION" > "$OUT/version.txt"
status() { echo "$1=$2" >> "$OUT/status.txt"; echo "::notice::$1 exit $2"; }

# 1. The mod inside the Horizon workspace.
rm -rf "$HZ/mods/aetherium"
cp -r "$ROOT/horizon/aetherium" "$HZ/mods/aetherium"
rm -rf "$HZ/mods/aetherium/target"
if ! grep -q '"mods/aetherium"' "$HZ/Cargo.toml"; then
  sed -i 's#^    "mods/official/zoom",#    "mods/official/zoom",\n    "mods/aetherium",#' "$HZ/Cargo.toml"
fi
grep -q '"mods/aetherium"' "$HZ/Cargo.toml" || { echo "could not add the workspace member" > "$OUT/place.log"; status place 1; exit 1; }
status place 0

cd "$HZ/mods/aetherium"
# 2. Unit tests (zoom maths, waypoints, settings) on the host.
cargo test -p aetherium > "$OUT/unit-tests.log" 2>&1; s=$?; status unit_tests $s
tail -n 40 "$OUT/unit-tests.log"
[ $s = 0 ] || exit 1

# 3. The .hmod.
cargo horizon-mod build > "$OUT/build.log" 2>&1; s=$?; status build $s
tail -n 40 "$OUT/build.log"
HMOD="$HZ/target/horizon-mod/aetherium.hmod"
[ $s = 0 ] && [ -f "$HMOD" ] || exit 1

# 4. The registry's checks, with a trial run of the mod.
cargo horizon-mod check "$HMOD" > "$OUT/check.log" 2>&1; s=$?; status check $s
cat "$OUT/check.log"
[ $s = 0 ] || exit 1

# 5. The scenario tests in tests/*.toml, in the game's own sandbox.
cargo horizon-mod test --hmod "$HMOD" > "$OUT/scenario-tests.log" 2>&1; s=$?; status scenario_tests $s
cat "$OUT/scenario-tests.log"
[ $s = 0 ] || exit 1

mkdir -p "$ROOT/dist"
NAME="aetherium-$VERSION-horizon-26.3.hmod"
cp "$HMOD" "$ROOT/dist/$NAME"
( cd "$ROOT/dist" && sha256sum "$NAME" > "$NAME.sha256" && cat "$NAME.sha256" ) | tee "$OUT/sha256.txt"
ls -l "$ROOT/dist" >> "$OUT/sha256.txt"
unzip -l "$ROOT/dist/$NAME" > "$OUT/hmod-contents.txt" 2>&1 || true
status all 0
