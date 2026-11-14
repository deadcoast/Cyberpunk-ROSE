#!/usr/bin/env bash
# Cyberpunk: ROSE — the one command.
# Builds release, runs every Rust test, regenerates placeholders + prompt sheet + contact sheet and validates,
# installs the GDExtension into the Godot project, imports it, runs the headless smoke test, and, when a display or
# Xvfb is available, writes native screenshots of every screen to assets/shots/ and plays the scripted session
# against its recorded digests.
#   ./run.sh [--quick] [path-to-godot-4.5-binary]        (or set GODOT=...)
#   --quick: the per-commit tier: build, tests, rose city, the smoke and the played session (play.gd: every beat it
#   plays held to happening, then its recorded digests); the proofs, the assets and the screenshots run in the full
#   tier.
set -euo pipefail
cd "$(dirname "$0")"
QUICK=0
if [ "${1:-}" = "--quick" ]; then QUICK=1; shift; fi
GODOT="${1:-${GODOT:-godot}}"
STARTED=$(date +%s)
step() { printf '\n  %-14s %s\n' "$1" "$2"; }

install_ext() {
  mkdir -p game/bin
  case "$(uname -s)" in
    Linux)  cp target/release/librose_godot.so game/bin/ ;;
    Darwin) cp target/release/librose_godot.dylib game/bin/ ;;
    MINGW*|MSYS*|CYGWIN*) cp target/release/rose_godot.dll game/bin/ ;;
  esac
}

# The played session (game/tests/play.gd, --fixed-fps 60): every beat it plays held to happening (a beat that did not
# fails the tier, named), then its digests against the recorded ones: the live game runs the one game::World `rose`
# plays, so the session's digests (assets/shots/play_digest.txt, 34 frames) change only when the game does. A change
# is reported with the first frame it shows at; name it in the verification log.
play_session() {
  if ! command -v xvfb-run >/dev/null 2>&1 && [ -z "${DISPLAY:-}" ]; then
    printf '  no display or Xvfb: the played session did not run\n'
    return
  fi
  local run=("$GODOT" --path game --rendering-driver opengl3 --fixed-fps 60 --script res://tests/play.gd)
  if [ -z "${DISPLAY:-}" ]; then run=(xvfb-run -a -s "-screen 0 1920x1080x24" "${run[@]}"); fi
  local out status=0
  out=$("${run[@]}" 2>&1) || status=$?
  printf '%s\n' "$out" | grep -E "\[play\] (the ending|[0-9]+ frames|PASS|FAIL)" || true
  if [ "$status" -ne 0 ]; then
    printf '  the played session FAILED: a beat it plays did not happen (each named above)\n'
    return 1
  fi
  if ! git ls-files --error-unmatch assets/shots/play_digest.txt >/dev/null 2>&1; then
    printf '  the played session'"'"'s digests are not recorded yet: commit assets/shots/play_digest.txt\n'
  elif git diff --quiet -- assets/shots/play_digest.txt; then
    printf '  the played session: every frame'"'"'s digest as recorded\n'
  else
    local first
    first=$(git diff -U0 -- assets/shots/play_digest.txt | grep -m1 '^+[0-9]' | awk '{print substr($1,2)}')
    printf '  the played session changed from frame %s: name the change in the verification log\n' "$first"
  fi
}

# The screenshots (game/tests/shots.gd, --fixed-fps 60): every screen at native size, each stage a shot sets up held
# to happening (a stage refused fails the tier, named: a shot of the wrong screen is no evidence).
shots_run() {
  local run=("$GODOT" --path game --rendering-driver opengl3 --fixed-fps 60 --script res://tests/shots.gd)
  if [ -z "${DISPLAY:-}" ]; then run=(xvfb-run -a -s "-screen 0 1920x1080x24" "${run[@]}"); fi
  local out status=0
  out=$("${run[@]}" 2>&1) || status=$?
  printf '%s\n' "$out" | grep "\[shots\]" || true
  if [ "$status" -ne 0 ]; then
    printf '  the screenshots FAILED: a stage did not happen (each named above)\n'
    return 1
  fi
}

if [ "$QUICK" = 1 ]; then
  step "build" "cargo build --release (core, bridge, tools)"
  cargo build --release -q
  step "test" "cargo test --release"
  cargo test --release -q 2>&1 | grep -E "test result" | awk '{p+=$4; f+=$6} END {printf "  %d passed, %d failed\n", p, f; exit (f>0)}'
  step "city" "rose city (twenty shifts headless -> out/city.md)"
  ./target/release/rose --out out city --shifts 20 --report out/city.md --log out/city.log | tail -3
  install_ext
  if command -v "$GODOT" >/dev/null 2>&1; then
    step "godot" "import + headless smoke"
    "$GODOT" --headless --path game --import >/dev/null 2>&1 || true
    "$GODOT" --headless --path game --script res://tests/smoke.gd 2>&1 | grep -E "\[smoke\] ([0-9]+ screens|PASS|FAIL)|SCRIPT ERROR"
    step "session" "play.gd under --fixed-fps 60: every beat held to happening, then play_digest.txt"
    play_session
  else
    step "godot" "binary not found ($GODOT): pass its path to run the smoke and the played session"
  fi
  printf '\n  quick tier done in %d s\n\n' "$(( $(date +%s) - STARTED ))"
  exit 0
fi

step "build" "cargo build --release (core, bridge, tools)"
cargo build --release -q
step "test" "cargo test --release"
cargo test --release -q 2>&1 | grep -E "test result" | awk '{p+=$4; f+=$6} END {printf "  %d passed, %d failed\n", p, f; exit (f>0)}'
step "assets" "rose-tools all"
./target/release/rose-tools all | tail -6
step "sim" "rose scale (headless renders to out/)"
./target/release/rose --out out scale --agents 3000 --seed 11 | tail -4
step "route" "rose route (seeded assembly, 200-seed sweep per stratum)"
./target/release/rose --out out route --seed 11 --stratum 1 --sweep 200 --loot 200 | tail -11
step "cadence" "rose cadence (20 bot runs per stratum against the budget -> docs/03/telemetry/cadence.md)"
./target/release/rose --out out cadence --runs 20 --strata 1,2,3 --seed 1 --report docs/03/telemetry/cadence.md | tail -6
step "city" "rose city (twenty shifts headless, threats live -> docs/03/telemetry/city.md)"
./target/release/rose --out out city --shifts 20 | tail -3
step "cleansing" "rose cleansing (Phase D's proof: a Cleansing with and without Gates -> docs/03/telemetry/cleansing.md)"
./target/release/rose --out out cleansing | tail -2
step "economy" "rose economy (Phase D's proof: thirty shifts trading, every demand inside its band; then feeding -> docs/03/telemetry/economy.md)"
./target/release/rose --out out economy | tail -3
step "campaign" "rose campaign (Phase E's proof: five scripts, each to its own ending -> docs/03/telemetry/campaign.md; about twenty-five minutes)"
./target/release/rose --out out campaign | tail -3

install_ext

# --fixed-fps 60: every frame advances the clock by 1/60 s whatever the machine's speed, so the scripted session
# plays the same and its frames and digests repeat byte for byte (a changed frame is a changed game)
if command -v "$GODOT" >/dev/null 2>&1; then
  step "godot" "import + headless smoke"
  "$GODOT" --headless --path game --import >/dev/null 2>&1 || true
  "$GODOT" --headless --path game --script res://tests/smoke.gd 2>&1 | grep -E "\[smoke\]|SCRIPT ERROR"
  if [ -n "${DISPLAY:-}" ] || command -v xvfb-run >/dev/null 2>&1; then
    step "shots" "native screenshots -> assets/shots (every stage held to happening)"
    shots_run
    step "session" "play.gd -> assets/shots/play_*.png, every beat held to happening, against play_digest.txt"
    play_session
  else
    step "shots" "no display or Xvfb: the screenshots and the played session did not run"
  fi
else
  step "godot" "binary not found ($GODOT): pass its path as the first argument to run the shell tests"
fi
printf '\n  done in %d s\n\n' "$(( $(date +%s) - STARTED ))"
