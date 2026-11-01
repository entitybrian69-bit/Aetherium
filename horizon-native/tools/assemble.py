#!/usr/bin/env python3
"""Writes aetherium/mixins/aetherium/runtime.rs: aetherium/src/runtime_main.rs with the .hmod's zoom and
waypoint maths (horizon/aetherium/src/{zoom,waypoints}.rs) inlined as modules, since a horizon-mixins
module must be one self-contained file. `--check` fails when the committed file is out of date."""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NATIVE = ROOT / "horizon-native" / "aetherium"
OUT = NATIVE / "mixins" / "aetherium" / "runtime.rs"


def inline(name):
    text = (ROOT / "horizon" / "aetherium" / "src" / f"{name}.rs").read_text()
    # The unit tests stay with the .hmod crate (cargo test there).
    text = re.split(r"^#\[cfg\(test\)\]\s*$", text, flags=re.M)[0].rstrip()
    # Inner doc comments become plain ones: the module is nested, not a file.
    text = re.sub(r"^//!", "//", text, flags=re.M)
    body = "\n".join(("    " + line) if line.strip() else "" for line in text.splitlines())
    return f"\n// ---- {name}.rs of the .hmod (horizon/aetherium/src/{name}.rs), copied by tools/assemble.py.\n" \
           f"#[allow(dead_code)]\nmod {name} {{\n{body}\n}}\n"


def build():
    main = (NATIVE / "src" / "runtime_main.rs").read_text().rstrip() + "\n"
    return main + inline("zoom") + inline("waypoints")


def main():
    text = build()
    if "--check" in sys.argv:
        if not OUT.is_file() or OUT.read_text() != text:
            sys.exit(f"{OUT.relative_to(ROOT)} is out of date: run python3 horizon-native/tools/assemble.py")
        print("runtime.rs is up to date")
        return
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(ROOT)} ({len(text)} bytes)")


if __name__ == "__main__":
    main()
