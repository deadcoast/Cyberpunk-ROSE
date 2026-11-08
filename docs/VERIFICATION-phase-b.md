# VERIFICATION — Phase B (the run), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-020 — `run` crate: chunk format and seeded assembly (2026-11-08)

**What exists.** `core/crates/run` with three modules: `chunk` (32 × 24 glyph grids loaded from `data/chunks.toml`,
validated on load), `assemble` (seeded placement on a chunk grid by matching opposite exits), `map` (the compiled
`RouteMap`: one substrate grid in `mat::sim::sub` codes plus containers, bodies, spawns, patrol points, lights,
hazards, the Mat edge and the grates). `data/chunks.toml` holds 19 authored chunks (S1 Sump 6, S2 Culverts 6, S3 Storm
Drains 7) written by `tools/gen/gen_chunks.py`, which is also where the glyph legend lives. The bridge exposes
`RoseRun`; `render::route_view` draws a route; `rose route` assembles, renders and sweeps; the Run screen plays on
the assembled route with the Mat edge as start and extraction, props on their tiles, and F-to-dig through collapses
and deposits.

**Decisions taken here, recorded for the later PACs.**

- Traversal: slimos are ooze and crawl any surface, so a Runner moves in four directions through every passable
  tile of the side-on cross-section; shafts need no ladders. Gravity is a rule for dropped items and bodies only.
- Exit windows are canonical (rows 11–13 on the west and east edges, columns 14–17 on the north and south), so any
  two facing exits join. Unjoined exits compile to brick.
- A chunk's `weight` is both its pick weight and its cap per route while alternatives remain (a vault once, a
  culvert up to three times); the cap lifts only when every fitting chunk is at it, and repeats are damped either
  way. With 19 chunks, repeats still happen; the design target of 7 per stratum (42) removes most of them.
- Nothing is ever placed west of the entry column: that is where the Mat is.
- Validation rejects: partial exit windows, exit glyphs off the border, open border cells outside exit windows,
  entry chunks without an east exit or with a non-zero weight, and any chunk whose exits do not all reach one
  another without digging. Sealed pockets behind rubble (the S3 vault) are allowed inside a chunk; the route-level
  rule is ≥ 95 % of passable tiles reachable from the Mat edge without digging and 100 % once every collapse is dug.
- `mat::sim::sub::COLLAPSE = 8` was added (conductance 0, not growable) so a route grid is a valid Mat substrate for
  the "Mat grows into a sealed room" rule later.
- The headless render palette (`render::pal`) still carried the Phase-A green Mat; it is now the SEEPAGE v2 Ooze
  ramp, matching the game.

**Automated.**

`cargo test -p run` — 12 tests: the shipped library parses with three strata and an entry per stratum; exit glyphs
only in whole windows on the border; a bad-chunk fixture is rejected for a partial window, an open border cell and an
exit sealed behind rubble; route length stays in 8–14 for 500 seeds; 200 seeds × 3 strata assemble exactly the
seeded count with the entry at (0,0) and nothing west of it; same seed → identical layout, different seed → different;
joins are mutual and face each other; 150 seeds × 3 strata compile with connectivity ≥ 95 %, every chunk reachable,
100 % reachable once dug, three entry tiles passable, extraction tiles non-empty; sealed exits are brick and joined
exits open; containers carry the chunk tier (lockers = stratum, crates = stratum + 1) and sit on passable tiles;
digging opens a collapse and removes its hazard. Workspace total: 38 tests, 0 failures.

`rose route --seed 11 --stratum 1 --sweep 200` — S1 10.7 chunks avg, 11.8 containers, 9.7 spawns, reach min 100 %;
S2 10.7 / 15.2 / 15.2 / 100 %; S3 10.7 / 12.0 / 18.7 / 98.3 %; 0.07 ms per route in release.

Godot headless smoke — `RoseRun` present; Run: route of 8–14 chunks from a library of ≥ 14, Runner starts on the Mat
edge, > 2,000 route tiles and ≥ 8 props painted, connectivity ≥ 95 %, the Runner steps east off the Mat, a collapse
digs open. PASS in 1.2 s with every other Phase-A check still green.

**On the screenshots.** `docs/shots/run.png`: the Mat edge in ooze cords with the Runner on it, brick-lined culvert,
water on the floor line, lockers and crates on their tiles, every element a placeholder in the one drawing system.
`docs/shots/route_map_s1_seed11.png` and `route_map_s3_seed7.png`: whole routes at 4 px per tile — chunk seams, the
ooze mark at the entry, light pools in Flood Light, containers in pink and gold, bodies in Bruise, patrol points in
Vestment White, gas in Pond Scum; unreached pockets dimmed.

**Known gaps, by design of the PAC.** Containers open nothing yet (PAC-022); burrows spawn nothing (PAC-023); the dig
is 1.2 s not 4–10 s until digging has a loot payoff; extraction is a 3 s channel to the Stash with no followers, toll
or corpse record (PAC-024); no fog or light cone (PAC-021). Chunks are authored in `gen_chunks.py`, not Godot's tile
editor as the design states; the TOML is the contract either way and the editor export lands with Phase F tooling.

## PAC-021 — `run` crate: fog, light and noise fields (2026-11-08)

**Research first.** `docs/research/07-light-noise-fog.md` reads Thief's light gem and sensory paper, Brogue's
stealth ranges and lighting, Minecraft's 0–15 propagation, Ford's symmetric shadowcasting, Mark of the Ninja's sound
rings, Hunt: Showdown's sound traps, Project Zomboid's sneaking multipliers, Tarkov's surface and occlusion reads and
Darkwood's visibility, and ends in ten decisions; every number ROSE adds is marked ours there.

**What exists.** `run::fov` (symmetric shadowcasting, exact rational slopes), `run::light` (0–15 field, 8-connected
Dijkstra with integer costs, water ×3, solids take light and stop it, static pass once + dynamic pass per tick),
`run::noise` (footprints by substrate cost — open 2, pipe 1, brick 3, water 4, rubble 4, rock blocks — decay 2 per
tick, event ring for the AI, the loudness table and surface adds), `run::fog` (unseen / remembered / visible, memory
bitset), `run::senses` (lume, exposure, notice range, the 20 Hz tick, the overlay packing). The save format is now
version 2 with `route_memory` per (seed, stratum) and a v1 migration. The bridge exposes all of it on `RoseRun`
plus `RoseSim.gene(i)` for the strain's Lume; `rose route` prints the authored light bands and a lantern-on /
dimmed comparison and writes `route_light_*.png`. In Godot, `fog.gdshader` draws fog, light and noise as a 1-bit
Bayer dither in Void and Cyan at the art's pixel grid from one RGB8 texel per tile; the Run screen ticks the senses
at 20 Hz, has sneak (Ctrl), sprint (Shift), dim veins (hold Q, stamina), lantern (L) and shows the light gem as
numbers until PAC-025 draws the arcs.

**Automated.** `cargo test -p run` — 28 tests (16 new): FOV open room, wall occlusion, radius cap, and symmetry over
every pair of floor tiles in a fixture; fixture pools fade one step per tile and stop at rock (< ¼ of rock lit by one
fixture); Mat glow 2 at gene 0 and 8 at gene 1; dynamic sources combine by max and restore the static field exactly;
water costs three times air; loudness table (sprint 7, gunshot 14, clamp 15) and surface adds; footprint 10 → 9 → 8
down a corridor, no noise in rock, decay empties the field in 5 ticks, events kept; synthetic corridors reach 10
(open) / 5 (water) / 19 (pipe) tiles; fog states progress and the memory bitset round-trips, a mismatched size is
ignored; lume adds, dims, halves when Soaked, clamps at 15 and floors at 1; notice range 3/7/14 at exposure 1/5/12,
+2 sprinting; a lantern Runner sees more than twice the tiles of a dimmed one and is lit; the Mat edge lights a
Runner standing on it; a sprint step reaches > 20 tiles, overlay packs RGB per tile, memory survives a rebuild.
`cargo test -p save` — route memory stored, replaced and read back; a hand-built version-1 body decodes and
migrates. Workspace: 55 tests, 0 failures.

`rose route --seed 11 --stratum 1` — authored light: 92 % dark, 7 % dim, 0 % lit of passable tiles (dark by
default holds); lantern on eight tiles in: sees 166 tiles, exposure 8 lit, noticed within 9; lantern off and
dimmed: sees 43, exposure 1 dark, noticed within 2; a sprint step there reaches 83 tiles.

Godot headless smoke — on the Run: > 20 tiles visible at start, memory bitset sized ⌈w·h/8⌉, six steps east leave
the Mat, lantern on reads "lit" with exposure ≥ 7, lantern off lowers exposure and shrinks sight, a sprint step's
footprint exceeds 20 tiles with noise ≥ 7 under the Runner, six ticks later it is 0, remembered tiles outnumber
visible ones after moving, the fog overlay is present. PASS with every earlier check green.

**On the screenshots.** `docs/shots/run.png`: the fog overlay over the route — Void beyond what has been seen, heavy
dither on remembered tiles, the lantern's pool open around the Runner, lit wall faces reading through the dither.
`docs/shots/route_light_s1_seed11.png`: the whole route under its authored light with the lantern pool and a sprint
step's cyan footprint eight tiles in.

**Known gaps, by design of the PAC.** Nothing listens yet: hearing thresholds (sewerat 2, scout 4) and the awareness
capacitor are PAC-023's. The gear light comes from a lantern toggle, not the rig (PAC-026 wires gear). Route memory
is produced and restorable but not yet written to the save by the shell (PAC-026). The HUD light gem is text; the arcs
are PAC-025. Authored sound traps (chains, loose grates) are a later chunk-authoring pass.

## PAC-022 — `run` crate: loot tables and containers (2026-11-08)

**Research first.** `docs/research/08-loot-tables.md`: Diablo 2's treasure classes (pool first, item second, quality
last, level gates the top), Dota 2's pseudo-random distribution (C·N ramp, reset on hit, "to limit the influence of
huge streaks"), Risk of Rain 2's 79/20/1 chest, Tarkov's 12.6 spawn randomisation (where is learnable, whether is
not), Darkest Dungeon's curios (a supply item buys certainty), plus research 01 and 02. Eleven decisions; every
number ROSE adds is marked ours and lives in `data/loot.toml`.

**What exists.** `run::loot`: tables from `data/loot.toml` (seven sources — locker, crate, deposit, newcomer corpse,
scout drop, carrier drop, cache — each with picks, NoDrop, condition band, category weights and six tier rows),
D2-style resolution, PRD on notable picks with the constant solved by bisection (matches Dota's published table to
±0.2 %), first-run generosity on the first three opens, the S1–S2 cap that keeps S2 at Uncommon, pry-bar versus
hands (noise, an extra pick on crates, forced lockers damage half their contents), piles from dug deposits,
corpses as sources, Oathkeeper drop tables ready for PAC-023, and taking into a `Rig` by auto-place. Contents roll
from (route seed, run nonce, source index). The bridge adds `begin_run`, `sources`, `source_at`, `open_source`,
`source_items`, `take`, `take_all`, `rig_*`, `rig_export`, `loot_stats` on `RoseRun` and `import_rig` on
`RoseStash`; `Sim` carries the run hand-off (nonce, per-route run count, pending haul). The Run screen shows the
source prompt under the Runner, opens with E (pry bar when carried), unrolls the manifest one line per 0.22 s with
the notable last on a longer beat, takes all with F, shows the rig line (items, free cells, weight, blind value), and
hands the rig to the Stash on extraction; the Stash reports the intake. `rose route --loot N` sweeps the tables.

**Automated.** `cargo test -p run` — 35 tests (7 new): PRD constants for 5/10/25/30 % within 0.002 of Dota's
0.38/1.5/8.5/12 %; sources = containers + corpses and 20–40 % of lockers locked over 60 routes; S1 and S2 lockers
roll no Rare over 40 routes each while S3 does; per-container value climbs ×1.4–3.2 per stratum; generosity flags
exactly the first three opens of a first run and never on a second; same seed/nonce/index gives identical
contents, a new nonce differs; pry is quiet and yields ≥ hands, a forced locked locker damages ⌈n/2⌉ items and a
pried one none; piles are Parts/Chrome at 40–80, corpses ≤ 60, scout drops are all Blackline plus vestment cloth
10–40 % of the time, carriers drop the lamp pole; 400 locker rolls keep the dry streak under the PRD bound with a
notable rate in 6–20 %; take_all fills the rig and leaves the rest. Workspace: 62 tests, 0 failures.

`rose route --loot 200` — per container, pry bar, not a first run: S1 mean 9.4 e (p50 7, p90 21), 68/32 C/U;
S2 19.0 (×2.02), 44/56, no Rare; S3 37.2 (×1.95), 38/45/14/3, notable 16.6 %, longest dry streak 25.

Godot headless smoke — ≥ 8 sources on the route; six opened in place with `source_at` agreeing with `sources()`,
items revealed, taken into the rig, every taken item unidentified with a hidden name, rig weight and blind value
read, the exported rig imports into a fresh Stash with count = taken − left, and a dug deposit adds a pile. PASS.

**On the screenshots.** `docs/shots/run_loot.png`: the Runner at a tier-1 locker with the panel open — one line,
tier colour, "unidentified common chrome 1x1 0.6kg c75", F take all.

**Known gaps, by design of the PAC.** The pry bar is a kit flag, not a rig slot (PAC-026 wires gear and the Rig
screen); quick-slot use, dropping and the body pickup are PAC-024/026; the manifest has no stinger sound or tier
colour wash yet (PAC-025/028); the loot panel is a label list, the grid-to-grid panel of the design lands with the
Run HUD; scout and carrier drop tables wait for kills (PAC-023); caches wait for Sever rooms and S5 vaults.

## PAC-023 — sewerats and scouts AI, combat, body sections (2026-11-08)

Full Design v3.1 came first: Expedition mode now carries every number Phase B fixed, PAC-023's included. Research
09 (Thief's awareness capacitor, Metal Gear's phases, Tarkov's limbs) is short and its decisions are in the design.

The run is now one Rust simulation, `run::sim::RunSim`, stepped at 20 Hz from the input word the save logs; the
Godot Run screen is a controller and view. Creatures (`run::creature`, `data/creatures.toml`): awareness with
sight by distance in the creature's own FOV and sound as a floor, phases Calm / Suspicious / Alert / Search / Flee /
Leaving, distance-field pathing (scouts over floors only, ferals over everything), bites with wind-ups, bursts,
rifle shots with magazines, patrols that map and leave, salt lines ferals will not cross. The Runner
(`run::runner`): four sections with Tarkov's transfer rule, statuses, stamina, actions with wind-ups and buffered
presses, poses — walk on a floor, crawl the back wall or ceiling at ×0.7 (no sprint), wade shallow water at ×0.75,
swim deep water at sneak speed and Soaked. 47 tests in the crate; smoke plays open → take → fight → extract →
Stash through the input word; `game/tests/play.gd` plays a scripted run under a display and saves
`docs/shots/play_*.png`, which were reviewed by eye and fixed from: a Runner floating mid-corridor (poses), a fleeing
crawler regenerating beside the Runner (flee away, recover out of notice), presses dropped between ticks and during
actions (latching and buffering), the extraction hold refused mid-swing (hold, not press).

Known gaps, by design of the PAC: followers, the corpse record, raider rolls and Spored regrowth (PAC-024); the HUD
is text and the rig is a kit (PAC-025/026); lamp carriers and raiders (Phase D); the `[runner]` numbers are the
first tuning and the loot/creature sweeps will be read together when the shift clock lands.

## PAC-024 — extraction with followers, the corpse record, the raider roll, Mat recovery, Spored regrowth (2026-11-08)

Full Design v3.2 first; research 10 is three sources and five decisions. Followers (`run::follower`) take Follow /
Hold / Guard, fight on their own, pick up a downed Runner within six tiles and carry them toward the nearest
extraction tile at 0.6× for twenty tiles; a carrier on an extraction tile holds the channel for both. Creatures hunt
the nearest target they can see, Runner or follower. The run ends in an `Outcome` (rig or corpse, followers
extracted / stranded / dead, Mat haul, regrown Spored gear, kills, chunks seen, route memory); the save is v3 with
`corpses`, and a re-run of the route places each body after a per-day raider roll (10 % + 5 % per stratum). The
Godot `Sim` autoload takes the outcome, advances the day and feeds the Stash intake. 51 tests in the crate; smoke
checks followers, orders and the outcome; the play frames show the followers trailing and standing at the Mat edge
during the channel. Reviewed and fixed from the frames: a press queued behind a swing used to expire (interactions
now buffer two seconds).

Known gaps: followers are a fixed pair with fists until the colony hands over real slimos and their gear (Phase C);
stranded followers are recorded, not yet placed on the next run; a body on the route is not yet shown on the route
card (PAC-025).
