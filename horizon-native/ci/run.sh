#!/usr/bin/env bash
# CI for Aetherium's native Horizon mod: the runtime file is up to date, the .hnmod packs, and the
# mod applies on the pinned Horizon and the whole game passes `cargo check` with -D warnings.
# Results go to horizon-native/ci/out/ (committed by the workflow) so they can be read back.
set -uo pipefail
cd "$(dirname "$0")/../.."
OUT=horizon-native/ci/out
rm -rf "$OUT"; mkdir -p "$OUT"
status=ok
step() {
  local name=$1; shift
  echo "== $name" | tee -a "$OUT/status.txt"
  if "$@" > "$OUT/$name.log" 2>&1; then
    echo "   ok" | tee -a "$OUT/status.txt"
  else
    echo "   FAILED" | tee -a "$OUT/status.txt"
    status=failed
    # The compiler's errors and warnings, without the progress lines.
    grep -nE '^(error|warning)|^ *--> |^ *= (note|help)|horizon-mixins|E_[A-Z_]+' "$OUT/$name.log" | head -400 > "$OUT/$name.errors.txt" || true
    tail -150 "$OUT/$name.log" > "$OUT/$name.tail.txt"
    return 1
  fi
}
step assemble python3 horizon-native/tools/assemble.py --check
step package python3 horizon-native/tools/native.py package aetherium && cp horizon-native/target/aetherium-*.hnmod "$OUT/" 2>/dev/null
mkdir -p dist && cp horizon-native/target/aetherium-*.hnmod dist/ 2>/dev/null || true
if [ "$status" = ok ]; then
  step check python3 horizon-native/tools/native.py check aetherium
fi
# Keep the applied source of the files the bundle changes, to read what horizon-mixins generated.
stage=$(ls -d horizon-native/target/stage/aetherium 2>/dev/null)
if [ -n "$stage" ]; then
  mkdir -p "$OUT/applied"
  for f in crates/horizon-client-render/src/options_ui.rs crates/horizon-client/src/game/native_hooks.rs; do
    grep -n "aetherium" "$stage/$f" > "$OUT/applied/$(basename "$f").grep.txt" 2>/dev/null || true
  done
fi
for f in dist/*.hnmod; do [ -f "$f" ] && sha256sum "$f" | tee "$f.sha256"; done
grep -oP 'version = "\K[^"]+' horizon-native/aetherium/mixins/aetherium/horizon-mixin.toml | head -1 > "$OUT/version.txt"
echo "$status" > "$OUT/result.txt"
echo "result: $status"
[ "$status" = ok ]
