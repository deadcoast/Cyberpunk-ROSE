#!/usr/bin/env bash
# One command: build the core, run its tests, build the GDExtension, install it into the Godot project,
# run the headless smoke test, and write the evidence renders.   Usage: ./run.sh [path-to-godot-binary]
set -euo pipefail
cd "$(dirname "$0")"
GODOT="${1:-${GODOT:-godot}}"
echo "  ROSE engine  --  build + test + smoke"
echo "  -------------------------------------"
cargo build --release -q
cargo test --release -q -p matsim 2>&1 | grep -E "test result" | head -1
case "$(uname -s)" in
  Linux)  cp target/release/librose_godot.so godot/bin/ ;;
  Darwin) cp target/release/librose_godot.dylib godot/bin/ ;;
  MINGW*|MSYS*|CYGWIN*) cp target/release/rose_godot.dll godot/bin/ ;;
esac
./target/release/matsim --out out evidence --seed 11
./target/release/matsim --out out scale --agents 3000 --seed 11
if command -v "$GODOT" >/dev/null 2>&1; then
  "$GODOT" --headless --path godot --import >/dev/null 2>&1 || true
  "$GODOT" --headless --path godot --script res://tests/smoke.gd 2>&1 | grep "\[smoke\]"
else
  echo "  godot binary not found ($GODOT); pass its path as the first argument to run the smoke test"
fi
