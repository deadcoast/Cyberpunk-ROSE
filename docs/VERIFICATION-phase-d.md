# VERIFICATION — Phase D (pressure), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-060 — Attention, Patrol / Sweep / Cleansing, Blooms, Floods, raiders (2026-11-09)

Full Design v3.14 first (Colony mode: Pressure; the Threats table; the shift clock; the Route Board; the Colony and Run
HUDs), research 18 (threat pacing: the storytellers' time since the last threat became each stage's quiet spell). The
stages and bodies are data (`data/threats.toml`; raiders and the run's Flood in `data/creatures.toml`); the rules are
the `threat` crate on the city's clock and the `run` crate on the run's; `game::pressure` joins them to the shift and
`game::siege` stages the proof.

**The proof** — `rose cleansing --seed 11 --shifts 10`: PASS, the designed spread. The steward plays ten shifts, then
a Cleansing comes down the first grate twice from that state:

| city | the Heart | Mat cells | slimos | dead | Oathkeepers fell | Mat tiles burned | seconds |
|---|---|---|---|---|---|---|---|
| as it stands | burned | 303 → 246 | 7 → 4 | 3 | 0 of 8 | 113 | 1657 |
| defended | 200 HP | 303 → 288 | 7 → 6 | 1 | 8 of 8 | 15 | 247 |

Defended is two Sphincter Gates on the squads' way down from grate 5 (102,1 and 102,18) and a Scrap Turret at
102,7, a Guard on each; the Gates close the way. Over seeds 11–18 the spread holds on seven; on 13 the steward's city
has thinned to too few to stand Guard by shift ten and the Heart burns defended too (its route card kills eight of
its Runners, PAC-045's registered weakness).

What the proof found on the way, each now in the design: a Patrol's two scouts with lamps that added burned 17 tiles
of a mapped Lume Farm, so lamps take the worse, not the sum, and a scout's lamp smokes (0.3 a Mat step) where a lamp
carrier's burns (1); a Cleansing came down in the Sump in shift 13 and burned the Heart, so each stage comes down only
from its stratum (Patrols S1, Sweeps S4, Cleansings S6 or the chapter-3 accord); feral pieces cut off by bleach kept
the Heart's signal from winning them back, so only a Bloom stops signal; Gates of 2 × 2 could not span the shaft and
any squad hit broke them, so a Gate is 2 × 1 and turnable, HP 400 a tier, opened only by flame; guns on lamp carriers
killed every Guard before the Gates mattered, so carriers and sprayers carry poles.

**The city with its threats** — `rose city --shifts 20` on seed 11: PASS, the build plan by the end, 0 of 20 Runners
dead, bit-exact replay; five Patrols came down grate 5 and mapped it, seven Flood shifts (the water held twice, rose
1, 2 and three times 3), the run's water rose on four of the runs. Over seeds 11–26 the cord holds or is won back on
all sixteen and the plan stands on ten; on 13, 14, 15, 23, 24 and 26 the route card kills 4–8 of twenty Runners and
the city thins before its plan stands — PAC-045's five and 24, the steward still never changes card. Seed 21 found
the one change to Phase C's city: a Salt Works whose cord had died went on filling bags that waited at its tile for a
way home, late cargo every shift after. The design already said signal is power; now a structure whose anchor is
under its need at a shift's end does no work that shift and takes none the next, and the log says so (one structure
on one seed of sixteen went dark).

**The run's raiders and Flood** (Full Design v3.14, Raiders; Floods). `rose city` found two things before they were
written down. A raider that walked floors as a scout does came down at the far end of a Sump route, reached 30
tiles and none of its 13 containers, and left: a raider climbs shafts as a Runner crawls them. Drowning counted under
any deep water put the bot under in S2's authored swims (S2's median run went from 3:37 to 9:01): only the Flood's
risen water drowns. `rose cadence`, deaths by stratum S1 / S2 / S3:

| raiders | runs a stratum | deaths |
|---|---|---|
| none | 40 | 28 / 32 / 22 % |
| 25 / 35 / 45 % a gang | 40 | 38 / 40 / 28 % |
| 0 / 25 / 35 % | 40 | 28 / 40 / 28 % |
| none | 60 | 25 / 30 / 23 % |
| 0 / 15 / 25 % (shipped) | 60 | 25 / 35 / 25 % |
| 0 / 15 / 25 % (shipped), `docs/telemetry/cadence.md` | 20 | 10 / 30 / 25 % |

The bot never waits for a gang to leave; a player can, and the death band is the player's (Phase F calibrates the bot).
The bot surfaces when it has been under the Flood's water half the drowning time.

**Reviewed by eye** (`docs/shots/colony_sweep.png`, `colony_flood.png`, `colony_stack.png`, `run_pressure.png`,
`route.png`): a Sweep sent down grate 5 stands at the Heart in one hard-edged pool of white light, its six bodies
fanned across the tile, bleach drips on the cords beside it; a Flood shift's two risen rows read in Flood Light with
ripples over the darker standing water, the surface a Cyan line; the Stack view keeps G5's gauge under its grate; on
a run, raiders in Oxide come down off the Runner's shoulder with RAIDERS ×2 in Muzzle Orange, the clock reads
"WATER +2 OF 3" and the risen water's surface shows through the dark; the route cards carry "FLOOD +2 AT 1:52" in Cyan.
What the frames changed: the white light read as flat grey until its pools took a hard edge; the risen water could not
be told from the standing until it took ripples; six bodies on one tile drew as one; the HUD gauge across the top bar
showed a continuous number the design keeps for Corpo intel, so it became three steps pinned under each reached grate.

**Tests**: `colony`'s jobs test adds a Bench gone dark; `threat` 9 (the stages' lines and quiet spells, white light against armour, bleach and salt, a Bloom from a
cut-off strain, Quarantine, the flood rising and falling, Gates holding a squad and the flood); `run` 61 (+ raiders
looting, shooting the other gang and dropping a pile; a Flood raising each chunk's water and a Runner going Down under
it; the card and the run agreeing on the Flood's moment); the smoke checks the zones and the Flood's clock on the run's
shift and card. Save v12 (the threats). `cargo clippy` adds no warning in this work's code.

## PAC-061 — the economy: streets, demand, reputation, stock tiers, services, contract chains (2026-11-10)

Full Design v3.15 first (Economy: the streets above the grates, the vendors, demand on the surface's clock, price,
reputation, stock tiers, buying and services, contracts, the Downer chain, the haul against the ground), research 19
(RimWorld's trading and goodwill, Tarkov's loyalty levels). The factions, the surface's events, stock and contract
templates are data (`data/factions.toml`); the rules are the `economy` crate on the surface's clock (a day is three
shifts); a stall's `items::Book` prices a sale; the ledger keeps glut by market and category, the books and the
orders; `game::trade` joins it all to the shift, the cords, the roster and the threats, and the bridge calls the same.

**The proof** — `rose economy --seed 11 --shifts 30`: PASS. Every faction's demand for every category walked at most
0.16 from its buy-list value over the ten days (band 0.30); over seeds 11–26 the widest is 0.16 on every seed. The
trading steward sold 312 pieces at the Nomads' stall over grate 5, nearly all blind (no Appraisal Table in its plan):

| category | sold | eddies | of base (× 0.4 blind) |
|---|---|---|---|
| chrome | 88 | 188 | 95 % |
| contraband | 18 | 161 | 130 % (a few eddies rounding up) |
| curio | 69 | 134 | 79 % |
| gear | 64 | 209 | 82 % |
| tech | 73 | 191 | 76 % (the Nomads' Tech 0.8) |

The Nomads ended at +1 and tier 3 on their lever (five sold a shift) and 883 e traded; one three-link chain taken.

**Trading against feeding** (the PAC-045 input, now the design's "The haul against the ground"), seeds 11–26, thirty
shifts each: selling the haul's Chrome earned 11,671 e against feeding's 8,489 (more on fifteen of sixteen); the
nearer grate fell in four cities either way; the selling cities went without a market 50 shifts in all, the feeding
ones 24; on seed 21 the selling city lost its market for 25 shifts and ended with nothing, the fed one reached the
nearer grate in shift 15. So the feed is insurance on the cord, not reach.

What the proof found on the way, each now in the design: the glut read as a fifth of the units recovering a shift
never cleared — a stall taking five a shift sat at the floor and sales fetched a fifth of their base — so the price
recovers 20 points a shift (Pass 1's words), four units' worth; a city whose only reached street closed at −3 had
nothing left that could earn favour there, so below 0 a faction comes round a favour a shift (RimWorld's goodwill
drift); a sale could round to 0 e, so a stall that buys pays at least 1. The steward's own lessons are its orders, not
the game's rules: a host it took became its best Scav and died on the run (a guest stays home now); it took two
chains on the same Parts at once (one chain a faction); five-link chains asked more than one S1 route hauls in their
shifts (three-link only).

**The city with the economy** — `rose city --shifts 20` on seed 11: PASS, plan 7/7, 0 deaths, bit-exact. Over seeds
11–26 against the same command before this work: the cord holds or is won back on all sixteen, the plan stands on
twelve (ten before), eleven seeds' deaths unchanged; on 12, 13 and 15 more Runners die (7, 12, 12 against 4, 8, 7) —
the same route card was killing every Runner from shift 9 or 10 there already, and clinic days and hosts brought more
slimos to it (4, 9, 7 joined against 2, 2, 2). `rose cleansing` over 11–18: the spread holds on seven, 13 as before.

**Reviewed by eye** (`docs/shots/colony_stack.png`, `stash.png`): G5's gauge carries NOMADS under its three pips; the
Stash card of a Blackline Lamp Core reads "SELL no stall at GRATE 5 NOMADS buys this" with the verb greyed, and the
bulk panel sends "TO GRATE 5 NOMADS". What changed for it: the markets were named by their index, so grate 5 read
GRATE 1; a Blackline piece showed a price no stall would pay.

**Tests**: `economy` 4 (the streets, the walk inside its band, reputation opening tiers and closing a faction and
only the Corpo remnants' tier 3 fencing Blackline, a chain rising link by link and a miss breaking it); the ledger's
glut recovering 20 points; `game` +2 (a delivery counting at its grate, a Relic riding home, Blackline sold adding
Attention; protection holding off the Patrol due at a Gang grate); the save round trip with the economy (v13); the
smoke checks the first grate's street and market name. `cargo clippy` adds no warning in this work's code. Not
exercised in play: the steward buys only protection, at a Gang grate in reach; intel, timings, the convoy and hires
are bought on the Market screen (PAC-062), and recover's body is planted on its route when that screen takes the
offer.
