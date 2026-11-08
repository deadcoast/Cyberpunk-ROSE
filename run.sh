#!/usr/bin/env bash
# Cyberpunk: ROSE — the one command.
# Builds release, runs every Rust test, regenerates placeholders + prompt sheet + contact sheet and validates,
# installs the GDExtension into the Godot project, imports it, runs the headless smoke test, and, when a display or
# Xvfb is available, writes native screenshots of every screen to docs/shots/.
#   ./run.sh [path-to-godot-4.5-binary]        (or set GODOT=...)
set -euo pipefail
cd "$(dirname "$0")"
GODOT="${1:-${GODOT:-godot}}"
step() { printf '\n  %-14s %s\n' "$1" "$2"; }

step "build" "cargo build --release (core, bridge, tools)"
cargo build --release -q
step "test" "cargo test --release"
cargo test --release -q 2>&1 | grep -E "test result" | awk '{p+=$4; f+=$6} END {printf "  %d passed, %d failed\n", p, f; exit (f>0)}'
step "assets" "rose-tools all"
./target/release/rose-tools all | tail -6
step "sim" "rose scale (headless renders to out/)"
./target/release/rose --out out scale --agents 3000 --seed 11 | tail -4
step "route" "rose route (seeded assembly, 200-seed sweep per stratum)"
./target/release/rose --out out route --seed 11 --stratum 1 --sweep 200 | tail -6

mkdir -p game/bin
case "$(uname -s)" in
  Linux)  cp target/release/librose_godot.so game/bin/ ;;
  Darwin) cp target/release/librose_godot.dylib game/bin/ ;;
  MINGW*|MSYS*|CYGWIN*) cp target/release/rose_godot.dll game/bin/ ;;
esac

if command -v "$GODOT" >/dev/null 2>&1; then
  step "godot" "import + headless smoke"
  "$GODOT" --headless --path game --import >/dev/null 2>&1 || true
  "$GODOT" --headless --path game --script res://tests/smoke.gd 2>&1 | grep "\[smoke\]"
  if [ -n "${DISPLAY:-}" ]; then
    step "shots" "native screenshots -> docs/shots"
    "$GODOT" --path game --rendering-driver opengl3 --script res://tests/shots.gd 2>&1 | grep "\[shots\]" || true
  elif command -v xvfb-run >/dev/null 2>&1; then
    step "shots" "native screenshots via Xvfb -> docs/shots"
    xvfb-run -a -s "-screen 0 1920x1080x24" "$GODOT" --path game --rendering-driver opengl3 --script res://tests/shots.gd 2>&1 | grep "\[shots\]" || true
  fi
else
  step "godot" "binary not found ($GODOT): pass its path as the first argument to run the shell tests"
fi
printf '\n  done\n\n'
