# Current state — 2026-11-13

MIVIM-PACER `current_state`, reviewed at commit `e344396` (Phases A–E done, PAC-001 to PAC-083; Phase F art open).
Every number below was measured for this review or read from a proof's report on this commit; where a figure is an
estimate it says so. Items marked *proposal* are this review's own ideas, not the design's. The items are numbered
for the register: each one maps to a Phase G row in `todo.PACER.csv` (the last column of every list).

Measured with: `cargo test --release --workspace --exclude rose-godot`; `rose city | cleansing | economy | campaign`;
`rose bench`; `rose campaign --aim world_eater` timed shift by shift; `rose cadence` (report of 2026-11-12); the
headless smoke; `game/tests/shots.gd` and `game/tests/play.gd` under Xvfb (frames in `assets/shots/`); a source read
of every crate's entry points and the colony and run screens.

1 The build in numbers
----------------------

| measure | value |
| --- | --- |
| Rust, production lines (tests excluded) | 34,239 in 16 core crates, the bridge and the tools |
| GDScript | 8,699 lines in 29 files (1,604 of them the smoke, shots and play tests) |
| Tests | 192 Rust tests (191 run, 1 ignored), all passing; `run` 62, `game` 32, `colony` 18, `mat` 12 |
| Untested code | the bridge (5,438 lines, 269 functions exposed to Godot), `cli`, `render`, the tools: 0 tests each |
| Proofs | `rose city`, `cleansing`, `economy`, `campaign` all PASS; the campaign takes 1,432 s on this machine |
| Content against the launch scope | at scope: 164 items, 42 chunks (7 a stratum), 60 recipes, 36 tech nodes, 40 traits, 8 laws, 379 barks |
| Asset slots | 177 in `game/assets/manifest/`, plus 164 item icons; 0 PNGs in `art/` or `anchors/` |
| Simulation cost, headless | 0.6 ms per city second at 26 slimos; about 10 ms at 1,726 slimos and 18,511 joined cells (World-Eater, shift 110): about 40 ms of a real second at the top speed, 8 Hz |
| Compiled tuning numbers | 245 `const` items across the core crates |
| Clippy warnings | 20 (8 index loops in `mat`, 8 `is_multiple_of`, 3 long argument lists, 1 `map_or`) |

The campaign (`docs/03/telemetry/campaign.md`) against Progression's table (Hamlet 1 h, Quarter 5 h, City 12 h,
Empire 18 h; a first run of 12–18 hours):

| script | Quarter (100) | City (500) | Empire (2,000) | ending |
| --- | --- | --- | --- | --- |
| Garden | 6.1 h | 10.5 h | 25.6 h | 26.7 h |
| Hive | 4.5 h | 7.8 h | 18.2 h | 19.1 h |
| World-Eater | 6.1 h | 9.7 h | 15.6 h | 16.9 h |
| Accord | 5.3 h | 9.5 h | 24.5 h | 25.5 h |

The run (`docs/03/telemetry/cadence.md`, 20 bot runs a stratum): deaths 15 / 20 / 25 % in S1 / S2 / S3 (budget
10–30 %); a pull every 10–14 s at the bot's pace, twice as often as the budget's 20–40 s (the bot taken as three
times a player); notables 0.00 a run in S1 and S2 (budget one in three runs); the longest quiet stretch 46 s and
62 s in S1 and S3 (budget under 40 s at the bot's pace, two minutes at a player's).

2 Design against build
----------------------

| the design asks (section) | state | evidence |
| --- | --- | --- |
| The master loop, steps 1–8 (The game) | met | `play.gd` walks prepare, run, haul, decide, build, breed, withstand and rise in one session; 34 frames in `assets/shots/play_*` |
| Pillar 1, Reward: a pull every 60–120 s, a notable a run from S3 and one in three runs in S1–S2 | partial | pulls twice as frequent as the budget; S3 notables 2.8 a run; S1–S2 notables 0.00 a run, a known miss the design left to "the loot pass" (Expedition mode: the cadence budget) |
| Pillar 1, the feedback stack: light, a kinetic label and a sound on every reward | gap | no audio anywhere in `game/`; `ScreenBase.make_label` (`game/ui/ScreenBase.gd:96`) is a plain Label, no slam or glitch |
| Pillar 2, Ownership: what a session leaves behind, visible next time it loads | gap | nothing is saved: the menu's CONTINUE (`game/modes/menu/Menu.gd:21`) shows the session's city, and quitting loses the campaign (Game shell: "until saves are on disk Continue is the city this session holds") |
| Pillar 3, Automation: quotas, cargo routing, loops, Nerve rules, the handoff | met | the campaign scripts use every one; `play.gd` frames 22–24 |
| Pillar 4, Self-governed: work cycles, foremen, petitions, unrest, laws | met | the living city runs a work cycle every minute (PAC-082/083) |
| Pillar 5, Risk vs reward: deaths 10–30 %, loss never lapses for two minutes | partial | deaths 15–25 %; pressure lapses in S1 and S3, the longest quiet 46 s and 62 s at the bot's pace, over two minutes at a player's |
| Pillar 6, Progression: strata, tech, classes, tiers, Authority, meta | met, slow | every system plays; the campaign runs 17–27 hours against 12–18 |
| Scale: 2,000–5,000 slimos at Empire, the sim in budget | met headless | about 10 ms a city second at 1,726 slimos; the live game can only show a city as far as one session plays it (NF-01, NF-04) |
| Three views from the same arrays | met | Street, District, Stack in `assets/shots/colony*.png`; the Stack frames the strata reached (v3.19) |
| Movers interpolated between ticks (Pass 2 budget: 2 Hz sim with per-frame interpolation) | gap | a slimo is drawn at its tile (`game/modes/colony/WorldView.gd:512`) and cargo at its tile (`:305`): both jump a tile a tick |
| Saves: autosave every shift, three slots, seed + inputs + snapshots | gap | the `save` crate round-trips a file in a test (`bridge/rose-godot/src/lib.rs:2001`); the game never writes one |
| Settings: video, audio, controls, gameplay, accessibility | gap | fullscreen and three speed buttons work; the rest are labels (`game/modes/settings/Settings.gd:10`), and the screen prints "seed 11 … save: autosave each shift" as fixed text (`:39`) |
| Controls: gamepad parity; Achievements: 40 | gap | no joypad input or achievement code in `game/` or the crates |
| UI: one alert a second, labels under 24 characters, kinetic labels | partial | alerts paced (`Colony.gd`, ALERT_EVERY); labels plain |
| Phase F: parallax, water, lighting, audio, accessibility palettes (PAC-093) | open | waits on the art (PAC-090–092) |

3 What holds up
---------------

- **Rules in one place.** Every rule and number runs in Rust; Godot reads arrays. The `mat` crate has no `unwrap`
  outside its tests and iterates its Pits in sorted order, so a seed replays; `rose city` asserts it replays bit-exact.
- **The whole game plays headless.** Four proofs drive a bot through the run, the city, the threats, the market and the
  campaign; that is how PAC-083's regressions (a starved climb, a fed Bloom, a burned Heart, Ivan walking into a
  Cleansing) were found at all.
- **Content is at its launch counts** and lives in `data/*.toml`.
- **The art pipeline is ready to receive art.** Manifest, generated placeholders, a validator (size, palette, outline)
  and a prompts generator that now writes `docs/03/pixellab-prompts.md` in the hand-edited format byte for byte.
- **The sim has headroom.** About 10 ms of simulation a city second at Empire-tier counts, measured.

4 Weak points and improvements
------------------------------

### CORE — the game's own systems

| ID | what | where | why (the mechanism) | verified by | row |
| --- | --- | --- | --- | --- | --- |
| CORE-01 | The late game runs long: Garden and Accord take about a hundred shifts from the City to the Empire and end at 26.7 and 25.5 h | `game::proof` beds, `colony` home costs | In a traced Garden run the beds waited on Parts from shift 120 (14–53 in the Stash against a want of hundreds); where the late Parts go, and what they should come from, is the pass's first trace | campaign ends inside 12–18 h on three seeds | PAC-105 |
| CORE-02 | A Hamlet stalls near sixty sleepers for ten shifts; the Quarter comes at 6.1 h in two scripts | `colony` meals (LARDER_SHARE), `mat` growth | The larder is half of what signalled tissue holds past what it would grow with, and a growing Mat keeps little past that line: World-Eater's larder held 1.4–3 nutrient at shifts 18–30, 27–60 meals for 56–80 sleepers | Quarter at 4–5.5 h on three seeds with no shift of mass paste | PAC-105 |
| CORE-03 | The stack hangs on one climb | the Cleansing's white light, the climb's crossings | In Garden a Cleansing at grate 7 in S4 cut the climb at shift 133: tissue joined to the Heart went 21,576 → 4,010 cells (81 % dark, Gates unpowered) and stayed cut about fifteen shifts; it happened again at shift 180. Hive ends with nothing joined above row 240, Accord above row 179 | joined share after each Cleansing, reported by the campaign proof | PAC-105 |
| CORE-04 | No notable find in S1–S2 | `data/loot.toml` tier rows | Rare enters at tier 3, so the first two strata, the first hours of play, never show the slot-machine peak the reward pillar is built on (0.00 a run, budget 0.33) | `rose cadence` S1–S2 notables ≥ 0.33 a run, value per minute inside its band | PAC-106 |
| CORE-05 | Quiet stretches past budget in S1 and S3 (46 s, 62 s; budget under 40 s at the bot's pace) | the routes in `data/chunks.toml`, the creatures' spawns | not traced yet: `rose cadence --logs DIR` keeps each run's log with every quiet stretch's clock times | `rose cadence` verdicts read "loss ok" for S1–S3 | PAC-106 |

### NF — features the design specifies that are missing

| ID | what | where | why | verified by | row |
| --- | --- | --- | --- | --- | --- |
| NF-01 | Saves on disk: autosave each shift and on run entry and exit, three manual slots, Continue from the menu | `save` crate, the bridge, `Menu.gd` | A 12–18 hour campaign that is lost on quit cannot be played; ownership (Pillar 2) assumes the city is there next time | quit and relaunch resumes the same shift; a save written by one process replays to the same digest in another | PAC-102 |
| NF-02 | Settings that work, and no false text | `Settings.gd` | Of the twenty settings listed, fullscreen and the speed buttons work; "save: autosave each shift" and "seed 11" are fixed strings, untrue to the player | every listed setting changes the game and persists in the profile | PAC-108 |
| NF-03 | The feedback stack: kinetic labels and sound | `ScreenBase.gd`, a new audio bus layout | Pass 2 has every reward hit light, label and sound together; two of the three channels do not exist | a reward fires all three; the smoke checks the bus | PAC-107 |
| NF-04 | A way to see a late city in the game (*proposal*): a developer command that plays a campaign script inside the live game to shift N, or loads a save from `rose campaign` | the bridge, `rose campaign --save` | The game has never been looked at, profiled or art-reviewed past its first hour; Pass 2's acceptance screenshots (a lit crowd, three districts, the Stack's circuit with a white point) need a city at the Quarter or later | frames of a 2,000-slimo city at all three views; frame time at each | PAC-103 |
| NF-05 | Gamepad parity and the 40 achievements | input map, `progress` | listed in the Game shell, not built | a pad plays a run and the colony; achievements unlock and persist | PAC-108 |

### SC — source code changes

| ID | what | where | why | verified by | row |
| --- | --- | --- | --- | --- | --- |
| SC-01 | One composition of the city's tick (*proposal*): the bridge owns a `game::World` instead of rebuilding its own tick | `bridge/rose-godot/src/lib.rs:247–310` against `core/crates/game/src/world.rs:199–224` | Two copies of the same orchestration have drifted: the bridge refreshes the guards' posts every work cycle (`lib.rs:290`), the headless World only when the stewards call `World::movers` (`world.rs:190`), so within a shift its guards stand where the shift began. The proofs prove the World; players play the bridge. Every feature is wired twice (PAC-083 was) | the bridge has no orchestration code left; smoke and play pass; a test steps both and compares | PAC-101 |
| SC-02 | The bridge's state as loaded values, not options | `bridge/rose-godot/src/*.rs` | 114 `unwrap`s in production, nearly all of `table`, `stash` and `recipes` held as `Option` and unwrapped on use; a call before loading panics inside Godot | 0 `unwrap` on bridge state | PAC-101 |
| SC-03 | Bots out of the shipped library (*proposal*): `game::proof` (1,078 lines) and the steward in `game::city` (937) into a `steward` crate used by `cli` and the tests | `core/crates/game/src/proof.rs`, `city.rs` | The GDExtension links two thousand lines of test players; balance changes and bot strategy changes sit in the same crate as the rules | the bridge does not depend on the bots; proofs unchanged | PAC-110 |
| SC-04 | Balance in data (*proposal*): `data/balance.toml` for the economy's knobs (the flush, translocation, the larder, paste, meals, Parts costs, the work cycle) | the 245 constants; the README says "balancing never recompiles" | PAC-083 needed temporary environment hooks to try five values of two constants; tuning should be data, and the proofs should be able to sweep it | a balance change needs no rebuild; `rose campaign --set` sweeps a value | PAC-104 |
| SC-05 | The crowd draws a whole slimo record to read its depth | `WorldView.gd:502` | `people.record()` clones the record and builds a 49-field dictionary for every visible slimo on every tick, to read one string; at a crowded Street view that is thousands of dictionaries a second | depth comes in a packed array beside the positions | PAC-101 |
| SC-06 | Split the four largest files by responsibility | `colony/src/lib.rs` 3,269 lines, `bridge/.../lib.rs` 3,031, `Colony.gd` 1,304, `Run.gd` 1,045 | each holds several systems; a reader or another agent cannot find one without reading the rest | no source file over about 1,200 lines | PAC-110 |
| SC-07 | Generated files followed the old layout and table style — **done in this pass** | `cli` report paths, `rose-tools prompts`, `run.sh`, `shots.gd`, `play.gd` | the proofs wrote `docs/telemetry/` with unspaced separator rows and the prompts tool wrote `docs/pixellab-prompts.md` with `###` headings, undoing the hand edits on every run | the prompts file regenerates byte-identical to the edited one; reports land in `docs/03/telemetry/` | PAC-100 |

### QOL — the player's experience

| ID | what | where | why | verified by | row |
| --- | --- | --- | --- | --- | --- |
| QOL-01 | The colony screen covers about half the world with a card open (estimate from `assets/shots/colony.png`): the two status lines and the chapter card on top, the alert line, tool bar and nav bar below, Ivan's box and the minimap on the right, the card on the left | `Colony.gd` layout | at 640 × 360 the world shows about 20 × 7 tiles between the bars; Ivan's box (about 190 × 50 px) sits empty between barks | a screenshot at each view with the world at two thirds of the frame or more | PAC-107 |
| QOL-02 | Movers jump a tile a tick | `WorldView.gd:305, 512` | At 2 Hz a slimo teleports twice a second; the crowd reads as flicker, not traffic | positions interpolated by the frame's share of the tick | PAC-107 |

### TECH — verification and tooling

| ID | what | where | why | verified by | row |
| --- | --- | --- | --- | --- | --- |
| TECH-01 | Every balance claim rests on seed 11 | `rose campaign`, `city`, `economy` | PAC-083's variants moved the Quarter between 4.5 and 6.7 h and the endings by hours: one seed cannot tell a fix from noise | the campaign proof runs three seeds and reports each tier's range | PAC-104 |
| TECH-02 | The bot is three times a player by assumption | the cadence and campaign hours | The design calls the factor "an assumption until a recorded human run calibrates it"; every hour in the tables above is the bot's seconds times three | the factor stays printed on every report until a human run exists | — |
| TECH-03 | The live game's frame time at scale is unmeasured | `WorldView.gd`, the bridge | the sim is measured (section 1); the drawing of 18,000 Mat tiles and thousands of movers is not | frame time at each view on a late city (NF-04) | PAC-103 |
| TECH-04 | A fast verification tier | `run.sh` | the full run takes over half an hour (the campaign alone 24 minutes); per-commit checks need tests, `rose city` and the smoke only | `run.sh --quick` under five minutes | PAC-110 |
| TECH-05 | Clippy's 20 warnings | `mat`, `run`, `logistics`, `game::cargo` | cheap, and the index loops in `mat` are its hottest code | `cargo clippy` clean | PAC-110 |

### ART — the lane for the pixel-lab-agent

| ID | what | where | why | verified by | row |
| --- | --- | --- | --- | --- | --- |
| ART-01 | World effects drawn as rectangles have no slot, so no art can replace them: cargo pulses, flood water, bleach, the white light, the new-growth outline, district tag paint, Ivan's order line, the run's water and rings | `WorldView.gd` `_draw_cargo` (288), `_draw_threat_fx` (557), `_draw_fresh` (545), `_draw_districts` (429); `Run.gd` `_draw_water` (807), `_draw_rings` (819) | `fx.toml` holds two slots (`vfx.spark`, `vfx.ooze_drip`) and nothing in `game/` draws either | each effect reads a manifest slot with a placeholder; the contact sheet shows them | PAC-109 |
| ART-02 | The agent's contract written down: what it owns, the order, the checks | `AGENTS.md` — **done in this pass** | the pixel-lab-agent arrives without the session's context | the art section of `AGENTS.md` | PAC-100 |
| ART-03 | Acceptance frames need a late city | Pass 2's Visual section | art cannot be judged on the founding's twenty tiles | NF-04's frames | PAC-103 |
| ART-04 | The engine draws any PNG in `art/`, validated or not | `game/autoload/Assets.gd:20` | the manifest README says art is used "when it exists and has passed validation"; nothing checks, so a failed file reaches the screen | the engine skips a slot whose `status` is `placeholder`, or the README says what is true | PAC-109 |

### DOC — the documents

| ID | what | why | row |
| --- | --- | --- | --- |
| DOC-01 | Paths after the move to `docs/01–03` and `assets/shots/` — **done in this pass** in the README, the design, the art bible, research 05, the verification logs and the register | links and the proofs pointed at files that had moved | PAC-100 |
| DOC-02 | `VERIFICATION-1.md` and `VERIFICATION-7.md` both open "Phase 1, run on 2026-11-05" (the old `docs/` and root copies; 7 is the older root file, not a seventh phase) | the numbering reads as phases; the map is in `AGENTS.md` | — |
| DOC-03 | The design's version line is one 4,779-character line, and 17 lines run over 2,000 characters | agents and editors address the design by line; a change history inside the header grows with every pass (*proposal*: move the history to its own file, the user's call) | — |
| DOC-04 | The README's status and facts were stale (Phase C status line, 19 chunks, "jobs finish at shift end", a ten-minute campaign) — **fixed in this pass** | the first file a newcomer reads | PAC-100 |
| DOC-05 | The design's repository block (Technical architecture) describes a tree that is not the one on disk: `game/shaders/` does not exist, the manifest is TOML under `game/assets/manifest/` (not `assets/manifest/*.yaml`), placeholders and art live under `game/assets/`, and "bridge: no logic (the shift lives in `game`)" is untrue while SC-01 stands | an agent reading the design first is sent to files that are not there; the block is the user's to rewrite or to keep as the target | PAC-101 |
| DOC-06 | The register's PAC-063 row had 18 fields: its DoD held unquoted commas, and its DONE edit wrote the dates over BlockedBy and Assignee — **repaired in this pass** from the row as first written (`d5f24e2`) | a reader or a tool shifts every later column of that row | PAC-100 |

5 Next
------

Phase G in `todo.PACER.csv` orders the work: one World first (PAC-101), because saves (PAC-102), the late-city
view (PAC-103) and a bridge that cannot panic all stand on it; balance as data with three-seed proofs (PAC-104)
before any pacing pass (PAC-105, PAC-106), so a fix can be told from noise; the feel items (PAC-107), the shell
(PAC-108), the art lane's effect slots (PAC-109) and code health (PAC-110) after. The art rows (PAC-090–093) belong
to the pixel-lab-agent and can start at any time: they block nothing in Phase G, and only PAC-109 adds slots to them.
