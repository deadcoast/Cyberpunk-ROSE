# VERIFICATION — Phase E (progression and content), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-080 — the tech tree, strain classes, organs, Authority, city tiers, collections (2026-11-11)

Full Design v3.18 first (Progression written to its numbers: research and the 36 nodes, the 12 organs, the 5 classes
and the Strainbook, the handoff, route loops, Nerve logic, Authority's deeds, the city tiers, the collections, scav
yield), research 21 (RimWorld's research benches and techprints, Dead Cells' blueprints: a blueprint opens a tier and
is spent studying it, points are linear in the researcher's skill, the costs come from the pacing table). The tree,
the classes, the Strainbook, the collections and the deeds are data (`data/tech.toml`, `data/strain.toml`,
`data/collections.toml`) read by the new `progress` crate; the organs are the `mat` crate's (`mat::organ`); the
colony, the threats, the run and the economy read the flags. Save v14.

**The founding, put back to the design.** The Heart now founds Pass 1's strain, every gene at 0.5. Until this row it
founded Vigor and Conductance 0.8, a number written nowhere, which by Pass 2's class lines was a Runner from the first
minute and would have handed every city the World-Eater's road for free. At 0.5 a founding takes 69–576 Mat steps and
leaves 107–290 cells over seeds 11–26.

**The city with its progression** — `rose city --shifts 20`, seeds 11–26: every plan and death count as before this
row except on 23 and 24, where the 0.5 strain reaches the first grate more slowly and the cord is lost from shift 3
and shift 6 (plans 2/7 and 4/7; they held the cord before). Seed 11 replays bit-exact.

| what | seeds 11–26, twenty shifts | the design's target |
|---|---|---|
| first organ | on 11 of 16, shifts 7–19 | hour 2, shift 10 |
| a tier 2 open | on 7 of 16, shifts 5–20 (by shift 15 on 6) | tier 2 by shift 15 |
| a node researched | none | — |
| a class held | none | first class hour 7, shift 35 |
| trophies | 1–3 a city (the Flood, the Crawler, the Burster) | — |

No headless city researches a node in twenty shifts: the steward keeps 10 Parts and its Chrome reserve back and never
holds the 6 Parts a tier 2 node costs beyond them. So the headless cities never loop a route or set a Nerve rule
either; those are proven in `game`'s tests and the live game below. PAC-081's twenty-hour campaign is where the
steward's research pace is measured against the table.

**The proofs as they stand**: `rose cleansing --seed 11` PASS (defended, the Heart 200 HP, 0 dead, 8 of 8 fell);
`rose economy` PASS. The Cleansing's aftermath now leaves 42 HP unmended over three shifts where PAC-063 left 15 → 0:
the Turret is the damaged one and its builders sleep or stand Guard. And "the Gates close the way" no longer holds on
seed 11 — the 0.5 Mat lies differently under grate 5. Both registered, not chased: they are the steward's plan, not a
rule.

**In play** (`play.gd`, the live game): a Chrome II blueprint studied opened Chrome tier 2 in 4 shifts; the Sphincter
housing researched 13 shifts later; the Gate appeared in the Build menu, PLACE shown where before it read RESEARCH
SPHINCTER HOUSING; a Lume Bulb grew at 105,1 and its card read the DARK rule; with Plasticity and Conductance set
the Hive held and a Relay Shrine stood on a Node after one shift; its rules read "signal > 0 → dim HOLDS" and "white
light < 8 → shut gates"; after S1's first run home and a second, Fenn's loop came home with 35 items at 0 % decay; a
jam gave the Mat's proposal of one tile, CONFIRM and SEVER on the tile's card.

**Reviewed by eye** (`docs/shots/bench_research.png`, `strain.png`, `codex.png`, `play_17_node.png` through
`play_24_handoff.png`): the Bench's RESEARCH tab, three branches of twelve tiles with the founding's three DONE in
Acid and the rest BLUEPRINT in Dust, the project bar with its points and its NEEDS line in Hot Pink; the Strain Lab's
genome, the twelve organs three by four with their lines and counts, the feed plan, the classes with HELD and the
lineage; the Codex's five tabs. What the frames changed: node names ran past their tiles — three lines, the tile 64
by 40; a node's tile gave no reason it could not be picked — the blueprint line says what opens it; the Build list
showed a locked structure as placeable — "RESEARCH <node>" and no PLACE; an organ's text ran off the card — two
lines; after a pick the Bench drew the old plan — it replans; the proposal said "1 TILES"; a Bulb dimmed by a Nerve
rule drew lit.

**Tests**: `progress` 5 (the tree is 36 and the founding holds tier 1; a blueprint studied opens its tier and points
finish a node; the book is 38 and the organs 12; classes held over their lines, two at most, three by a Relic; the
collections), `game` +6 (a blueprint studied and a node researched; organs grown and a class held;
reflection and the deeds; a Nerve rule's gates; a loop's decay and run; the handoff's proposal and its tile),
`save`'s round trip carries the progress. `cargo clippy` adds no warning in this work's code.
