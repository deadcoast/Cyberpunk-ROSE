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
