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
