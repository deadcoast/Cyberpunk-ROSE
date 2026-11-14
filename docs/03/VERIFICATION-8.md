# VERIFICATION — Phase G (from the current state), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-100 — the current state reviewed, the agent spec written, the moved docs linked (2026-11-13)

**The review**: `docs/03/CURRENT-STATE.md`. Its numbers come from `cargo test`, the four proofs as run at PAC-083 on
the same code (`docs/03/telemetry/`), `rose bench`, a shift-by-shift timing of `rose campaign --aim world_eater`
(0.31 s of wall time for shift 10 at 26 slimos; 3.32 s for shift 110 at 1,726 slimos and 18,511 joined cells, a
5.3-minute city shift), `game/tests/shots.gd` and `play.gd` under Xvfb (23 screens and 34 frames, rewritten into
`assets/shots/`), and the source at the lines it cites. Two claims were corrected before it was written down: the
run's key line fades after its first seconds (`Run.gd:422`), so it is not listed; and the "loss lapses" verdicts are
quiet stretches past two minutes at a player's pace, so Pillar 5 is partial, not met.

**The agent spec**: `AGENTS.md` at the root: where the truth lives, the verification logs by number, the register's
sixteen columns as this repository fills them, the rules, and the two lanes. The art lane's facts were read from the
code: anchors at 64 × 64, 32 × 48 and 32 × 32 (`game/assets/manifest/anchors.toml`), the `ui.font.mono` slot
(`ui.toml:74`), the validator's checks (`game/assets/manifest/README.md`), and that `Assets.gd` draws any PNG in
`art/` without checking it — which the manifest README did not say, now CURRENT-STATE ART-04.

**The register**: Phase G added, PAC-100 to PAC-110, each row's Context naming the CURRENT-STATE IDs it answers;
PAC-090 to PAC-092 assigned to `@pixel-lab-agent` with `AGENTS.md` as their contract; PAC-093's note says what
stays code. PAC-063 repaired: its DoD's commas were unquoted when it was written (`d5f24e2`), which split it into 18
fields, and its DONE edit (`2091d5a`) wrote the dates over BlockedBy and Assignee; it now reads as first written with
the done dates and notes kept. Every row has sixteen fields, every ID is unique and every blocker exists (checked with
Python's `csv`). The bundled PACER validator also accepts the new rows' timestamps; it rejects every phase name,
because its list is another project's.

**The moved docs**: the user moved the design, logs and telemetry into `docs/03/` and the screenshots into
`assets/shots/`. Every reference now points there: the README, the design, the art bible, research 05, the
verification logs, the register, `run.sh`, the `rose` report defaults, `shots.gd`, `play.gd` and `rose-tools`.
`grep` for `docs/telemetry`, `docs/shots`, `docs/pixellab-prompts`, `docs/FullDesign`, `docs/art-bible` and
`VERIFICATION-phase` outside `target/` and `out/` finds nothing.

**The generators keep the hand edits**: `rose-tools prompts` writes `docs/03/pixellab-prompts.md` with each slot as
its id over a `---` line, the format the file was hand-edited to; regenerated, the file is byte-identical to the
committed one (`diff`, no output). The proofs write their tables with `| --- |` rows into `docs/03/telemetry/`;
`rose city` regenerated `city.md` identical to the hand-edited one, and no `docs/telemetry/` was created.

**Still green**: `cargo test --release --workspace --exclude rose-godot` 191 passed, 0 failed, 1 ignored; `rose city`
PASS; the headless smoke PASS; `shots.gd` wrote 23 screens and `play.gd` 34 frames to `assets/shots/` and reached
its ending (Hive, warchest 743).

## Phase G baseline — 461b80c measured in the cloud workspace (2026-11-13)

**The engine**: Godot 4.5.1 stable, the official Linux build, its SHA-512 checked against the release's sums; the
user's editor is the same version (`.vscode/settings.json`). The first `--import` of a fresh project cache crashes
with signal 11 on exit; the second is clean. `run.sh` already ignores the import's exit status.

| check at 461b80c | result |
| --- | --- |
| `cargo test --release --workspace --exclude rose-godot` | 191 passed, 0 failed, 1 ignored (4 min 43 s cold, two cores) |
| `rose city` | PASS, replays bit-exact; `city.md` identical to the committed file but for the log's path |
| `rose cleansing`, `rose economy` | PASS, PASS; `economy.md` identical; `cleansing.md` identical in its numbers, its two log headings and code fences in the generator's old format |
| `rose campaign` | PASS, all five endings identical to `telemetry/campaign.md` (2,323 s here; the seconds column is the machine's) |
| `rose cadence` | identical to `telemetry/cadence.md` but for the date |
| `rose-tools all` | 419 placeholders, the prompts file and the contact sheet byte-identical |
| `cargo clippy` | 28 warnings on cargo 1.97 (CURRENT-STATE counted 20 on another toolchain) |
| the smoke | PASS, but `smoke.gd:366` called `sim.step()` on the Sim autoload, which has no `step`: a script error, so the Bench check's 1,201 ticks never ran and the shiv was checked after a shift end alone |
| `play.gd` | reaches the Hive ending, warchest 690 (PAC-100 logged 743 on its machine); two plays diverge from frame `09_extracted` |
| `shots.gd` | 23 screens; two runs differ in 10 of them |

**Why the frames did not repeat**: the Run screen's clock adds each frame's delta, and a frame's delta follows the
wall clock; every animation phase (pulses, blinks, a strip's frame, a fresh tile's glow) read
`Time.get_ticks_msec()`; New Run offers a seed from the engine's random stream. Fixed in the instruments below.

**The instruments** (overlord OVR-009, all additions, no rule changed):

- `game::city::digest_split` — the city's digest whole and by part (Mat, structures, Stash and ledger, roster, log,
  surface); `rose idle --seed S --shifts K` prints it for the World founded and left alone, a line a shift;
  `RoseSim.digest_split()` and `game/tests/world.gd` print the same lines from the live game.
- `--fixed-fps 60` on `shots.gd` and `play.gd` (run.sh and AGENTS.md), `Sim.anim_ms` as the picture's clock in place
  of the wall clock at the eight animation sites, and `seed(11)` in both scripts: two runs of each now write
  byte-identical files (the 23 screens; the 34 frames and the digest file). `play.gd` writes `assets/shots/play_digest.txt`, the city's digest at each of
  its 34 frames.
- The smoke calls `sim.ensure().step()`: the Bench's ten work cycles now run before the shift ends, and the check
  still passes (221 textures resolved, the cycles' frames among them).
- `rose cleansing` writes its logs as the committed file has them (a quoted heading over a `bash` fence); the
  regenerated `cleansing.md` is byte-identical. `rose campaign`'s per-script tables write `| --- |` throughout.

**The live game against the World at 461b80c**: founded on seed 11 and left alone, the two print the same digests,
every part, for 60 shifts. An idle city never staffs a post, takes an offer or reaches a grate, so none of the drift
in OVR-006 shows there; the digest check guards the founding and the idle shift, and `play_digest.txt` guards the
played session.

**The quick tier** (OVR-008): `./run.sh --quick godot` builds, tests, plays `rose city`, runs the smoke and the
same-city check (six digests equal) in 75 s on an incremental build. A cold build adds the compile (about 5 min on
two cores).

## PAC-101 (part 1) — one data loader, the World on the game's shift, the crowd's depth packed (2026-11-13)

**One loader** (OVR-010): `game::data` loads items, structures, laws, recipes, traits, names, chunks, loot, creatures
and factions once a process. The game installs it as the extension loads (`on_stage_init`, the Scene stage) from
`data/` beside the project; `rose --data DIR` installs it from a directory; without either, the compiled copies. The
bridge's `data_text` and its silent fallback are gone, and so are its two private `include_str!` copies of the items
and chunks; every runtime reader in `game`, the bridge and `rose` goes through it. Checked:

- `cargo test`: 193 passed, 1 ignored; the two new tests load a directory whose `laws.toml` is edited (read from
  disk) and whose `recipes.toml` does not parse (reported with its file, the compiled copy in its place).
- The smoke reads `RoseSim.data_report()`: 10 of 10 files from `data/`. With a line of garbage appended to
  `data/laws.toml` the game prints `ERROR: data/laws.toml: TOML parse error at line 94, column 6` and the smoke
  fails on it; restored, it passes.
- `play.gd`'s 34 digests identical to 461b80c's: the game reads the same data through the loader.

**The World on the game's shift** (OVR-011): `World::start_shift` (the movers and posts, the surface's start, the
first ten minutes' beats), used at the founding and every shift end; the posts from every work cycle's plan; the
Board's strata from the founding (`board_strata` of the city's stratum: S1–S3) and from each breach, the raid chance
with them; a recover's body laid through a Board on the shipped chunks, kept in `World::laid`. Measured against the
baseline:

| proof | 461b80c | after |
| --- | --- | --- |
| `rose city`, `cleansing`, `economy` | PASS | PASS, every report identical (no raider offer, Guard post or breach in their 20–30 shifts changes a number they print) |
| `rose campaign` garden | 180 shifts, warchest 9,119 | 146 shifts, warchest 45,209 |
| hive | 134, 52,408 | 134, 52,860 |
| world_eater | world_eater, 124, 49,891 | **garden**, 115, 43,027 |
| accord | 175, 17,105 | 159, 17,691 |
| verdict | PASS | **FAIL**: a script missed its ending |

Which change moves them, by a build that switches each off by name (`ROSE_REVERT`, not committed): with every change
off the Garden script reproduces 461b80c's log line for line; through shift 60 only the founding's strata or the
breach's change it, each alone giving the after run's numbers exactly (Slimos 110 at shift 40 against 122); the
posts change it from shift 80. With the raid changes off and the posts on, the World-Eater script ends as at
461b80c (124 shifts, 49,891). The mechanism: with raiders walking the Board, each reached faction's offer slots draw
from its clear-raiders template too (`economy::template_open`), the steward takes only deliver and host links, so
the offers it takes, the newcomers it hosts and the feeds that follow change from the first grate on. The
World-Eater's strain ends with Lume 0.97 and Thrift 0.94 (0.89 and 0.90 before) beside Appetite and Feral Drive at
1.00, and the ending's score gives the ally's reputation a tenth between those two endings a step (Garden +0.05,
World-Eater −0.05): one step past the first puts the Garden ahead. The proof's World-Eater ending stood on a World
that offered no clear-raiders link; the steward's script is the open item (OVR-017), and CURRENT-STATE's campaign
hours (CORE-01 to CORE-03) were measured on that World, so PAC-105 starts from the table above.

**The crowd's depth** (OVR-014): `RoseSim.agent_depths()`, one byte a mover in `agent_ids` order; `WorldView` draws
from it and no longer builds a record a slimo a tick. Every one of `shots.gd`'s 23 screens and `play.gd`'s 34 frames and digest file is
byte-identical before and after. Its cost at a crowded late city is not measured (no late city in the game until
PAC-103).

**Not checked**: the live game still runs the bridge's own composition until OVR-013, so its posts, Board and bodies
are as they were; the steward's World-Eater script; frame time.

## PAC-101 (part 2) — the game runs the one World; the scripts hold their beats (2026-11-14)

**RoseSim owns the World** (OVR-013): `RoseSim` holds one `game::World`, founded by `World::new_run` as `rose` founds
it, and nothing else of the city's: `step` is `World::tick`, `colony_end_shift` is `World::end_shift`, the movers
are `World::movers`, the alerts `World::alerts`. The bridge's own copy of the city — its tick, shift end, founding,
movers, log, plan, threats and colony, the 26 fields `RoseSim` carried at 461b80c — is deleted. `RoseStash` and
`RosePeople` hold nothing: `Sim.gd` attaches each to the `RoseSim` (`attach`) and they read and change its World.
The bridge's `.unwrap()`s: 98 → 0. Before the two scripts below changed, `play.gd`'s 34 digests (all seven parts) and
every frame of `shots.gd` and `play.gd` were identical to 461b80c's: the game played the same on the one World.

**The bridge split by responsibility** (OVR-015): `lib.rs` 3,031 lines → 71 (the extension's entry, the data
install, `decide`); `sim_bridge.rs` (RoseSim: founding, the clock, the shift, the city's verbs), `stash_bridge.rs`
(RoseItems, RoseStash), `run_bridge.rs` (RoseRun) beside the earlier `*_bridge.rs` files. Moved, not changed: the
digests and frames identical across the split.

**The scripts were not checks** (found here, fixed): `play.gd` printed what each verb answered and played on. At
461b80c and up to this entry, 12 of the beats it scripts did not happen — the Chrome tier never opened (the study
needs researchers' points, which the living city of v3.21 puts in inside the shift, and the script ended 20 shifts
with no clock between them), so the Sphincter housing was never researched, the Relay Shrine never built and its
rules never set; the Stash was full, so the Sump Grate Key never went in, the slab never opened and the Mat never
climbed; so chapter 3 never opened, no envoy came, ACCEPT answered "no envoy is waiting" and the Tithe was never
signed. Every one of those printed a line and the session ended `quit(0)`; its frames showed the screens without
the thing named. `shots.gd` the same: 4 of its stages were refused (the Stash's sale and feed: the colony's shots had
sold its curios; the Bench's shiv: the builders had spent the housings).

Now both scripts hold each beat and stage to happening (`_expect`): one that does not fails the script, named, with
what was answered, and `run.sh` fails with it (the quick tier's played session and the full tier's shots and
session). A shift in either is a played shift — `World::play_shift`, the city's clock for `SHIFT_TICKS` (ten minutes
at 2 Hz) then its end, through `RoseSim.colony_play_shift` and `Sim.play_shift` — where the script waits on what the
clock does; `shots.gd` adds what its Stash and Bench shots queue, as a haul brings it. Checked:

- `play.gd`: PASS, 37 beats. Chrome tier 2 opens 4 shifts after the study; the Sphincter housing 13 after; the Shrine
  built in 1, both rules set; the key opens the slab; the Mat reaches the Culverts (S2); chapter 3 opens with the
  envoy waiting at the grate nearest the Heart; ACCEPT signs the Tithe; the Hive ending, warchest 1,630.
- The same script with `Sim.play_shift` put back to a bare shift end: FAIL, 12 of 37, the twelve above named. The
  checks fail when the game does not do the beat.
- `shots.gd`: PASS, 28 stages (the Rack built in a played shift, its stockpile painted; the sale, the feed, the shiv).
- Two plays of each: the digests and all 67 frames repeat byte for byte.
- `play_digest.txt` is recorded new here: the session now plays shifts on the clock, so its city from frame 17_node
  on is not 461b80c's. The 18 frames 00_board to 16_study keep 461b80c's digests.
- `cargo test`: 193 passed (`World::play_shift` is the tests' shift); `./run.sh --quick`: PASS.

**Not checked**: the steward's World-Eater script (OVR-017, the campaign proof still FAILS as in part 1); every verb
through `World::apply` (OVR-012); frame time.
