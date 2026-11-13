# VERIFICATION — Phase A (foundations), run on 2026-11-07

Environment: Linux (kernel 6.18), 2 vCPUs, Rust 1.97.0 release profile, Godot 4.5.1 stable headless and under
Xvfb (Mesa software GL). Numbers are from this machine.

## Automated (`./run.sh /path/to/godot`)

`cargo test --release` — 26 tests across five crates, all passing.

| crate | tests | what they hold |
| --- | --- | --- |
| mat | 10 | determinism (same seed, same digest); nutrient bounded and finite; fed growth and reach; Runner reaches farther than default; die-back when unfed; transport conserves mass on a chain; salt blocks growth; PCG32 stream and bounds |
| logistics | 2 | 3,000 agents deliver; packets never exceed cord capacity; city deterministic |
| items | 6 | 164 items load with every category at its designed count; gear has gear blocks, relics unsellable, chrome has feed and bias; pricing formula (condition, glut, found-in-run, blind sale); appraisal bars by tier; feed value and condition penalty; sets complete at three |
| inventory | 4 | footprint and rotation placement, overlap and bounds refused; auto-place fills and reports no room; Stash upgrade keeps items, Vault hides contraband; rig weight and overweight |
| save | 4 | items round-trip through encode/decode with pins, provenance, containers, inputs and snapshots; corrupt and foreign bytes rejected; strain codes round-trip and reject a bad checksum; snapshot trimming |

`rose-tools all` — 60 manifest slots + 164 expanded item icons → 259 placeholder files, every slot and state covered;
prompt sheet written; contact sheet 1192x1250; validator: 0 art files (none yet), 0 failures.

`rose scale --agents 3000` — Mat 1,506 cells, 309 nodes; 0.5 ms/tick CA, 0.72 ms/tick with 3,000 slimos and cargo.

Godot headless smoke (`game/tests/smoke.gd`) — extension classes RoseSim, RoseItems, RoseStash present; 14 screens
instantiate; 223 textures resolved through slots, none missing; Colony: sim advanced past tick 500, Mat > 500 cells,
world and Mat TileMapLayers painted, view switched to Stack, a tool applied; Stash: starting haul of 8 placed, 3 more
added, save round trip through the Rust save crate true, upgrade to 16x14 with every item kept; Run: the Runner moved
a tile. PASS in 1.2 s.

Screenshots (`game/tests/shots.gd` under Xvfb) — 15 PNGs at native 640x360 in docs/shots/: every screen plus the
Colony at Stack zoom.

## Manual, on the screenshots

| check | observed |
| --- | --- |
| one drawing system | every element on every screen is a 32-px-grid placeholder with the dashed Void border, a ramp fill and a tag; UI frames are clean nine-patches; text is the 3x5 bitmap font from the same sheet the placeholders use |
| ooze, not green | the Mat is the Ooze ramp on every tile; green appears only on water placeholders (Bile) |
| the Stash is real | eight items from `data/items.toml` placed by the inventory crate with their footprints; the card shows sell and feed numbers computed in Rust |
| the Colony is the sim | 1,500+ Mat tiles painted from `RoseSim.hyphae()`, cords recoloured by signal state, crowd drawn from agent arrays |
| text legible at 3x | 5-px cap height reads at 1080p; non-ASCII glyphs removed from UI strings (the tool font has none) |

## Known gaps, by design of the phase

The Sell / Feed / Craft verbs on the item card are buttons without actions until Phase B; the Run has movement, HUD
and extraction only; eleven screens are laid out on sample data. The bundled PACER validator rejects our phase
names because its phase whitelist is from a sample project; the register's own rules (unique IDs, blockers exist,
DONE gate, acyclic) were checked directly and hold.
