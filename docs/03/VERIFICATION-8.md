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

## PAC-101 (part 3) — the campaign proof on the game's shift; PAC-101 closed (2026-11-15)

**The World-Eater script** (OVR-017): traced with `rose campaign --aim world_eater`. The ending reads the genes and the
ally's reputation past the alliance's own step (Garden +0.05 a step, World-Eater −0.05). The script signed the alliance
with the Gangs at +1, and then bought the Gangs' protection every shift a Patrol was due at one of their grates: the
Gangs' lever is protection bought, two favour a day, and ten days of it carried them to +2. At +2 the Garden scored
(0.97 + 0.94) / 2 + 0.05 = 1.005 against the World-Eater's 1.00 − 0.05, and the Great Cleansing's "the ally's 4
militia" in the log is the +2 (two a step). The fix is the steward's: for an ending the ally's steps cost (the
World-Eater's) it buys no service from the ally once it is signed (`Steward::shun`). The World-Eater script then ends
World-Eater (114 shifts, warchest 50,061, Lume 0.94 and Thrift 0.93); the ending's score is the design's and is
unchanged.

**The full proof**: `rose campaign` PASS, every script its own ending (1,397 s on two cores):

| script | ending | shifts | Hamlet | Quarter | City | Empire | the end |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Garden | Garden | 146 | 0.6 h | 6.1 h | 10.4 h | 19.8 h | 20.8 h |
| Hive | Hive | 134 | 0.6 h | 4.5 h | 7.8 h | 18.1 h | 19.1 h |
| World-Eater | World-Eater | 114 | 0.6 h | 6.1 h | 9.8 h | 15.4 h | 16.3 h |
| Accord | Accord | 159 | 0.6 h | 5.3 h | 9.7 h | 21.1 h | 22.2 h |
| Scattered | Scattered | 11 | 0.6 h | — | — | — | 1.7 h |

This table is PAC-105's starting point on the game's shift (the hours are the bot's run seconds times 3).

**OVR-012 re-scoped, not built**: every verb through `World::apply(Input)` was planned so that S2's save could be the
seed and an input log. The save is the World's snapshot instead (PAC-102 below: loading a late city takes under a
tenth of a second; replaying nine of its shifts would take about 30 s), so nothing in S2 imports `World::apply`. The
row moves to S10 Code health as a code-shape step. PAC-101's DoD (RoseSim holds the World, no orchestration and no
unwrapped Option state in the bridge, the crowd's depth packed) was met in part 2; with the proof passing, the row closes.

## PAC-102 — saves on disk (2026-11-15)

**The World serialises** (OVR-020): serde on the Mat (`MatSim`, its genome, params, organs, the RNG), the logistics
`City` with its cargo fields and walk fields, the threats with their caches (only the rules, which are data, are taken
from the build on restore), the colony's per-shift working state (Nerve state, dimmed structures, dark bulbs, the
strain's shift, the Heart's distances, the wave: before this they were skipped, and a city restored at shift 60 parted
from the saved one in its roster within a shift), the log, the posts, the plan, the laid bodies and the sales.
`World::snapshot()` is postcard then deflate; `World::restore(bytes, table)` takes the data files from `game::data`.

- `cargo test`: `a_restored_world_steps_as_the_world_it_was_taken_from` — restored at the founding, 301 ticks into a
  shift, and after the city proof's 10 shifts, each plays on (3 and 2 played shifts) to the same digest by part, the
  same threats, surface, campaign, tick, nutrient and signal.
- `rose save-check` (new): one process plays the Garden script to shift N, writes the save and the digests of three
  played shifts after it (by part, and the threats, surface, campaign and routes by their printed state), and checks a
  restore in the same process; a second process loads the file and plays the same shifts:

| at shift | slimos | Mat cells joined | save | written in | loaded in | same process | fresh process |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10 | 26 | 614 | 704,681 bytes | 13 ms | 17 ms | the same digests | PASS, 4 digests |
| 60 | 360 | 10,227 | 4,755,196 bytes | 81 ms | 63 ms | the same digests | PASS, 4 digests |
| 110 | 1,562 | 12,737 | 9,157,918 bytes | 152 ms | — | the same digests | PASS, 4 digests |

**The campaign's state moves into the World** (OVR-021): the Board's routes — runs, fog memory, the bodies on them
(ours and recover contracts', which the World now lays itself at the shift's end and on taking an offer) and the
stranded — are `game::routes::RouteBook` in the World. `Sim.gd` reads them through `RoseSim` (`route_runs`,
`route_memory`, `bodies_on`, `stranded_on` and their writers); its own dictionaries are gone. The day is the World's
(`RoseSim.day()`). What stays in `Sim.gd` and rides in the save's view block: the camera, bookmarks, overlay, the
alerts shown, the focused slimo, the last run's card and haul, New Run's choices.

**The file** (OVR-022): save v18 is a header (seed, strain code, difficulty, created and saved times, the shift, the
tick, a label "shift 61 · Quarter · 360 slimos") and the World's snapshot and the view block. Written to `.tmp` and
renamed; a file that does not decode is renamed `.bad`. The three old save-crate tests became three on the new file
(a round trip and an older version refused; a flipped byte and foreign bytes refused; a newer version refused).

**The shell** (OVR-023): an autosave at every shift's end and as a run leaves the Board, before the rig is packed;
three slots on Settings (SAVE and LOAD), the autosave LOAD-only; CONTINUE on the menu loads the newest save that reads
when the session holds no city (the menu line names it and its time); Settings prints the city's seed and strain code
(`RoseSim.strain_code`), the last save, and each slot's label. The fixed "seed 11 … save: autosave each shift" is gone.
The tests keep their own save folders (`user://saves_test_<script>`), emptied as they start.

**Quit and relaunch** (OVR-024): `game/tests/save.gd -- write` founds the city, plays three shifts (each ending in its
autosave), writes slot 2, plays two more and quits; `-- read`, a fresh Godot process, finds the autosave newest, loads
it in 12 ms and matches the fifth shift's digests, loads slot 2 and matches the third's (and its route runs, memory,
body and view state), plays the two shifts and matches the fifth's again: PASS both. `run.sh` runs it in both tiers.

**Not checked**: a save written by a build and loaded by a later one (any change to the World's fields refuses it: a
development format, as the save crate's rule has it); the file's time on a slow disk.

## PAC-106 — the run's early rewards and its quiet stretches (2026-11-15)

**Notables in S1–S2** (OVR-070): Rare entered the loot rows at tier 3, so the first two strata rolled no notable (0.00 a
run). A weight of 1–2 in 100 in rows 1–2 gave 0.00 and 0.15 a run: notable picks run on PRD, and PRD starts each run
at the bottom of its ramp, so a small weight is nearly nothing over a run's thirty picks. Swept with `rose --data`
(no rebuild): lockers 3/4/5 and crates 5/6/8 in S1 gave 0.20/0.35/0.55 a run. Shipped: S1 lockers 6, crates 8; S2
lockers 4, crates 7 (in 100).

**The quiet stretches** (OVR-071): the stretches the bot met no trouble in were S1's routes (no patrol, crawlers that
stay at their burrows) and quiet S3 chunks. A rule, not a chunk edit: **the drain stirs** — a Runner off the Mat at
three fifths of their body or more who has met no trouble for `stir_s` (40 s, `data/creatures.toml`) brings a crawler
out of the burrow nearest them (`RunSim::stir`, an `Event::Stirred` the cadence log counts as a loss, as it counts
raiders coming down). Tried and not kept: a whole pack at 25 s or 35 s (the bot died on 60–80 % of its runs), one of a
pack at 30–38 s (38–55 % in S1). With one crawler the bot still died on 35–42 % in S1: the logs showed it bleeding out
from a hit it had no antiseptic left for while it went on looting. The bot now heads home when it bleeds with no
antiseptic below three fifths of its body (`run::bot`, `BLEED_HOME_HP`), a cautious Runner's call.

`rose cadence --runs 20 --strata 1,2,3` on three ranges of route seeds:

| seeds | stratum | deaths | notables / run | longest quiet (median) | blind e / min | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 1–20 | S1 | 30 % | 0.50 | 40 s | 10.8 | pull fast · loss ok · death ok |
| 1–20 | S2 | 15 % | 0.65 | 40 s | 30.2 | pull fast · loss ok · death ok |
| 1–20 | S3 | 20 % | 2.70 | 40 s | 28.5 | pull fast · loss ok · death ok |
| 101–120 | S1 | 5 % | 0.65 | 40 s | 14.5 | pull fast · loss ok · death corridor |
| 101–120 | S2 | 10 % | 0.75 | 40 s | 30.5 | pull fast · loss ok · death ok |
| 101–120 | S3 | 20 % | 3.35 | 40 s | 41.6 | pull fast · loss ok · death ok |
| 201–220 | S1 | 25 % | 0.45 | 40 s | 10.4 | pull fast · loss ok · death ok |
| 201–220 | S2 | 35 % | 0.60 | 40 s | 31.1 | pull fast · loss ok · death wall |
| 201–220 | S3 | 30 % | 3.05 | 40 s | 44.8 | pull fast · loss ok · death ok |

Before (`telemetry/cadence.md` at 461b80c, seeds 1–20): notables 0.00 / 0.00 / 2.80, longest quiet 46 s / 8 s / 62 s,
deaths 15 / 20 / 25 %, "loss lapses" in S1 and S3. Every stratum's quiet is now at the budget's 40 s, and notables are
over the one-in-three in S1–S2 on every range. **Not met**: deaths left their 10–30 % band on two of the nine (S1 5 %
on 101–120, S2 35 % on 201–220), so the DoD's "deaths still inside their bands" holds on seven of nine and the row stays
in REVIEW; "pull fast" is the standing source-density finding (Expedition mode: the cadence budget), not this row's.

## PAC-107 (part 1) — movers between ticks, kinetic labels, the sound channel (2026-11-15)

**Movers between ticks** (OVR-080): `WorldView` keeps each slimo's and each packet's last step by id
(`RoseSim.agent_ids`, `packet_ids`, the packet's shipment) and draws it between the tile it left and the tile it
entered over its step's time on the picture's clock (`Sim.anim_ms`): a slimo over two ticks, a packet over one, in
whole pixels, ooze-time slowing it with the clock; a jump of more than a tile lands at once. While anything glides the
crowd and the cargo redraw each frame. The screenshots and the played session hold every stage and beat with it, and
the Colony's shots changed (the movers stand where they glide to). **Not checked**: a shot at half a step, and frame
time at a late city (PAC-103).

**Kinetic labels** (OVR-081): `ScreenBase.slam(label, text, colour, frame)`: frame 0 at twice the size and scrambled
in Hot Pink, frames 1 and 2 scrambled a pixel off (Hot Pink, then Cyan), frame 3 the text where it stood in its colour,
a frame each 33 ms on the picture's clock, the panel behind lit for the four. Used where a moment lands: the Colony's
new line for the city's newest landing or loss (shown four seconds under the chapter's card, hidden otherwise, so it
takes no room at rest), Ivan's line and the chapter's card when they change (not when the screen opens), a pull's title
on a run, the haul on the Stash. The smoke checks the slam's frames 0, 1 and 3. The screenshot scripts turn it off
(`Sim.kinetic`): their city ticks every frame, so a line lands every frame, and a shot of a glitch frame showed
scrambled text (seen and fixed: the first shots of this pass had Ivan's box in glitch glyphs).

**The sound channel** (OVR-082): the `Audio` autoload makes Music, World, UI and Voice under Master; twelve slots,
each a made file under `res://assets/sound/<id>.wav|.ogg` or a placeholder tone from its recipe (square, triangle or
seeded noise, a hard envelope, 22 kHz 16-bit). The city's landings and losses play the reward's blip, the big one for a
chapter, a trophy, a stratum or an ending, and the loss's fall, one a quarter second at most; a run's pull, notable,
extraction, hit, raiders and stir play theirs. Settings sets each bus in steps of ten, kept in `user://settings.cfg`.
The smoke: five buses, twelve slots each a stream (none made, all placeholders), a reward played, an unknown slot
refused. The container has no sound device, so no sound was heard: Godot falls back to its dummy driver here.

**Not done** (PAC-107 stays open): room for the world (OVR-083: the Colony at two thirds of the frame with a card open).

## PAC-104 — balance as data; the campaign on three seeds (2026-11-15)

**`data/balance.toml`** (OVR-050): the `balance` crate holds the economy's knobs and `game::data` loads the file as its
eleventh, installing it before any reader asks (`balance::get()` in `mat`, `people`, `colony`, `economy`, `game`):
recruitment (the flush's batch, its cadence, the census's, the tier's batches, the beds' draw), food (the larder's
share, a meal's take, paste's price), the Mat (translocation's share and ends) and the economy (the per-street stock,
PAC-105). Each value keeps the reasons the code kept beside it, as comments in the file. With the shipped values as
the constants were, `rose city`'s report came out identical but for its log's path. The structures' Parts costs were
already data. **Not moved**: the work cycle (`WORK_TICKS`, `SHIFT_TICKS` are compile-time constants used across the
crates and the bridge). A value edited in a copy of `data/` moves a proof with `rose --data DIR` and no build: every
loot, stir and meal sweep of PAC-105 and PAC-106 ran that way.

**Three seeds** (OVR-052): `rose campaign --seeds 12,13` plays every script on each seed after `--seed` and writes the
city's hours by script and seed and each script's range (`docs/03/telemetry/campaign_seeds.md`, 2,409 s on one core).
Seed 11 alone stays the standing proof (`campaign.md`, PASS). The three-seed run FAILS the proof: on seed 12 the
Garden script's Heart burned in chapter 3's Cleansing at shift 45 (the accord accepted, its flamers through the
defences), and the Accord script ended in the Garden. Neither seed was played before this pass, so whether either was
so before it is not known.

## PAC-105 (part 1) — pacing: the late Parts, the single climb, chapter 6, the Hamlet's food (2026-11-15)

Traced first (OVR-060), with prints in the steward that are removed again: its bed decisions shift by shift, Ivan's
place and the factions' standing through chapter 6, and the Mat round the cut climb tile by tile.

- **The late Parts** (OVR-062): from shift 69 the stalls in reach held no Parts at all while 20,000 eddies sat unspent.
  A stall at every street: a faction's day of stock again for each further street of its in reach (`street_stock` 1.0).
- **The single climb** (OVR-063): the cut stood at the Sump's crossing beside joined tissue at 0.96 nutrient; every
  empty tile in the gap touched three strands, and the sprout's neighbour limit refused each. The climb mends: ground
  between tissue the Heart reaches and the city's own tissue cut from it may be grown into whatever it touches. With
  the per-street stock and no mend, the Garden script's climb stood cut from shift 60 to the end of 220; with the mend
  it rejoined and the script ended at 113.
- **A stage past its time** (OVR-065): the World-Eater script's Great Cleansing left its lamps among the Bloom's
  tissue with no way back up their grates, and chapter 7 waited ninety shifts. A squad still on the map past its
  stage's time and a leave's five minutes has gone by another way.
- **Chapter 6** (OVR-066, the steward's): Ivan was ordered onto the tile under a grate, often no Mat above the Sump,
  and refused; the Garden and Accord scripts waited 43–44 shifts for their alliance. Now to the Mat within his
  Parley's reach: the Garden's alliance at shift 69, not 118.
- **The Hamlet's food** (OVR-061): a meal takes 0.03 nutrient, not 0.05; the steward's Hamlet sat at 56–92 sleepers
  for nine shifts with one in five on paste, and the World-Eater's City of 1,600 ate 24,000 eddies in paste.

The runs between: Ivan's walk alone moved nothing past the Empire (the end waits on 2,000 sleepers); the per-street
stock alone brought Hive, World-Eater and Accord to 13.8, 16.1 and 14.3 h and left the Garden's climb cut; with the
mend Garden ended at 16.1 h and the World-Eater's Great Cleansing never left (OVR-065 found there).

**The campaign at the end of this pass** (`rose campaign`, seed 11: PASS, every script its own ending; and seeds
12–13):

| script | seed 11: Quarter | seed 11: the end | Quarter, seeds 11–13 | the end, seeds 11–13 | inside 12–18 h |
| --- | --- | --- | --- | --- | --- |
| Garden | 5.5 h | 18.6 h | 3.8–5.5 h | 15.9–18.6 h (seed 12: Scattered at 6.4 h) | 1 of 3 |
| Hive | 4.5 h | 14.6 h | 3.7–6.8 h | 14.6–20.7 h | 2 of 3 |
| World-Eater | 6.9 h | 19.9 h | 6.1–8.7 h | 18.8–22.6 h | 0 of 3 |
| Accord | 6.2 h | 15.0 h | 5.0–12.6 h | 15.0–20.9 h (seed 12: ended Garden) | 2 of 3 |

Against the start of this pass (seed 11 on the game's shift: Garden 20.8 h, Hive 19.1 h, World-Eater 16.3 h, Accord
22.2 h; the Quarter at 6.1 / 4.5 / 6.1 / 5.3 h): on seed 11 three of the four end inside 12–18 h or within an hour of
it (Garden 18.6), the World-Eater runs 3.6 h longer (19.9), and the Hamlet comes at 1.0–1.4 h, not 0.6. **Not met**
(the row stays open): every ending inside 12–18 h on three seeds (5 of 12); the Quarter at 4–5.5 h (3 of 12: early on seed 13, late on
seed 12 and for the World-Eater on every seed); seed 12
is slow from its first hour (the Hamlet at 2.7–3.2 h) and two of its scripts miss their endings. What is not yet
traced: why seed 12's Hamlet comes three hours in, why the World-Eater's Empire comes last on every seed, and the
Hamlet's 0.6 h to 1.0–1.4 h on seed 11 (the stir and the bot's bleeding-home rule change every run the city's Runner
makes, and with it the haul the first beds are built from).
