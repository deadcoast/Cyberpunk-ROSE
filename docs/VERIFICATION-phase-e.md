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

## PAC-081 — the campaign: the stack, the chapters, Ivan, the shell, and the proof (2026-11-12)

Full Design v3.19 first (the campaign written to its numbers: the stack of six strata and the breach, newcomers by
stratum, Ivan on the map, the seven chapters, the accord, the chapter's Bloom, the council, the Great Cleansing, the
warchest, the endings, panels, barks, Endless, Daily, the strain code and the profile, difficulty), research 22; then
v3.20 for the climb and for what `rose campaign` found. Nine commits: the stack and the breach (part 1); the story
crate, the seven chapters and the endings with their panels (2); Ivan on the map and his 379 barks (3); the routes of
S4–S6 and every source on its own stratum's loot row (4a); New Run, the strain code, the profile, the Daily Seed,
Endless, and the Mat's climb (4b); `rose campaign` and the rules it found wanting (4c–4g).

**The proof**: `rose campaign --seed 11` PASS — every script reached its own ending (`docs/telemetry/campaign.md`,
every tenth shift of each there and every decision in `out/campaign_<ending>_11.log`):

| script | the accord | ending | warchest | shifts | chapters 2–7 opened at shift | strata 2–6 reached at shift |
|---|---|---|---|---|---|---|
| Garden (Lume, Thrift) | accepted | Garden | 29,410 | 114 | 12 · 14 · 19 · 81 · 83 · 107 | 2 · 38 · 80 · 89 · 107 |
| Hive (Chrome Affinity, Conductance) | accepted | Hive | 26,865 | 85 | 13 · 17 · 21 · 56 · 59 · 73 | 2 · 29 · 57 · 66 · 73 |
| World-Eater (Appetite, Feral Drive) | accepted | World-Eater | 35,030 | 136 | 13 · 17 · 22 · 110 · 112 · 126 | 2 · 46 · 111 · 121 · 126 |
| Accord (Thrift, Toxicity) | accepted | Accord | 9,085 | 102 | 13 · 15 · 19 · 59 · 62 · 93 | 2 · 49 · 61 · 77 · 93 |
| Scattered (no defence) | refused | Scattered | 87 | 16 | 12 · 14 · — | 2 · — |

Four scripts climb the six strata in 73–126 shifts and meet the Great Cleansing at chapter 7; Scattered's refusal
brings chapter 3's Cleansing down whole on an undefended city and the Heart burns in shift 16. 575 seconds for all
five.

Each script is the steward of `rose city` with a player on top (Full Design, Narrative and Ivan: The proof); its
steering is the proof's plan, not a rule. What playing it found was a rule each time, written into v3.20 before the
code (the history line lists them all): the Mat's growth (a fed strand's end grows again, a sprout enters ground
touching two hyphae, the pump pushes past the Relays, a breached slab takes the Mat in, a bare Heart or Relay seeds
again, a meal takes only what lies past the extension reserve); the Oathkeepers (a stage spends its line, only a
scout's death is +15, the posts, organs, the tamed and Toxicity let a scout go, a Patrol leaves after four minutes, a
Sweep takes its district and goes home, a Patrol or Sweep turns back at a shut Gate, the mapped Heart draws only its
own zone, the gauge's Cleansings wait on the Undercroft and fall silent once the Great Cleansing comes); Blooms (feral
in three shifts, two crawlers for every four hundred tiles over every Bloom, a Bloom's tissue not the city's); and the
endings (the ally's reputation counted past the alliance's own step: counted from zero it gave the Garden 0.05 over a
level Accord, and the Accord script, Thrift and Toxicity at 1.00, tied a Garden of Lume 0.90 by that step and lost the tie). The last
round also fixed the steering itself: holding every rival's every gene kept nearly all the Chrome off the Pits and
spent 16,000 eddies buying more to keep, so the World-Eater's climb, cut by shift 102, never grew back; and the
Garden's ally fell from +3 to 0 on contracts it missed after the alliance.

**The other proofs**: `rose city` PASS; `rose economy` PASS; `rose cleansing --seed 11` PASS (defended, the Heart 200
of 200, eight of eight fell). The Cleansing failed first: the founding's Mat, grown by this part's rules, sent the
squads in on the Heart's far side from a Turret placed nearest it, and the flamers burned the Heart six seconds before
the last of them fell. The proof's Turret now stands where the campaign's steward puts its own — within three of the
Heart, on the grate's side (`siege::turret_spot`, one rule for both) — and v3.20's proof paragraph says so. `./run.sh`
now runs `rose campaign` too (ten minutes).

**In play** (`play.gd`, the live game): the Sump Grate Key opened the slab at 102,239, the city's stratum read S2 and the Mat
reached the Culverts; Ivan rallied five, walked to 102,240 in 71 ticks, Parleyed with the Nomads at grate 5 (favour
+10, tier 2 open today) and spoke ("The districts voted. Mostly with their feet."); chapter 3's card read ANSWER THE
ENVOY with the envoy at the grate, ACCEPT signed the Tithe and the card turned to THE ACCORD'S CLEANSING; the ending
read Hive, the warchest 290.

**Reviewed by eye** (`docs/shots/newrun.png`, `panels.png`, `ending.png`, `play_25_slab.png` through
`play_31_ending.png`): New Run's mode, seed, strain code with its hint ("blank: the Heart is born at the founding's
genome"), the three difficulties with the chosen one's line, BEGIN and BACK; the slab's card and the breach; Ivan on
the Mat, his Parley and bark box; the envoy at the grate, chapter 3's panel and the HUD's chapter card ("CH 3 THE
HAND: THE ACCORD'S CLEANSING") with Ivan's line after ACCEPT; the ending's panel and then the held Stack view with the
warchest, the strain code, NEW GAME+ and THE MENU.

**Tests**: `story` 6 (the chapters and endings as data; the endings scored from the genome and the ally, the
alliance's own step no one's edge; the campaign opening on chapter 1; Ivan's bark bags and their tokens), `game` +12
(a chapter closing on its goal, the accord and the Tithe, chapter 4's Bloom, the Great Cleansing with its militia and
the gauge silent after it, the breach by Relay or key, Ivan's walk, his Rally, Feed and fall, his voice on the city's
events, New Game+, Endless, the Daily Seed), `save` +1 (the profile; the strain code now read back with its typo
caught), `mat` +1 (the stack and its slabs), `logistics` +1 (the Board's strata), `route`'s library at six strata, and
`threat`'s Turret test rewritten (a Patrol maps and leaves, a Sweep is fired on). 187 tests pass. `cargo clippy` adds no warning in this work's code.

## PAC-082 — the living city and the population economy (2026-11-12)

Full Design v3.21 (The living city; Onboarding; Recruitment; Homes, meals; City tiers; Building). Found by playing
the live game and reading `rose campaign`'s logs: a placed ghost waited for the run to end; a Feed queued before the
first run bounced back to the Stash (the city's consumers were published only at a shift's end); the founders took
the player's first Bunkhouse; the HUD's clock read the Mat's tick; and the campaign's "Empire of 2,000" was four in
five D5 bodies, while its sleepers stopped near 300, held by the bot's own food cap and by Parts.

**What changed**: colony labour, newcomers and the Stash's orders every 120 city ticks (`game::shift::work_cycle`, the
headless world and the bridge alike); the founders' Bunkhouse; consumers published every cycle; the first ten
minutes as ten beats with Ivan's line, a light on the control and the Codex's Systems tab; the Build tool's dots;
Ivan's box off the map; the run camera inside the route. Then the population: a D5 off the street is a flesh pile the
Mat takes at the grate; the tiers and the chapters count sleepers; meals come from the whole signalled network, half
of each tile's surplus a shift; the Bunkhouse is 6 / 24 / 60 beds, every step 10 Parts. The campaign's scripts build
beds while the city eats, buy Parts for the waiting, spend on beds while a chapter waits on the city's size, and a
script whose rival the alliance favours holds that rival's genes from the founding.

**The proof**: `rose campaign --seed 11` — every script reached its own ending. The city's hours are the bot's run
seconds times its pace factor 3:

| script | ending | shifts | Hamlet (20) | Quarter (100) | City (500) | Empire (2,000) | the end |
|---|---|---|---|---|---|---|---|
| Garden | Garden | 122 | 0.8 h | 4.8 h | 7.1 h | 14.8 h | 16.0 h |
| Hive | Hive | 110 | 0.8 h | 3.8 h | 6.8 h | 12.0 h | 12.8 h |
| World-Eater | World-Eater | 112 | 0.8 h | 5.3 h | 10.1 h | 14.3 h | 16.3 h |
| Accord | Accord | 84 | 0.8 h | 4.9 h | 6.8 h | 10.8 h | 11.9 h |
| Scattered | Scattered | 11 | — | — | — | — | — |

Pass 2's tiers ask for the Hamlet in the first hour, the Quarter at three to four, the City at about eight and the
Empire at about fourteen, a campaign of 12–18 hours; before this the same scripts ended at 184–257 shifts, 25–37 hours.
The Quarter comes an hour late. The newcomers still outrun the beds from the City on: about 4,000 an hour come down
and nineteen in twenty go feral (one line of the log a cycle).

**In play** (the live game under xvfb, driven by input): a Bunkhouse placed on the Build tool's dots stood at the
second work cycle with "2 at work" on its card; four newcomers joined at minute four; a Street Eye fed from the Stash
before any run landed in the Heart at 3:00.5; a Fried Board sold at 10:24 and the markets beat ended on it; the
beats fired in order with BUILD, the ghost and the nav buttons lit; Ivan's line left the run's HUD as the Runner
departed; after a run the city line read "fed by the Mat".

**The other proofs**: `rose city` PASS (fourteen sleepers fed by the Mat; the ledger at 192 e by the twentieth shift,
where it sat near nothing on paste); `rose cleansing` PASS; `rose economy` PASS; the headless smoke PASS. **Tests**: 188
pass (the onboarding beats in order and the notices in any order; meals from tissue far from the home; the warren's
cost).
