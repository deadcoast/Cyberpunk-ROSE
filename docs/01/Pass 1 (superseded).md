# Cyberpunk: ROSE — Rise of the Slimecity Empire

Nov 7, 2026 · @heat

## The game on one screen

ROSE is a side-view colony builder where the city is a living thing. The player feeds a mycelial network (the Mat) that spreads through the sewers by cellular-automaton rules; every hab, forge and market in Slimecity stands on tissue the Mat grew, and the Mat eats what it stands under when it starves.

|  |  |
| --- | --- |
| Genre | Underground colony builder on a cellular-automaton substrate; real-time with pause; tactical slow-time defense sequences |
| View | Side-on cross-section of the sewers under Night City, six strata deep, pixel art at 480×270 scaled by integer factor |
| Engine | Godot 4.x; simulation core in Rust (GDExtension); game logic and UI in GDScript |
| Player | Ivan Ooze, run from above as commander and placed on the map as a hero unit |
| Session | 7-chapter campaign (12–18 h) plus Endless Sump sandbox with shareable seed codes |
| Platform | PC first (Windows, Linux); mouse+keyboard and controller |

**The retained dilemma.** Every unit of chrome scrap goes one of two ways: up a grate to a Seep Market for eddies, food and tools, or into a Pit to feed the Mat. Sell everything and the Mat starves; a starving cord digests the structure it carries, then the slimo sleeping in it. Feed it rare chrome and it grows fast and mutates hard; past a point the strain you fed no longer answers to you. The old space design had miners feeding monsters to keep them off the ships. Here the monster is the ground the city stands on, and the lore makes that literal: the Oathkeepers' antigen turned decaying chrome into mycelium, so a slimo colony is a fungal network with people in it.

**Why this genre for this lore.** The lore's verbs are grow, build from scraps, trade up, hide, survive a purge, rise. A builder puts every one of those in the player's hands each shift. The CA gives the Game-of-Life unpredictability the original brief wanted, and the feeding system gives the player a hand on the wheel without a steering lock.

**One shift, played.** A scav crew comes back from a storm drain with 14 units of junk chrome and 3 of street chrome. The player sells the junk, drops the street chrome into the east Pit to pull the Mat toward a dry culvert where a Hab-Pod is needed, severs a cord that has wandered under the market and is starving there, and watches the Attention gauge at the north grate tick up because the new Lume Bulbs are visible from the street. A tweaker gets the Pod up before the Flood shift raises the water two tiles. Nothing in that sequence was scripted; the Mat's shape, the organ that grew, and the water level came from the seed and the player's choices.

## Lore to mechanics, and what this pass invented

Each lore term is cashed out as one system. The right-hand column is the mechanic; the lore itself is not restated here.

| Lore term | Mechanic it becomes |
| --- | --- |
| MAP-SSRI-II vaccine, "re-organifying" chrome | Newcomer supply: flesh piles flushed from surface clinics arrive at storm-drain grates every 3–5 days, each carrying their dissolved chrome as embedded scrap (T1–T3) and a Mod-Depth |
| Mycelium antigen networks | The Mat: the cellular automaton every structure stands on; every slimo is a node on it |
| "Self-inflicted liquidation" | Appetite rule: a starving cord digests the structure or slimo it touches; Dissolved (D5) slimos can be fed to a Pit |
| Sewerats | A Mat region cut off from the Heart whose strain crossed the feral threshold; hazard, then tameable guard animal |
| Tech-tweakers | The builder class; structures are scrap plus a grown organ |
| "No end to what a tweaker can make from scraps" | Scrap is the only build material; there is no ore |
| Oathkeeper experimental mil-tech | T4 Blackline scrap tier; Attention and Cleansing threat; white light versus biolume |
| Helium-3 holy rock | T5 Relic tier: lunar reliquary fragments; wild genome jumps |
| Trade with the Oathkeepers, then genocide | Chapter 3 scripted betrayal; the Attention system runs from then on |
| "Warchest beneath Night City" | Finale is scored on stockpile: biomass, chrome-infused Mat, tamed sewerats, tweaker arsenal |
| CyberSewer | Lighting model: ambient near-black, neon only as emitted light |

**Invented in this pass (not in the lore; every one is swappable):** the Mat, the Heart, Pits, Severing, Salt Lines, Bait Smears; the ten genes and their names; strains and speciation; Ferality; Mod-Depth D1–D5; the scrap tier names Junk, Street, Corpo, Blackline, Relic; eddies as the surface currency; Downer (the Mat-enzyme palliative sold topside); Seep Markets; the Attention meter and the three Oathkeeper escalation stages (Patrol, Sweep, Cleansing); ooze-time; the six strata names; chapter titles; the four endings; faction placeholders Gangs, Nomads, Corpo remnants, Badges; the SEEPAGE palette and its colour names.

## Core loop and session structure

The Mat ticks at 2 Hz by default (1, 2 or 4 Hz selectable, pause always available). One shift is 120 ticks, one minute of real time at 2 Hz. Three shifts make a day: Dim, Dark, Flood. Rain on the surface during Flood raises the water in the lowest stratum by 0 to 3 tiles, decided by the seed.

Every shift the player does some of six things; none is mandatory, and skipping one has a cost the next shift.

1. **Scav.** Crews of 2–4 tweakers go into unlit tunnels or up a drain for one shift and come back with scrap by tier, parts and sometimes a newcomer. Risks: ferals, Oathkeeper scouts, drowning in Flood.
2. **Feed.** Scrap goes into Pits by tier. A Pit is a stockpile the Mat draws down at its own intake rate, so nothing is wasted by over-filling; what tier you feed decides growth speed and genome drift.
3. **Build.** Tweakers raise structures on Cord or Node tiles that carry signal ≥ 0.3. A structure whose tile necroses loses its foundation and decays.
4. **Tend.** Sever a cord to quarantine or redirect; smear Bait to pull growth; lay Salt to stop it; place a Relay to extend signal.
5. **Trade.** Orders queued at a Seep Market resolve at shift end; prices move with faction reputation and glut.
6. **React.** Purge alarms, Blooms and floods interrupt in real time. Pause is free; ooze-time is Ivan's and has a cooldown.

A session is 6–12 shifts for a chapter beat; a campaign chapter is 2–4 sessions. The sandbox has no chapters: Attention and newcomer flow scale with the size of the Mat instead.

## The Mat: cellular automaton spec

The Mat is a stochastic cellular automaton (CA: a grid where each cell changes state by rules that look only at its neighbours) coupled to a continuous nutrient field. The field is the differential-equation half; the hyphal states are the Game-of-Life half. The player touches it only through what they feed, where, and what they cut. Every parameter below is the one the prototype in the next section ran with.

### Grid and fields

Each stratum is 192 × 48 tiles; six strata stack to 192 × 288 = 55,296 cells, simulated as one grid. Four fields per cell:

| Field | Values | Notes |
| --- | --- | --- |
| Substrate S | Rock, Concrete, Brick, Pipe, Open, Water, Grate, Deposit | Static except Deposit (depletes) and Water (level moves with Flood) |
| Nutrient N | 0.0 – 1.0 | The food the Mat lives on; a cell holds at most 1.0 |
| Hypha H | Empty, Tip, Cord, Node, Necrotic | Plus age in state, strain id, stall and starve counters |
| Signal σ | 0.0 – 1.0 | Recomputed each tick by breadth-first search from the Heart and Relays |

Nutrient conductance by substrate: Pipe 0.6, Open and Grate 0.4, Water 0.25, Brick 0.08, Concrete 0.04, Rock 0. Between two hyphal cells the link conductance is the Conductance gene's cord value (0.6–1.0), whatever the substrate.

### Nutrient update (every tick, in this order)

1. **Pit draw.** Each footprint cell of a Pit takes up to 0.30 per tick from the Pit's stock, never above 1.0. Stock that is not drawn stays in the Pit.
2. **Diffusion**, explicit, 4-neighbour:

```latex
N_i' = N_i + D \sum_{j \in adj(i)} k_{ij}\,(N_j - N_i), \qquad D = 0.11
```

Stability needs D · Σk ≤ 0.5; with four links at k = 1 that is D ≤ 0.125, so 0.11 holds everywhere. Nutrient never crosses Rock. 3. **Uptake.** A hyphal cell absorbs 25% of the nutrient in each adjacent empty cell per tick. This is how the Mat eats a Deposit and how it harvests a Pit's overflow. 4. **Heart pump.** A signalled hyphal cell pushes its surplus toward hyphal neighbours farther from the Heart (lower σ), split equally across those links, each gift capped by the receiver's free capacity:

```latex
g_{i \to j} = \min\!\left( \frac{F \cdot \max(0,\; N_i - R)}{c_i},\; 1 - N_j \right), \qquad R = \theta_{starve} + 0.05
```

where F is the pump fraction (gene-driven, 0.5 at default), c the number of outward links. The reserve R means a fed cord never drains itself below the starvation line; the capacity cap means dead-end cords saturate and the flow re-routes to branches that still want food. Without the pump the field is pure diffusion and nutrient travels about 5 tiles before decay eats it; with it a single strand carries food 35 tiles in 120 ticks. 5. **Upkeep and decay.** Each hyphal cell burns 0.0006 per tick (Thrift gene scales it 0.6–1.4×); the whole field loses 0.1% per tick; a Deposit emits 0.004 per tick into itself; necrotic cells that finished decaying release their stored nutrient.

### Hyphal rules (synchronous, seeded RNG, in this order)

| Rule | Condition | Result |
| --- | --- | --- |
| Starvation | Cord or Node under θ\_starve for 24 ticks; Tip for 8 | Necrotic. The Heart never starves |
| Isolation | Any hypha with no hyphal neighbour for 12 ticks | Necrotic |
| Decay | Necrotic for 30 ticks | Empty; releases 0.35 N to its cell and 0.10 to each neighbour |
| Maturation | Cord with ≥ 3 hyphal neighbours and N ≥ 0.25 for 10 ticks | Node, p = 0.2 per tick. Nodes are junctions; organs and structures anchor here |
| Anastomosis | Tip touching ≥ 2 hyphal cells | Cord (a loop closed) |
| Lateral branch | Cord with exactly 2 hyphal neighbours, holding reserve ≥ need | Sprouts a Tip sideways, p = 0.05 · b per tick |
| Extension | Tip holding N ≥ θ\_grow + 0.16 | Moves into one growable empty neighbour that touches at most 2 hyphae (p = 0.7); pays 0.04, hands the new Tip 0.12; parent becomes Cord unless it forks (p = b) |
| Stall | Tip that failed to extend for 14 ticks | Cord |

Direction choice is chemotropic: candidate cells are weighted 0.25 + N, so richer ground ahead is favoured without being required, and a cell that would close a loop is weighted at 15%. Growth is therefore reserve-gated (a tip extends when the network has delivered enough to it) and the frontier speed is set by supply, not by a timer. That is what makes feed rate equal growth rate.

### Appetite (the feed-or-be-eaten rule)

A Cord or Node under θ\_starve that touches a Structure or a slimo digests it with probability α per tick: Structure HP −1 of 20, slimo health −1, cord gains 0.05 N. A city built on a starving Mat loses its foundations first and its people second.

### Signal

Breadth-first search from the Heart and every Relay along same-strain hyphae; step cost Cord 1, Node 0.5, Tip 2. σ = max(0, 1 − d/R) with R = 24 + 32 · Conductance (40 tiles at default). Structures need σ ≥ 0.3; the pump only runs where σ > 0, so growth beyond signal range lives on local Deposits alone and dies back. Relays and Nerve organs are how the Mat reaches the next stratum.

### Genome

Each strain carries ten genes in \[0, 1\], all 0.5 at the Heart's birth. They are the only knobs, and they are expressed linearly into the parameters above.

| Gene | Expression |
| --- | --- |
| Vigor | θ\_grow = 0.35 − 0.25 · g (grows in poorer ground) |
| Branching | b = 0.08 + 0.42 · g (fork chance, lateral sprout rate) |
| Thrift | θ\_starve = 0.05 + 0.10 · (1 − g); upkeep × (1.4 − 0.8 · g) |
| Appetite | α = 0.02 + 0.30 · g |
| Lume | light radius per Node = 1 + 6 · g tiles; raises Attention near grates |
| Toxicity | damage to Oathkeepers standing on the Mat = 2 · g per tick; slimo Mood −0.5 · g per shift |
| Chrome Affinity | cord armour HP = 1 + 9 · g; above 0.6 cords conduct power without a Relay |
| Feral Drive | Ferality gain per isolated tick = 0.01 + 0.09 · g |
| Conductance | R = 24 + 32 · g; cord link k = 0.6 + 0.4 · g; pump F = 0.3 + 0.4 · g |
| Plasticity | mutation magnitude × (0.5 + 1.5 · g); the gene that makes the others move |

**Drift on feeding.** Feeding mass m of tier k into a strain moves its genome by

```latex
\Delta G = m \cdot \mu_k \cdot (0.5 + 1.5\,G_{plast}) \cdot \left( u + 1.5\, b_k \right)
```

with u a random unit vector (the Conway part), b\_k the tier's bias vector (the natural-selection part) and μ\_k the tier's drift magnitude: 0.01, 0.03, 0.07, 0.15, 0.40 for T1–T5. Genes reflect at 0 and 1, so a Relic feed can bounce a gene off the ceiling and land somewhere nobody chose.

**Organs** grow by themselves at Nodes once a gene crosses its line: Lume Bulb (Lume > 0.6), Filter Lung next to Water (Thrift > 0.55), Sphincter in a 1–2 tile chokepoint (Chrome Affinity > 0.5), Nerve Relay (Conductance > 0.65), Spore Sac (Feral Drive > 0.7). The player did not place them; they are what the strain does.

### Strains, speciation, Blooms

A strain is one connected component with one genome. When a Severing, a Cleansing or starvation splits the network, each piece keeps a copy of the genome and drifts on its own; two shifts apart with genome distance above 0.25 and they are different strains, and their tips fight at the border (both lose 0.05 N per tick while touching). Re-merging blends genomes by biomass and adds a shock of ±0.1 on one random gene.

A strain with no Heart or Relay in it and Feral Drive above 0.5 starts a Bloom: its Spore Sacs spawn sewerats and its Appetite doubles. This is the late-game disaster of the old design (the monsters eat the ships) arrived at by rule rather than by script.

### Determinism and cost

One PCG32 stream seeded per run; every random draw in the sim comes from it, UI and VFX use a separate stream. A run replays from (seed, input log); seed codes are shareable the way Life patterns are. The prototype's unoptimised numpy loop runs a 192 × 48 stratum at 1.3–1.7 ms per tick; the Rust core targets under 2 ms per tick for the full six-stratum stack by touching only active cells.

## Prototype evidence

The rules above were run headless (Python/numpy, `mat_sim_proto.py`, delivered with this doc) on a generated 192 × 48 stratum: three culvert rows, vertical shafts every 23 tiles, collapsed caves from smoothed noise, brick lining, scrap deposits, a water line. One Heart at the centre, fed 0.6 nutrient per tick (about one Street-tier unit every two ticks); a second Pit opens at tick 120 some 55 tiles away. Seed 11 for every frame.

What the four runs show, same seed throughout:

| Run | Cords | Nodes | Reach from Heart | Reading |
| --- | --- | --- | --- | --- |
| Default genome, fed to t=350 | 151 | 42 | 26 tiles | Strands along the culvert, branches climbing a shaft and dropping to the water line, loops at junctions |
| Default, feed cut at 350, shown at 600 | 172 | 49 | 31 tiles | The Mat keeps extending for a shift on stored reserve, then the first necrosis appears at the far ends |
| Runner strain (Vigor 0.9, Conductance 0.9) | 221 | 24 | 43 tiles | Long thin runs, few junctions: the same seed, a different animal |
| Bramble strain (Branching 0.9) | 173 | 31 | 37 tiles | Denser forks, more side-shoots into caves |

Three things the run settled that the first draft of the rules had wrong, kept here because they are the mechanism:

- A network whose tips all mature stops growing for good. Lateral branching from strand cells is required, and it must be reserve-gated or it fills corridors into blobs.
- Pure diffusion cannot feed a frontier; nutrient travels about 5 tiles before decay takes it. The Heart pump, moving only surplus and only outward, is what lets a strand reach 35 tiles in a shift.
- A pump that shoves into saturated dead ends wastes most of the supply. Capping each gift at the receiver's free capacity re-routes the flow to branches with demand, and that one change took reach from 16 tiles to 30 on the same feed.

The same seed run twice produces identical state (asserted in `mat_evidence.py`). Upkeep is tuned so 0.6 nutrient per tick sustains roughly a thousand cells; the frontier advances about 0.6 tiles per tick at that feed, which is the right order for a stratum 192 tiles wide to take 3–4 shifts to cross with Relays.

The snapshot sheet is below; the second Pit's nutrient halo is visible at top right in every frame as the dark-green dither the Mat has not yet reached.

&#91;image: Mat CA prototype: four runs on seed 11, 192 x 48 stratum, SEEPAGE palette\]

## Economy

Scrap is the only material. Five tiers, each with a feed value (nutrient per unit when it goes into a Pit), a drift magnitude and a bias (the genes it pushes), and a sale price in eddies at a Seep Market before reputation.

| Tier | Scrap | Where it comes from | Nutrient per unit | Drift μ | Bias (genes pushed) | Base price |
| --- | --- | --- | --- | --- | --- | --- |
| T1 Junk Chrome | Dead, vaccine-liquefied implants; cheap street mods | Every stratum; newcomers carry it | 0.18 | 0.01 | Thrift, Branching | 4 €$ |
| T2 Street Chrome | Working street cyberware | Culverts up; gang trade | 0.30 | 0.03 | Vigor, Lume | 15 €$ |
| T3 Corpo Chrome | Arasaka-grade hardware | The drowned Vault (S5); Corpo trade | 0.45 | 0.07 | Chrome Affinity, Conductance | 60 €$ |
| T4 Blackline | Oathkeeper experimental mil-tech | Salvaged after Patrols, Sweeps, Cleansings | 0.60 | 0.15 | Appetite, Toxicity, Feral Drive | 220 €$ fenced; raises Attention |
| T5 Relic | Lunar reliquary fragments | Reliquary carriers in Cleansings; a handful per campaign | 1.00 | 0.40 | Random axis ×3, Plasticity | Unsellable: anyone holding it topside is traced |

**Feed or sell.** Every Pit shows two live numbers side by side: feed → "+X cells this shift" (from current intake and frontier demand) and sell → "+Y €$". The player never has to compute the trade-off; the UI puts both prices on the same unit. The salvage loop is the lore's irony made mechanical: the Oathkeepers' own gear is the richest feed, and feeding it makes the strain more dangerous.

**Scav runs.** A crew of 2–4 is gone one shift. Yield depends on stratum and crew Pasts (Nomads find more, Badges avoid patrols). Typical returns: Sump run 8–14 T1; Culvert run 6–10 T1 + 1–3 T2; Service Tunnel run 4–8 T2 + a chance of a part; Vault expedition (chapter 5) 2–6 T3. A run into unlit tunnels has a 10–20% feral encounter chance, higher next to a Bloom.

**Population costs nutrient.** A slimo standing on the Mat draws 0.004 N per tick from its tile, 0.5 N per shift, which is the feed value of three T1 units. Forty slimos eat what a Culvert scav run brings back; growth slows unless the Pits get more. Food paste bought topside (2 €$ per slimo per shift) lifts that draw entirely, which is why trade matters even before the Oathkeepers arrive.

**Build costs** are in scrap units plus tweaker-shifts and are listed with the structures. Refining at a Scrap Forge turns 6 T1 into 1 T2 over one shift, and 8 T2 into 1 T3; the Mat gets nothing from refining, so a Forge is the player choosing eddies over growth.

**What the surface sells.** Food paste, tools (build speed +50% for a shift), intel (reads the Attention gauge exactly; warns one shift ahead of a Sweep), weapons for the militia, road salt by the sack.

## Slimos

A slimo's body is set by how deep the chrome went before the vaccine turned it. Mod-Depth decides what they look like, where they can go and what they are for.

| Depth | Body | Can go | Role |
| --- | --- | --- | --- |
| D1 Skin-deep | Passes for human; mycelial veins show under the skin as a faint Mat Glow | Topside, with a coat on | Traders and spies; the only slimos who can staff a Seep Market in person. Fragile on the Mat (no bonuses) |
| D2 Limbed | Chrome limbs re-organified into muscle with Rebar-grey inclusions | Anywhere underground | Scavengers: +1 scrap per run tier, carry capacity 6 |
| D3 Core-modded | Torso and organs were chrome; half the body is Mat tissue | Anywhere underground | Tweakers: build at 2× speed on Node tiles, can sense σ (shown as a halo when selected) |
| D4 Deep | Barely a silhouette; cannot leave the Mat (σ < 0.2 damages them) | Along cords, riding them at 3× walk speed | Shepherds: tips within 4 tiles extend with +0.1 reserve bonus; Ivan is D4 |
| D5 Dissolved | A flesh pile | Nowhere | Cannot act. Can be fed to a Pit as a T2 unit; city Mood −5, Ivan comments every time |

**Pasts.** Gang, Nomad, Corpo, Badge (ex-NCPD), Civ. A Past gives one surface faction a reputation bonus when that slimo trades, and one field bonus: Nomads +2 scav yield, Badges see Patrols a shift early, Corpo knows the Vault layout (chapter 5), Gangs fight at +1 in the militia, Civs cost half the Mood penalty when things go wrong.

**Needs**, each 0–100 and visible on the slimo card:

- Fed: draws 0.004 N per tick from the Mat tile, or none on food paste.
- Lit: wants a light source within 6 tiles; Lume Bulbs or Lume Farms satisfy it, so do Night City leaks near grates.
- Dry: water on the tile or the one above drops it 10 per shift.
- Signal: standing on σ < 0.1 for a shift adds Ferality at the strain's Feral Drive rate.
- Mood: events, Toxicity, D5 feeding, deaths. Below 20 a slimo refuses work; a D3 or D4 with Ferality ≥ 1.0 becomes a sewerat on the spot and joins the nearest feral strain or starts one.

**Newcomers.** Flesh piles and half-turned bodies arrive at storm-drain grates after surface clinic days, every 3–5 days, 1–6 per batch. Each is a Mod-Depth, a Past, 2–6 units of embedded scrap at the tier their chrome was, and a name. Early in the campaign most are D1–D2; by chapter 5 the vaccine has reached the deep-modded and D4s arrive in numbers. A newcomer not reached and sheltered within two shifts of arrival goes feral in the drain.

## Building

Structures stand on Cord or Node tiles with σ ≥ 0.3 and nothing else. The Mat decides where the city can be; the player decides where the Mat goes by feeding and cutting. When a foundation tile necroses the structure loses 1 HP per tick until it is re-rooted (a tweaker shift) or gone.

**Steering tools** (the only direct hands on the CA):

| Tool | Cost | Effect |
| --- | --- | --- |
| Pit | 4 T1, 1 tweaker-shift | 3 × 3 stockpile the Mat draws from; the main steering tool, since growth follows supply |
| Bait Smear | 1 T1 | Paints 0.3 N onto up to 5 tiles; cheap directional pull, decays in a shift |
| Salt Line | 1 sack per 8 tiles (bought topside) | Tiles the Mat will not enter; Oathkeepers use bleach for the same thing |
| Severing | 1 tweaker-shift per cut | Removes one hyphal tile; splits strains; the quarantine tool |
| Relay Shrine | 6 T2, 2 tweaker-shifts, needs a Node | A built signal source, range 24 tiles |

**Structures**

| Structure | Footprint | Cost | What it does |
| --- | --- | --- | --- |
| Heart | 1 | given | The strain's origin Node and first signal source; one per city; relocatable only in chapter 6 |
| Hab-Pod | 2 × 2 | 6 T1 | Houses 4; satisfies Dry and part of Lit |
| Scrap Forge | 3 × 2 | 10 T1, 2 T2 | Refines T1 → T2 (6:1) and T2 → T3 (8:1); makes parts |
| Tweaker Bench | 2 × 2 | 8 T1, 1 T2 | Weapons and tools from scrap; arms the militia |
| Lume Farm | 2 × 2 on a Lume Bulb organ | 6 T1 | Harvests biolume as "cold light", a trade good; adds to Attention within 10 tiles of a grate |
| Lung Housing | 2 × 2 on a Filter Lung organ | 8 T1 | Clean water: Dry need for 12 slimos, lowers Flood level by 1 tile in its culvert; water is a trade good |
| Seep Market | 3 × 2 within 6 tiles of a Grate | 12 T1, 2 T2 | Trade post; needs a D1 to staff it; trade volume raises that grate's Attention |
| Sphincter Gate | 1 × 2 in a chokepoint | 4 T2, or free on a Sphincter organ | Seals a tunnel against water and squads; opens for slimos |
| Sewerat Pen | 3 × 2 | 10 T1 | Tames ferals that spawn within 8 tiles; holds 6 |
| Downer Still | 3 × 2 | 6 T2, 1 T3 | Distils Mat enzyme into Downer, the street palliative for cyberpsychosis; the alliance's economic engine and the Oathkeepers' first target |
| Reliquary Vault | 2 × 2 | 8 T2 | Stores T4 and T5 without the Attention leak they cause in the open |

**Organs are not built.** When a strain's gene crosses its threshold, the organ appears at a Node of its own accord (see Genome). A Lume Farm or Lung Housing is the tweakers putting a housing around something the Mat grew; without the organ there is nothing to build on. This is the steering-without-a-lock fantasy: you fed Street Chrome for three shifts to push Lume, and the Bulbs came up where the Mat decided they would.

**Placement rule for defense.** Squads move along open tiles and across cords; they cannot pass a closed Sphincter or a Salt Line of their own. Barricades are not a structure: the Mat's Chrome-Affinity cords are the walls, and a Hive-strain city is armoured by its own tissue.

## Trade and surface factions

Trade happens at Seep Markets under grates, staffed by a D1. Orders queue during the shift and resolve at its end; a market only trades with factions whose people use that street above it, so which grate you reach decides who you meet.

| Faction | Sells | Buys | Wants from you | Reputation lever |
| --- | --- | --- | --- | --- |
| Gangs | T2 scrap, weapons, protection (delays one Patrol) | Downer, cold light | A cut of Downer output | Gang-Past slimos; paying protection; never selling them out to Badges |
| Nomads | Food paste, salt, route intel; move goods laterally between your grates for a fee | Water, cold light, T1 bulk | Steady volume | Nomad-Past slimos; reliable orders over several shifts |
| Corpo remnants | T3 scrap, Vault intel (chapter 5), Attention readout | Downer at premium, T4 fenced | Discretion | Corpo-Past slimos; buying their intel; keeping Blackline in Vaults |
| Badges (ex-NCPD) | Patrol timing a shift early, weapons | Downer for officers going psycho | Not flooding their streets with Lume | Badge-Past slimos; low Attention at their grates |
| Oathkeepers | Chapter 3 only: a "sanitation accord" | Your T1 as tithe | Your map, in effect | None after chapter 3; later only through Corpo fences |

Reputation runs −3 to +3 per faction, moves prices 30% each way at the ends, and gates events (a Nomad convoy will only haul your goods at +1; Badges only warn at +2). Selling Blackline through anyone adds Attention at that grate, because the Oathkeepers track their own hardware.

**Downer** is the pivot of the second half. The Mat metabolises chrome toxins; a Still distils that enzyme into a street palliative that lets a chrome-heavy surface dweller keep their mods and stay level. It is the cure the Oathkeepers' vaccine pretended to be, sold from a sewer by the people the vaccine made. Every faction wants it, each Still raises Attention, and the alliance in chapter 6 is built on it.

**Glut.** A good's price falls 5% per unit sold at one grate in a shift and recovers 20% per shift, so a trade empire spreads across grates, which spreads Attention, which is the pressure the design wants.

## Threats

### Attention and the Oathkeepers

Each grate zone carries an Attention value 0–100. Per shift it gains: the sum of Lume gene values of Nodes within 10 tiles of the grate; 1 per 10 €$ traded; 5 per Blackline or Relic unit held outside a Vault anywhere in the zone; 8 per Still running in the zone. It loses 2 per shift on its own. Intel from Corpo contacts shows the exact number; otherwise the player sees a three-step gauge.

| Attention | Oathkeeper response | What comes down | Salvage |
| --- | --- | --- | --- |
| 30 | Patrol | Two scouts with white-light lamps; they map and leave. Killing them adds 15 Attention | 1–2 T4 |
| 60 | Sweep | A squad of 6 with bleach sprayers: nutrient set to 0 in a 3-tile radius along their path, cords they walk on take 1 damage per tick | 3–5 T4 |
| 90 | Cleansing | Three phases, below | 8–15 T4, 0–2 T5 |

**Cleansing** is a scripted-start, systems-run event of about two shifts:

1. **Bleach flood.** Oathkeepers open a sluice at the grate; for 20 ticks nutrient along connected Water tiles drops to 0 and Water rises 1 tile. Lungs cut the duration by a quarter each.
2. **White light.** Lamp carriers advance; every hyphal tile in their light takes 1 damage per tick, armoured cords (Chrome Affinity) absorb 1 + 9 · g before dying. Slimos in the light panic (Mood −20, flee).
3. **Sanitation.** Flamer squads target Pits first, then the Heart. If the Heart dies the strain loses signal; whatever survives beyond Relays goes feral within three shifts.

Defense is the Mat and the city together: Sphincter Gates close tunnels; Toxicity hurts anyone standing on cords; tamed sewerats rush lamp carriers; the militia fires from Benches; Salt Lines deny bleach a path by denying it the cords that would conduct it; decoy Lume Farms far from the Heart pull the squad the wrong way, since squads path toward the brightest Attention source. Ivan's ooze-time belongs here.

### Blooms (feral strains)

A strain with no signal source and Feral Drive above 0.5 spawns sewerats at its Spore Sacs, doubles Appetite, and spreads toward any hyphae it can smell (chemotropism aims it at your Pits). A Bloom is cleared by Severing its approaches, starving it (no Pits in reach, Salt on its deposits), or letting a Pen tame what it spawns. Sewerats attack slimos and chew structures at 2 HP per tick; tamed, they follow a D2 handler and bite Oathkeepers for 3.

### Floods

The Flood shift raises water in the lowest occupied stratum by the seed's roll, 0–3 tiles. Water conducts nutrient at 0.25, drowns Hab-Pods and Forges on flooded tiles, and makes a Sweep's bleach travel farther. Lungs lower the level; Sphincters hold it back; a Mat with Thrift above 0.55 grows Lungs on its own.

### Overgrowth

A fed Mat with Vigor and Branching high fills corridors with junctions and Nodes until scav crews cannot pass. Severing clears a path at a tweaker-shift per tile. The Mat does not eat the city while fed; Appetite is a starvation behaviour, which keeps the feed decision honest.

## Ivan Ooze

Ivan is two things at once: the voice that frames every chapter, and a unit on the map. He is a D4 Deep slimo by my call, which keeps him underground where the lore has him and gives him cord-riding mobility; his lume is the one Deep Purple light source in the world, so the player can always find him.

**Abilities (one unit, four verbs):**

| Verb | Effect | Cost |
| --- | --- | --- |
| Rally | Slimos within 8 tiles: +50% speed, Mood +10, for a shift | Cooldown one shift |
| Parley | Opens a faction's second-tier deals at a market he stands in; the only way to reach reputation +3 | He must be there in person, so a D1 cannot do it |
| Feed the Heart | A Pit he loads personally applies the tier's bias vector at 3× weight for that feed | Once per shift. He is the one who steers evolution on purpose |
| Ooze-time | Within 12 tiles of combat: time runs at 0.2× for up to 6 seconds while orders are given, Gates triggered, Salt laid | Cooldown one shift; the Max Payne nod made into a strategy verb |

If Ivan dies the campaign continues; his narration does not. Chapter cards go silent until a new Heart is seeded, and the Scattered ending becomes the only one left.

**Narration.** Between chapters, 3–5 graphic-novel panels with his voice-over (see Art direction). In play, text barks on events, hard-boiled and profane, never longer than two lines. Samples in the register the lore sets:

- First Patrol sighted: *White light in the culverts. The Moon sent a cleaning crew.*
- Selling Blackline topside: *Fence their gear up there and they'll smell it from orbit. Your eddies.*
- Feeding a D5 to a Pit: *He'd have done the same to me. Doesn't mean I'll sleep.*
- First Bloom: *That one's gone sewerat. It still has your face in its head somewhere.*
- Flood shift: *Rain up top. Down here that's a tide.*
- Chapter 3 betrayal: *They took the tithe, drew the map, and came back with bleach. Write it down somewhere.*

Barks are picked from pools keyed to the event and the current strain's dominant gene, so a Hive city hears Ivan talk about armour and a Garden city hears him talk about light; the pools are the writing budget's largest item after the panels.

## Campaign: the Rise

The world is six strata stacked under the street. The Mat climbs them over the campaign; that climb is the title.

| Stratum (bottom to top) | What is there | Scrap | Pressure |
| --- | --- | --- | --- |
| S1 The Sump | Black water, silt, the Heart; two grates far apart | T1 | Floods |
| S2 Culverts | Brick arches, pipe runs, first Seep Market | T1, some T2 | First Patrols |
| S3 Storm Drains | Newcomer flushes; water moves every Flood | T1–T2 | Blooms (cut-off strains) |
| S4 Service Tunnels | Power conduits, Badge patrol routes | T2 | Sweeps |
| S5 The Vault | Drowned corporate archive, Corpo scrap | T3 | Expedition hazards |
| S6 Undercroft | Metro and utility levels directly under Night City; neon leaks through every grate | T2–T4 | Cleansings; the finale |

**Chapters.** Each has one goal that the systems can satisfy in several ways and one beat the narration owns.

1. **Seepage.** Grow the Heart to 60 cords and shelter the first newcomer batch. Beat: Ivan's welcome, the panels that open the game.
2. **Tweakers.** First Forge and first Seep Market; the feed-or-sell gauge is introduced on a real decision. Beat: the first bark about eddies.
3. **The Hand.** An Oathkeeper envoy offers a sanitation accord: a tithe of T1 per shift for protection. Accept, and the tithe route maps the Heart and the first Cleansing comes in two shifts, large. Refuse, and it comes in one shift, small. Either way it comes; the choice is what the player loses first. Beat: the betrayal.
4. **Sewerats.** A scripted cut-off strain in S3 Blooms toward the city. Pens unlock. Beat: a named newcomer from chapter 1 goes feral in a drain the player did not reach.
5. **Warchest.** Expedition into the Vault: Corpo chrome, Corpo contacts, the first Still. Beat: what Arasaka left behind, told by a Corpo-Past slimo who worked there.
6. **Common Ground.** The Oathkeepers occupy Night City above. Faction envoys come down through the grates. An alliance council; the Heart may relocate to the Undercroft. Beat: the factions that hunted slimos asking for Downer.
7. **Rise.** The Great Cleansing meets the stockpile. Scored on biomass, armoured cords, tamed sewerats, militia, Still output and alliance reputation. Beat: the ending the genome earned.

**Endings** are read off the dominant genes and the alliance score, not picked from a menu:

| Ending | Dominant genes | What Slimecity becomes |
| --- | --- | --- |
| Garden | Lume, Thrift | Night City's light, water and cure utility; the grates stay open |
| Hive | Chrome Affinity, Conductance | An armoured Mat under the whole city; the warchest literal |
| World-Eater | Appetite, Feral Drive | The Mat comes up through the streets; the Oathkeepers lose and so does the city above |
| Scattered | Heart destroyed | Ivan narrates from a new sump; New Game+ with the old genome as the starting strain |

**Endless Sump** removes chapters; Attention and newcomer rate scale with Mat size, and the seed code is the whole run.

## Art direction: the CyberSewer pixel bible

The brief's two inspirations pull in opposite directions, a muted photo-noir and a saturated neon maximalism, and the resolution is a lighting rule rather than a compromise: the world is painted in the noir palette and the neon exists only as light. Every saturated pixel on screen is something emitting.

**Render.** 480 × 270 base, integer-scaled (4× at 1080p, 8× at 4K). Tiles 16 × 16. Slimos 16 × 24; Ivan and Oathkeeper lamp carriers 24 × 32; sewerats 16 × 16 with a 4-frame morph. Structures in multiples of 16. Idle animation 8 fps, actions 12 fps; the Mat pulses on the sim tick.

**SEEPAGE palette**, 33 colours in eight ramps. Delivered as `SEEPAGE.gpl` (imports into Aseprite and GIMP) with the swatch sheet below.

| Ramp | Colours (hex) | Used for |
| --- | --- | --- |
| Noir | Void 07070C · Sump 0E1018 · Culvert 171A26 · Concrete Night 232737 · Wet Concrete 343A4F | Ambient, shadow, concrete; Sump is the CanvasModulate colour |
| Industrial | Rust Gray 4A4F5E · Rebar 6B6F7B · Dust 8E9097 · Chalk B5B4B0 | Steel, chrome inclusions, bone |
| Mud | Silt 2A1F19 · Sewer Brown 4A3526 · Rust 6E4E34 · Oxide 9A7049 | Brick, muck, leather, dry necrosis |
| Bile | Bile 2E3A1E · Vomit Green 4E5E27 · Pond Scum 7A8A3A | Standing water, mould |
| Drain | Drain Blue 122238 · Cistern 1F3D63 · Flood Light 35679C | Moving water |
| Flesh | Bruise 5B3A4A · Flesh Dim 9A6070 · Flesh D99AA6 | Slimo skin; Bruise is fresh necrosis and Ivan |
| Mat (emits) | Mat Dark 0F2E1C · Mat 1F6E3A · Mat Lit 3FD66E · Mat Glow B6FFC8 | Cords, tips, nodes, veins, the signal pulse |
| Neon (emits) | Hot Pink FF2E97 · Cyan 19E6FF · Acid Yellow F6FF2E · Deep Purple 7A1FFF · Muzzle Orange FF6A1A | Night City leaking through grates, UI callouts, Ivan, gunfire and flame |
| White (emits) | Vestment White F2EFE4 · Reliquary Gold D4A52A | The Oathkeepers, and nothing else |

&#91;image: SEEPAGE palette swatch sheet, 33 colours in eight ramps\]

**Seven rules.**

1. **Surface rule.** Environment and bodies use Noir, Industrial, Mud, Bile, Drain and Flesh only.
2. **Emitter rule.** Mat, Neon and White colours appear only as light sources, light pools, UI and signage. Never as a surface colour in shadow.
3. **Chiaroscuro rule.** Ambient is Sump. Everything visible is lit by something the player can point at. Light pools have hard edges with a two-step dither falloff, no smooth gradients.
4. **Ink rule.** Characters and props carry a 1 px Void outline; environment tiles carry none. This is the Max Payne photo-against-ink contrast in pixel form: people are drawn, the world is photographed.
5. **Dither rule.** Ordered dither at 50% and 25% only, two tones, never three.
6. **Wet rule.** Every horizontal surface takes a 1 px highlight in the nearest light's colour. That single pixel is what makes the sewer read as wet.
7. **White rule.** Vestment White is the only pure white in the game. By chapter 3 the player's eye has learned that white means the Oathkeepers are in the tunnels.

**The Mat on screen.** Tips are 1–2 px strands in Mat Lit. Cords are 4–6 px with a Mat Dark core and Mat Lit edge; a pulse of Mat Glow travels outward from the Heart at 6 tiles per second along the signal, so the player can read σ by eye. Nodes are 12–16 px bulbs. Necrosis desaturates through Bruise to Silt over its 30 ticks. Chrome-Affinity strains show Rebar and Chalk flecks in their cords; Lume strains throw real light. Strain identity is a tint on the core colour so two strains at a border read as two.

**Slimos** are composited from parts so five depths times five pasts do not become 25 sprite sheets: base silhouette by Mod-Depth, skin from the Flesh ramp, an inclusion overlay (Rebar flecks, heavier with depth), a vein overlay animated in Mat Glow, and a clothing layer carrying the Past's cue in muted ramps (a Badge's jacket is Rust Gray with one Chalk stripe; a Nomad's coat is Oxide). Ivan is the D4 silhouette in Bruise with Deep Purple veins.

**Oathkeepers** are tall, 24 × 32, in Vestment White with Reliquary Gold trim, bulky Y2K military hardware, lamps on poles. Their light is the only white light; it casts hard shadows from the Mat's cords.

**Night City** is present only at the top of S6: street level as a strip of silhouettes with Hot Pink, Cyan and Acid Yellow bleeding down through grates as shafts. The Edgerunners saturation lives there and nowhere else, which is why it hits when the Mat finally reaches it.

**Cutscene panels.** Pixel-art stills at 480 × 270, 3–5 per chapter break, high contrast, hard ink outlines, Ivan's narration in boxes set in a condensed pixel grotesk. Transitions are hard cuts with a kinetic slide; Oathkeeper panels get a two-frame glitch.

**UI.** Diegetic labels pinned to objects in the world, slamming in with letters resolving from glitch (the Edgerunners kinetic-typography habit). Numbers in a pixel monospace in Cyan on Void; alerts in Acid Yellow; the feed-or-sell gauge on every Pit. Mutation events print the gene delta as a glitching ticker. Ooze-time desaturates everything except emitters, adds 2 px chromatic aberration, draws Muzzle Orange bullet trails and debris sprites.

**Audio direction** in one line: wet industrial ambience (drips, hum, distant trains) under a lo-fi dub pulse that follows the Mat's tick; Cleansings bring breakcore; the only clean sound in the game is the Oathkeepers' lamp hum.

## Godot architecture

Godot 4.x (4.4 or later), GDScript for game logic and UI, the Mat simulation in Rust as a GDExtension (a native library Godot loads; godot-rust/`gdext` is the binding). The split is deliberate: the sim is a tight loop over 55k cells with a BFS and a seeded RNG where determinism and 2 ms budgets matter, and Rust is already your stack. Everything that touches the player is GDScript.

**Scene tree**

```markdown
Main (Node)
├─ Sim (MatSim, GDExtension)          tick scheduler, owns all grids and the RNG
├─ World (Node2D)
│  ├─ Strata/ (one Node2D per stratum)
│  │   ├─ Substrate   (TileMapLayer)  rock, brick, pipe, open, water, grate, deposit
│  │   ├─ Mat         (TileMapLayer)  tip/cord/node/necrotic via terrain autotile; strain tint by alt-tile
│  │   ├─ Structures  (TileMapLayer)  multi-tile footprints
│  │   ├─ Decals      (TileMapLayer)  salt, bait, bleach stains
│  │   └─ FieldOverlay (ColorRect + shader)  nutrient underglow and signal pulse from two ImageTextures
│  ├─ Lights/      (PointLight2D pool, 64 active, nearest to camera)
│  ├─ Occluders/   (LightOccluder2D built from Substrate + Mat edges)
│  ├─ Units/       (slimos, sewerats, Oathkeepers; Node2D + grid A* from Sim)
│  └─ Camera2D     (pixel-snapped, 2×/3× zoom)
├─ UI (CanvasLayer)   diegetic labels, build menu, pit gauges, bark feed, panel player
└─ Audio
Autoloads: GameState, EventBus, Narrator, Palette, SaveSystem
```

**MatSim API** (the GDExtension surface, all other code goes through it):

- `step() -> int` runs one tick, returns the count of changed cells
- `changed_cells() -> PackedInt32Array` indices to repaint; the renderer touches only these
- `feed(x, y, tier, units)`, `sever(x, y)`, `salt(x, y)`, `bait(x, y, amount)`, `place_relay(x, y)`
- `nutrient_image() -> Image (R8)`, `signal_image() -> Image (R8)` for the overlay shader
- `strains() -> Array[Dictionary]` id, genome, biomass, has\_signal, is\_feral
- `digest_events() -> Array` structure and slimo damage from Appetite this tick
- `path(from, to, walker) -> PackedVector2Array` grid A\* with walker rules (slimo crosses cords, squad cannot pass closed Sphincters)
- `save() / load(bytes)`, `seed()`; the save is seed + input log + a compressed snapshot every 10 shifts

**Rendering the Mat.** The Mat layer is a terrain set with 47-tile blob autotiling per state (Tip, Cord, Node, Necrotic); strain tint is an alternative-tile modulate. The two R8 images from the sim feed one full-screen CanvasItem shader on the FieldOverlay: nutrient drives a dithered Mat Dark underglow on empty cells, signal drives the outward Mat Glow pulse along cords. Both update only when the sim reports changes.

**Lighting.** `CanvasModulate` set to Sump. Every emitter is a `PointLight2D` with a hand-drawn light texture: a hard-edged disc with the two-step dither baked in, so Godot's smooth falloff never appears. Shadows on, occluders from tile edges, so the Mat's cords cast shadows in Oathkeeper white light. Pixel sprites for structures and characters carry normal maps (`CanvasTexture` with a normal texture) so the chiaroscuro reads on a 16-px figure. Neon decals use light-only blending.

**Pixel-perfect settings.** Viewport 480 × 270; stretch mode `viewport`, aspect `keep`, integer scaling on; default texture filter Nearest; `snap_2d_transforms_to_pixel` and `snap_2d_vertices_to_pixel` on; Camera2D position floored every frame; no sub-pixel motion anywhere, motion smoothing off.

**Shaders** (all small, all CanvasItem): palette-lock post pass (quantise to SEEPAGE, so no interpolation ever introduces a 34th colour); field overlay; cord pulse; glitch text; ooze-time (desaturate non-emitters, 2 px chromatic aberration); wet highlight (1 px specular on horizontals driven by the nearest light's colour, which is what the wet rule costs at runtime).

**Data.** Strata authored as TileMapLayer scenes in Godot or imported from Tiled JSON; genome expression, tier tables, structure costs and bark pools as JSON resources so balancing never recompiles the Rust. One PCG32 stream in the sim; a second in GDScript for VFX and bark choice.

**Budget.** 16.6 ms per frame at 60 fps: sim ≤ 2 ms per tick at 2–4 Hz (so it is not even every frame), tile repaint via changed cells ≤ 1 ms, lights ≤ 3 ms with the 64-light cap, the rest is headroom for ooze-time's extra passes.

## Asset production plan

About 1,700 frames for the campaign, produced in the order the prototype needs them so the game is playable with placeholders at every stage. Aseprite with the SEEPAGE palette locked; normal maps generated (Laigter) and hand-corrected on hero sprites; 16-px grid throughout.

| Set | Pieces | Size | Frames (approx.) | Order |
| --- | --- | --- | --- | --- |
| Mat terrain | 47-tile blob × 4 states + 6 organs × 4 frames | 16 × 16 | 212 | 1st: the Mat is the game |
| Strata tiles | 6 strata × \~48 terrain + \~40 props | 16 × 16 | 530 | 2nd: S1–S2 first, the rest per chapter |
| Slimo parts | 5 depth silhouettes × 4 anims × 6 frames, + inclusion, vein and 5 clothing overlays | 16 × 24 | 420 | 3rd: D2 and D3 first |
| Ivan | 6 anims × 8 frames | 24 × 32 | 48 | 3rd |
| Oathkeepers | scout, lamp carrier, flamer × 4 anims × 6 frames | 24 × 32 | 72 | 5th |
| Sewerats | 3 variants × 2 anims × 6 frames | 16 × 16 | 36 | 5th |
| Structures | 13 × (4 idle + 2 damaged) | 32–48 wide | 80 | 4th |
| VFX | sparks, lume pulses, bleach foam, flame, bullet trails, debris, glitch | mixed | 120 | 6th |
| UI | 9-slice panels, gauges, icons, two pixel fonts | mixed | 60 | 4th |
| Panels | 7 chapters × 4 stills | 480 × 270 | 28 | last |
| Light textures | hard-edged dithered discs, 4 radii × emitter colours | 32–128 | 24 | 1st, with the Mat |

Two fonts: a pixel monospace for numbers and terminal text, a condensed pixel grotesk for callouts and narration boxes. Both at 8 px cap height so they survive integer scaling.

The Mat set is first because every other asset is seen against it, and because the autotile rules (which of the 47 blob tiles a cord state resolves to) are a design decision the artist and the sim have to agree on before anything else is drawn.

## Greybox milestones

Shapes before sprites; every milestone runs headless or in the engine with no hand-testing handed back. M0 is already done in Python; the Rust port is the first engineering task.

| Milestone | Deliverable | Proof |
| --- | --- | --- |
| M0 Sim proof | Mat CA in Rust (`matsim` crate) with a CLI that renders PNG snapshots; parity with the Python prototype on the same seed | Same seed, same cords/nodes/reach counts as `mat_evidence.py`; < 2 ms per tick on the full six-stratum grid |
| M1 Engine shell | Godot project; GDExtension loaded; one stratum rendered from the sim as coloured squares; camera, pause, 1/2/4 Hz; Pit, Sever, Bait, Salt as mouse tools | A recorded 2-minute run showing growth steered by Pits |
| M2 Economy loop | Scav runs (abstract), Seep Market, slimos as rectangles with needs, structures as rectangles with foundations; the feed-or-sell gauge | A seeded 10-shift run ends with a positive eddy balance and a living Mat under a scripted feed plan |
| M3 Threats | Attention per grate, Patrol/Sweep/Cleansing as scripted waves, Blooms from cut-off strains, Flood; ooze-time | A Cleansing on a seeded city with and without Gates shows different survival counts |
| M4 Lighting and first art | CanvasModulate, light pool, occluders, palette lock, the Mat terrain set and S1 tiles | A screenshot that obeys all seven rules |
| M5 Chapters 1–3 | Panels player, bark pools, the accord decision, newcomers | Chapter 3 playable end to end on two seeds |

After M5 the plan is content: strata S3–S6, chapters 4–7, the remaining asset sets, and a balance pass on the genome table against real play data.

## IP note and editorial calls

**Names.** Night City, Arasaka and cyberpsychosis are CD Projekt / R. Talsorian property; Ivan Ooze is a Saban/Hasbro character. The lore keeps them, so this design keeps them, and every proper noun sits behind a single string table so a commercial release can swap them in one place. Nothing mechanical depends on the names: "the city above", "the fallen megacorp", "the sickness" and a renamed narrator leave the design intact.

**Calls made in this pass without asking, all reversible:**

- Side-view cross-section rather than top-down, because "what seeps down below" is a vertical idea and the Rise needs a vertical axis.
- Ivan is a D4 Deep slimo who cannot surface; the lore places him below and a hero who could leave would undercut the city.
- Eddies as the surface currency; nothing in the lore names one.
- Slimos draw nutrient from the Mat, so population competes with growth; this keeps trade relevant before the Oathkeepers arrive.
- The accord in chapter 3 is a lose-less choice rather than a true branch; both answers lead to the Cleansing, which is what the lore says happened.
- Appetite is starvation-triggered only. A well-fed Mat never eats the city; a design where it did would turn feeding into an upkeep tax rather than a choice.
- Organs grow themselves; the player builds housings around them. This is the only place the design refuses the player direct control, on purpose.

**Open questions, kept to two:**

- [ ] Does the sim tick at a fixed 2 Hz with the player speeding the clock, or does speed scale ticks per second? Fixed tick is assumed everywhere above; it is the simpler determinism story.
- [ ] Does D5 feeding stay in? It is the darkest single decision in the game and the lore's "liquidation in more ways than one" argues for it; the Mood penalty and Ivan's comment are the guard rails.
