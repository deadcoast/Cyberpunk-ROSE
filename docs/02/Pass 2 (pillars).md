# Cyberpunk: ROSE — Game Design

Nov 7, 2026 · @heat

## What the game is

ROSE is a logistics empire builder. The player runs a city of thousands of autonomous slimos on a living transport network under Night City, automates its supply chains, writes the laws it governs itself by, and climbs six strata to the surface before the Oathkeepers burn it out. The Mat is the conveyor belt, the power grid and the ground at once; the city is the thing that moves on it.

One sentence of loop: haul scrap in, feed and route it, let the city run itself on the rules you set, spend the surplus to climb, and defend what you own when the white light comes.

**The six pillars are the spine.** Every system below exists to serve at least two of them; a system that served none was cut.

| Pillar | Core mechanic | Where the player feels it |
| --- | --- | --- |
| Reward | Haul reveals, mutation flips, organ discoveries, contract chains, stratum breaches | Every 10–30 s something lands; every 5–10 min something new exists |
| Ownership | A bred strain with a lineage and an export code; named slimos with histories; districts you named; laws you signed; grates you hold | The city looks like your decisions and nobody else's run |
| Automation | Cargo rides the cords by demand; quotas keep Pits fed; explored routes run themselves; organs act on rules; Nerve logic programs Relays | You stop doing a thing by hand and watch it keep happening |
| Self-governed | The Mat grows on its own rules; slimos pick their own jobs under district priorities; the Charter changes behaviour city-wide; factions and ferals have economies you do not control | The city surprises you inside the rules you set |
| Risk vs reward | Feed tier, scav depth, Attention, Relic feeding, breaching up, the accord, Dissolution | Every lucrative move has a tell that it is being noticed |
| Progression | Six strata, a three-branch tech tree gated by loot, strain classes at genome thresholds, population tiers, Ivan's authority, New Game+ strains | Each hour the city can do a thing it could not do before |

**Five systems, one game.** Substrate (the sewer geometry, water, grates), the Mat (the cellular automaton, Pass 1 spec unchanged), Logistics (cargo, quotas, routes, Nerve logic), Society (slimos, jobs, districts, Charter, unrest) and Surface (factions, Attention, Oathkeepers, newcomers). The Mat is one system of five. Pass 1 wrote it as if it were the game; here it is the floor the other four stand on.

**Genre neighbours** by loop shape only: automation builders for the conveyor-and-quota loop, colony sims for the autonomous population with needs, settlement survival games for laws under pressure. None of them has a transport network that grows, mutates and eats; that is the hook, and every pillar below is written against it rather than against them.

## The loops

Four timescales, each with its own question, its own reward and its own failure.

| Scale | The player is asking | What they do | What comes back | What goes wrong |
| --- | --- | --- | --- | --- |
| Seconds | Where is the pressure right now? | Read the Stack view: a district's pulse dims, a trunk cord goes white-hot with traffic, the Attention gauge at a grate climbs | A cargo pulse arrives, a label slams in, a slimo wakes and gets a name | A starving cord digests a Pod; a queue backs up at a market |
| Minutes (a shift, 60 s) | What do I do with this haul? | Sell or feed by tier; place a Pit, a Pod, a Still; set a quota; queue a market order; dispatch a crew | The haul reveal; eddies tick; a structure completes; the Mat races down a culvert you baited | Over-fed Mat mutates past the plan; over-sold Mat starves |
| Hours (a stratum) | Can I reach the next grate before the white light? | Breach a stratum with Relays; automate the routes that worked; sign a law; take a contract chain | A new stratum lights up; a new scrap tier; a new faction; an organ type nobody has seen | A Sweep takes a district; a Bloom eats the route home |
| A run (12–18 h) | What kind of city did I breed? | Steer the genome, choose alliances, build the warchest | A strain class, an ending, a strain code to carry forward | Scattered: the Heart dies; the strain survives as the seed of the next run |

**Action budget.** A shift is 60 seconds. The design target is that a competent player has 90–120 seconds of worthwhile decisions queued at any moment: a haul to split, two structures to place, one quota to adjust, a crew to send, a market order, a cord to sever. That gap is what makes pause worth pressing and what keeps the game from ever being watched rather than played. The Mat's autonomy is what generates new decisions; it is never a reason for the player to have none.

**The first ten minutes**, played at 2 Hz with no tutorial text beyond Ivan:

| Time | What happens | What the player learns |
| --- | --- | --- |
| 0:00 | Panels: Ivan's welcome, four stills. Cut to the Sump: the Heart, three slimos (a D2, a D3, a D1), a Pit with 6 Junk Chrome in it | Where they are |
| 0:20 | The Pit drains; the Mat's first tips race along the culvert, pulse visible | Feeding moves the ground |
| 0:50 | The D2 is standing on a scrap Deposit. Click: she scavs it. First haul reveal, 5 units, one Street Chrome among them | Scrap comes from the world and has tiers |
| 1:30 | The Pod ghost snaps only to Cord tiles. Place it; the D3 walks over and builds. Ivan: one line on tweakers | The city stands on the Mat |
| 2:30 | A grate is visible 40 tiles east. The Mat is 15 tiles out. Second Pit, baited east | Steering by supply |
| 4:00 | First newcomer batch drops at the grate: two bodies, one wakes, gets a name, has a Past. The other is D5 | People arrive from above; some are gone |
| 5:30 | The Mat reaches the grate. Market ghost appears; the D1 can staff it. First sell: 8 Junk for 32 €$. The feed-or-sell gauge shows what those 8 would have grown | The dilemma, on a real decision |
| 7:00 | A Patrol lamp appears at the far end of the culvert. White light; cords in it smoke. It maps and leaves. Attention gauge is now on screen | White means danger; being seen costs |
| 8:30 | The Mat's first Node matures into a junction and Ivan names it a district. Priorities panel appears: feed / build / trade | The city governs itself under your rules |
| 10:00 | Flood shift. Water rises one tile; the Pod is fine, the Deposit floods. Chapter 1 goal card: 60 cords, 8 slimos | The clock exists |

By minute ten the player has fed, scavenged, built, sold, named, been seen and been rained on, and has not read a wall of text.

## Pillar 1: Reward

Rewards are scheduled on three clocks and every one of them is a reveal, something the player did not know the exact shape of until it landed.

| Clock | Reward | How it is revealed | Variable element |
| --- | --- | --- | --- |
| 10–30 s | A haul lands | Manifest unrolls item by item, tier colours in order, rare items last with a longer beat; Blackline and Relic get their own stinger | Count and tier mix roll on the route's table; a crew's Pasts tilt it |
| 10–30 s | A cargo pulse arrives at a starving district | The district's cords brighten in a wave from the arrival point outward | Which district it chose (demand routing) |
| 30–90 s | A structure completes, a market order fills, a newcomer wakes | Label slams in; eddies counter runs; the newcomer's name and Past card | Past, Mod-Depth, embedded scrap |
| 2–5 min | A mutation event | The genome ticker glitches, then resolves one gene at a time; a new organ threshold crossing gets a card and a Strainbook entry | Which genes moved, by how much, and whether a threshold tripped |
| 5–10 min | An organ grows; a contract completes; a Patrol is repelled | Organ: a bloom animation at the Node and a codex unlock. Contract: faction rep tick and a chained follow-up offer. Patrol: salvage manifest | Organ type and position; contract chain length; T4 count |
| 20–40 min | A district forms; a law takes effect | District: Ivan names it, a hub glyph appears on the Stack view. Law: a signal wave propagates from the Heart across every cord | Which Node became a hub; what the law changes in each district |
| 1–3 h | A stratum breach | The camera follows the first tip up the shaft; the new stratum's lighting state reveals as the Mat lights it | New scrap tier, new faction at its grates, new hazard |
| Run | Strain class reached; ending; strain code | Class card with the lineage tree; ending panels; an exportable code | Which class the genome earned |

**Contract chains.** A faction contract ("deliver 40 Downer at the north grate in 3 shifts") completes into a follow-up with a higher ask and a higher payout; the chain breaks on a miss. Chains are the compounding mechanic: a 5-link Corpo chain pays triple the sum of five singles and unlocks that faction's intel tier.

**The Strainbook.** Every organ type, every strain class, every notable mutation (a gene reflecting off 1.0, a merge shock) is a card the player has or has not seen. Completion is tracked per run and across runs; it is the collection layer and it costs nothing but play.

**Feedback stack.** Every reward hits three channels at once: a light change on the Stack view (the world itself reacts), a kinetic label (the UI names it), a sound on the Mat's tick (the pulse carries it). Nothing rewards silently and nothing rewards twice in the same channel in the same second; the audio mixer ducks lower-tier events under a higher one.

**Loss framing.** The Attention gauge, a starving district's dimming pulse and a newcomer batch that will go feral in two shifts are visible losses in progress. They are what makes a reward feel earned instead of dispensed.

## Pillar 2: Ownership

Ownership is anything the player would recognise as theirs in a screenshot of someone else's run.

| What they own | How it becomes theirs | How it persists |
| --- | --- | --- |
| The strain | Bred by every feed decision; its genome history is a lineage tree with the feed events that moved it | Exported as a strain code; New Game+ starts from it; codes trade between players |
| People | Every slimo has a name, a Past, a Mod-Depth, a skill that levels with the job they do, a home Pod and a record (hauls, builds, kills, the Cleansing they survived) | A memorial wall at the Heart lists the dead with the shift they died |
| Districts | A hub Node Ivan names on formation; the player renames; each district has a tag glyph painted on its tiles in the player's chosen Neon colour | District names appear on the Stack view and in every bark about them |
| Grates | A market under a grate claims it; the claim is a painted sign on the street above, visible in S6 as a leak of the district's colour | Claimed grates are where factions come to you |
| Laws | The Charter is a board at the Heart listing every law Ivan has signed, with the shift and the vote | Laws travel into New Game+ as unlocked, not signed |
| Trophies | Oathkeeper lamps captured in a Cleansing hang in the Heart chamber and light it white; a Relic kept unfed sits in a Vault as a display | Cosmetic, permanent for the run |
| The look | Hab decoration from salvaged signage; captured Night City neon leaks re-routed into a district's lighting | Cosmetic unlocks persist across runs |
| The seed | Every run has a seed code; Endless Sump runs on a seed are comparable | Shareable; leaderboards per seed |

Two rules keep ownership from being decoration. First, everything owned has a mechanical shadow: a named slimo's skill is a real modifier; a district's tag colour is its light on the Stack view; a captured lamp is a real white light source that Oathkeeper scouts path toward. Second, nothing owned is given: no cosmetic is bought with eddies, every one is salvaged, bred, captured or earned through a chain.

## Pillar 3: Automation

The Mat is the conveyor. Scrap, water, Downer and cold light move along cords as discrete cargo packets, one tile per tick, routed by demand. The player never carries anything; they decide where demand is and watch the network answer it.

**Cargo routing.** Every consumer (a Pit below its quota, a Still short of enzyme, a Pod cluster short of water, a market with an open order) publishes a demand value. Packets at a source follow the signal-weighted distance field toward the consumer with the highest demand per tile of distance; ties break toward the Heart. The field is rebuilt every 20 ticks from the Mat's current shape, so a Severing reroutes traffic within a shift and a necrosed trunk shows up as a traffic jam the player can see before anything starves. Cords have capacity: 4 packets per tile; a trunk above capacity slows to half speed and glows white on the Stack view, which is the prompt to grow a parallel cord or raise Conductance.

**Quotas.** Each Pit, Still, Vault and Pod cluster carries a target ("keep 40 units, Junk first, never Blackline"). Hauls arriving at a market are split to quotas automatically by the priority order set on the market; anything above all quotas is the surplus the sell order draws on. The feed-or-sell decision becomes a standing policy the player tunes rather than a click per haul, and the first time they change a quota and watch the whole network re-weight is the moment the automation pillar lands.

**Routes.** A scav route is risky the first time: the crew walks into dark tunnel, the player sees nothing until they return or do not. A route that returned twice can be assigned to a crew on a loop; it then runs every shift without orders, with yield decaying 10% per repeat until the Deposit is spent, and the crew's Nomad and D2 skills slow that decay. Nomad convoys under contract haul between the player's own grates laterally, so two districts on different culverts share one market.

**Organ rules.** Organs act without orders on rules the player can edit: a Sphincter closes when white light is within 8 tiles and opens for slimos; a Lung runs when water in its culvert is above the Pod floor; a Bulb dims when the grate zone's Attention passes 50. The default rules are sane; the edits are where the game's depth sits for players who want it.

**Nerve logic.** A Relay accepts up to four rules of the form *when* \[reading\] \[compare\] \[value\] *then* \[action\], where readings are signal, nutrient, Attention, water, traffic, white light, strain id, and actions are sever here, open or close a Gate, flip a quota, raise a militia alarm, dim Bulbs, dump a Vault's Blackline into the nearest Pit. Rules propagate as a signal pattern, so a rule on one Relay can read a value 30 tiles away if a cord connects them. This is the circuit layer; a Hive strain with Chrome Affinity above 0.6 lets cords themselves carry the rule without a Relay.

**The handoff.** Late in a run, a strain with Plasticity and Conductance above 0.7 begins proposing its own reroutes: the Stack view shows a dashed cord where the Mat wants to grow a bypass around a jam, and the player confirms or severs. The organism automating its own logistics is the point the lore has been pointing at since the first panel, and it is earned by breeding, not bought.

## Pillar 4: Self-governed

Two meanings, both built: systems that run without the player, and a city that governs itself by rules the player writes.

**Systems that run without you** (each with its mechanism, not a mood):

| System | What it does on its own | Mechanism |
| --- | --- | --- |
| The Mat | Grows, starves, speciates, grows organs | Pass 1 CA spec, unchanged |
| Slimos | Pick jobs, eat, sleep, drift feral | Utility scoring per shift: each slimo scores open jobs in its district by skill match, distance and district priority, takes the best, works a shift, rests a dwell; needs below 20 override jobs |
| Districts | Elect a foreman, allocate labour, request cargo | A hub Node with ≥ 3 structures within 12 cords forms a district; the slimo with the highest skill total and no Ferality becomes foreman; the foreman posts jobs and publishes the district's cargo demand |
| Ferals | Rise and fall as a population | Sewerats spawn at Spore Sacs on a strain's Feral Drive, eat Deposits and structures, die of starvation when their strain starves; a Bloom with nothing to eat collapses on its own in 4–6 shifts |
| Surface | Prices, patrols, newcomer flow move | Night City has its own clock: clinic days, Oathkeeper deployments, gang wars raise T2 supply and drop its price; the occupation in chapter 6 halves food paste supply city-wide |
| Weather | Floods | Seeded rain table per day |

**The Charter.** Ivan signs laws. A law is a signal pattern that leaves the Heart and takes effect in each district as the wave reaches it, so a city with a long trunk sees the law arrive in the far district a shift later. Laws cost Authority (Pillar 6) to sign and Mood to keep. First set:

| Law | Effect | Cost |
| --- | --- | --- |
| Rationing | Slimo nutrient draw halved | Mood −5 per shift while active |
| Open Grates | Markets trade with any faction at +1 rep | Attention +50% at every claimed grate |
| Conscription | Every D2 is militia during a Patrol or worse | Scav yield −20% during alarms |
| Quarantine Protocol | Relays auto-sever any region that loses signal for a shift | Loses outlying cords to false positives |
| Dissolution Rite | D5 feeding allowed without the per-event Mood hit | A standing Mood −3; Ivan's barks change for the run |
| Tithe | The chapter-3 accord: 10% of T1 to the Oathkeepers | They map the Heart |
| Curfew | Slimos stay in Pods during Dark shift | Flood damage down, output down |
| Foreman's Cut | Foremen take 5% of district cargo as discretionary stock | Districts self-repair faster; your quotas fill slower |

**Unrest.** Mood below 30 in a district for two shifts: strike (no jobs taken). Below 20: defections to a feral strain if one borders the district, or to the Oathkeepers if a Patrol is in the zone (they take the slimo up; it does not come back). Below 10: the foreman petitions; refusing twice removes the district's hub and its structures lose signal priority. Unrest is how the city tells the player a law or a quota is wrong, in the only language a self-governing system has.

## Pillar 5: Risk vs reward

Every risk in the game has a tell the player can read before committing, and a payoff that scales with how far past the tell they go. A risk without a tell is a trap; there are none.

| Decision | The upside | The downside | The tell | Scaling |
| --- | --- | --- | --- | --- |
| Feed tier | Growth speed and organ unlocks rise with tier | Genome drift magnitude rises 40× from Junk to Relic; a stable strain can lose its class | The genome ticker shows the expected drift band before the feed confirms | Linear growth, geometric volatility |
| Scav depth | Higher strata and unlit tunnels carry higher tiers and blueprints | Feral encounters, drowning, Patrols; a lost crew is lost | Route card shows last-known hazard and the Bloom proximity | Each stratum up roughly doubles tier value and hazard |
| Attention spend | Lume, Stills and trade are the eddy engine | Patrol at 30, Sweep at 60, Cleansing at 90 | The gauge, exact with Corpo intel, three-step without | Payout per Attention point rises with each stratum |
| Salvage | Killing scouts yields Blackline | +15 Attention per kill | Scouts carry visible lamps; the count is known | Blackline is the richest feed and the biggest drift |
| Relic feed | A 0.40 drift with Plasticity bias; the only way to some organs | Reflection off a gene ceiling can land anywhere | Relic cards show the three random axes before the feed, not their sign | All-or-nothing |
| Breach up | New tier, new faction, new mechanics | Closer to white light; Attention baseline rises per stratum | The shaft shows the stratum above's light state | Each stratum up is the game's act break |
| The accord | Two shifts of calm and a smaller first Cleansing deferred | They map the Heart | Ivan says so in the panels | One-shot |
| Dissolution | A D5 becomes 0.30 nutrient of Street grade | Mood city-wide; a law to make it routine costs standing Mood | The D5 card shows the feed value | Per event or per law |
| Alliance | A faction's top-tier deals and its militia in chapter 7 | Its rival drops to −2 | Reputation deltas preview on the Parley screen | Permanent per run |
| Trunk capacity | Running a cord above 4 packets per tile keeps a quota fed | Half speed and a white-hot line that Patrols path toward | The Stack view | Continuous |

The scaling column is the design rule: a risk that scales linearly in both directions is a chore; the ones that matter pay linearly and punish geometrically (feed tier) or pay geometrically and punish once (breach up). The accord is the single one-shot risk and it is scripted because the lore says the betrayal happened.

## Pillar 6: Progression

Six tracks advance in parallel; no single one gates the others for long, so a player blocked on one is always moving on another.

**Strata** (the act structure): Sump, Culverts, Storm Drains, Service Tunnels, the Vault, Undercroft. Each breach introduces one mechanic and one scrap tier: Culverts bring markets, Storm Drains bring Blooms and newcomer flow, Service Tunnels bring Nerve logic and Sweeps, the Vault brings Corpo chrome and Stills, Undercroft brings the alliance council and Cleansings on Night City's doorstep.

**Tech tree**, three branches, researched at Tweaker Benches with scrap and gated by blueprints that drop in hauls, so research is also loot:

| Branch | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
| --- | --- | --- | --- | --- |
| Chrome | Scrap Forge, Bench, basic militia arms | Vaults, Sphincter housings, armoured Pods | Blackline weapons, lamp capture | Relic containment, the Hive conduit |
| Flesh | Pods, food paste trade, Pens | Downer Still, Mod-Depth medicine (D4s can leave the Mat for a shift) | Shepherd training (D4 growth bonus doubles), foreman schools | The Rite, the memorial, the end-run census |
| Mycelium | Pits, Bait, Salt, Relays | Quotas, Lung and Bulb housings, route automation | Nerve logic, organ rules, strain grafting (merge two strains on purpose) | The handoff: the Mat proposes its own reroutes |

**Strain evolution.** The genome moves with every feed; classes are thresholds it crosses, each with a perk and a Strainbook card. Garden (Lume and Thrift above 0.65: Bulbs cost no Attention within 3 tiles of a Lung), Hive (Chrome Affinity and Conductance above 0.65: cords are walls and wires), Runner (Vigor and Conductance: tips extend two tiles per step), Bramble (Branching and Appetite: Spore Sacs tame themselves within Pen range), World-Eater (Appetite and Feral Drive above 0.75: the Mat digests Oathkeeper squads, and your own districts at half rate). A strain can hold two classes; holding three is a Relic-only state and the Strainbook's rarest card.

**Population tiers.** Hamlet at 20 slimos, Quarter at 100, City at 500, Empire at 2,000. Each tier unlocks a district count cap, a second market per grate zone, and a title card. The game's name is literal: the Empire card is the 2,000th slimo waking up.

**Ivan's Authority.** Earned by chapter beats, repelled attacks and district formation; spent on laws and on Parley tiers. Authority is the only currency that never comes from scrap, which keeps the Charter from being bought.

**Meta.** A finished run exports a strain code and a seed; New Game+ starts with the strain as the Heart's genome and the Charter's laws unlocked. The Strainbook persists across runs. Endless Sump on a shared seed is comparable between players by Empire-tier shift count.

## Scale

The game is built for thousands of autonomous nodes on screen at once, and the visual approach is justified by that density or not at all.

**Targets at Empire tier**

| Thing | Count | Each one is |
| --- | --- | --- |
| Mat cells | 20,000–50,000 across six strata | A CA cell with four fields |
| Nodes | 2,000–5,000 | A junction; a potential organ or structure anchor; a district hub candidate |
| Slimos | 2,000–5,000 | An agent with a state machine, needs, a skill, a home and a job |
| Cargo packets in flight | 1,000–3,000 | A tier, a position on a cord, a destination |
| Structures | 300–600 | A footprint on Nodes with HP and a quota |
| Districts | 20–40 | A hub, a foreman, priorities, a tag colour |
| Markets | 10–20 | A grate claim with faction access and an Attention zone |

**Three views, one world.** The camera has three zoom stops and each renders the same data at a different level of detail (LOD: how much of an object is drawn at a given zoom).

| View | Scale | What a slimo is | What cargo is | What the Mat is | What the player does here |
| --- | --- | --- | --- | --- | --- |
| Street | 1 tile = 16 px, \~30 × 17 tiles | A 16 × 24 sprite with ink outline, veins, clothing cue | A 4 × 4 pulse with the tier's colour riding the cord | Autotiled cords, bulbs, organs, structures, lit and shadowed | Place structures, read a slimo, watch a haul land, fight a Patrol |
| District | 1 tile = 4 px, \~120 × 68 tiles | A 2 × 3 glyph in Flesh with one Mat Glow pixel | A 1 px pulse | Cords as 1–2 px lines, nodes as 3 px bulbs | Quotas, routes, severing, district priorities, Nerve logic |
| Stack | 1 tile = 1 px, the whole 192 × 288 stack | Not drawn; traffic per tile is drawn as light | Not drawn; counted into traffic | A circuit board: cords by signal, traffic as brightness, districts as hubs, white light as white | Read the city, spot the jam, the starving quarter, the Patrol coming down the shaft |

The Stack view is the game's signature image: a black cross-section of the city with its trade network drawn in light, thousands of movements collapsed into brightness, Night City's neon leaking down from the top edge, and the Oathkeepers' white moving through it. It is also the screen the player spends the most time on, which is why the back end has to make it true rather than painted.

**Backend that carries it** (revises the Pass 1 architecture; Rust GDExtension throughout):

- **Segment graph.** The CA's cord cells are compiled every 20 ticks into a graph of edges between Nodes; cargo and signal run on the graph, the CA runs on cells. The compile is incremental (changed cells only).
- **Flow fields, not pathfinding per agent.** Every consumer class (markets, pits, pods, benches, each district hub) has a multi-source breadth-first distance field over passable cells; an agent or packet steps to its field's next cell. Thousands of movers cost one array index each per tick; the fields are rebuilt only when the Mat's shape changed.
- **Agents as columns** (structure-of-arrays): position, state, dwell, home, skill, need vector, each a contiguous array; the per-tick update is a handful of vectorised passes, which is what the prototype below does in numpy and the Rust core does with iterators.
- **Packets as ring buffers per edge** with the 4-per-tile capacity enforced at the edge, so a jam is a number the Stack view can draw.
- **Jobs as a per-district priority queue**; a slimo's utility score is computed when it wakes, not every tick.
- **Render by LOD from the same arrays**: Street draws sprites for the visible window only; District draws glyphs from the position arrays; Stack draws a traffic heat texture the GPU scales.

**Budget.** CA 2 ms, graph and fields amortised under 1 ms, agents and packets 3 ms at 5,000 of each, render 6 ms; 12 ms of a 16.6 ms frame, at 2 Hz sim with per-frame interpolation of movers.

## Scale evidence

`rose_scale_proto.py` (delivered) grows a Mat across a full 192 × 48 stratum from one Heart and seven Relays for 500 ticks, then runs a logistics layer on it for 300 ticks: three Seep Markets under grates receive hauls every 15 ticks, cargo packets ride the cords to the nearest Pit by flow field, and 3,000 slimos cycle home → market → pit → home on their own state machines, moving along flow fields with half of them walking at half speed. The CA keeps running underneath every tick.

| Measure | Result |
| --- | --- |
| Mat | 1,435 hyphal cells, 204 Nodes, one strain across the stratum |
| Agents | 3,000 slimos; 5,000 also run |
| Deliveries in 300 ticks | 4,964 (3,000 agents); 8,371 (5,000 agents) |
| Cost per tick, everything | 3.9 ms at 3,000 agents; 4.1 ms at 5,000 (numpy, one thread, unoptimised Python loops in the CA) |
| Determinism | Same seed, same counts |

The render below is the two views that matter for this argument. The Stack view draws no slimo and no packet individually; it draws traffic per tile as brightness, and the city's trunk routes come out white-hot on their own because thousands of movers chose them. The Street view is the same moment at 16× with the glyph LOD: crowds on the trunk, cargo pulses on cords, carriers marked. Both are drawn from the same arrays.

&#91;image: Scale prototype: Stack view of one stratum with 3,000 slimos and cargo drawn as traffic light; Street view crop at 16x with glyph LOD\]

What this proves: the back end described above runs thousands of autonomous movers on a grown network at a cost that leaves most of the frame free, and the visual approach has something to show at that scale. What it does not yet prove: jobs beyond hauling, demand-weighted routing (the prototype routes to the nearest Pit), capacity jams, and districts. Those are the M1–M3 content of the Rust port, and their data structures are the ones listed.

## Visual design, revised

The 16-px pixel approach stays, and it earns its place by density: at Street view a crowd of forty slimos on a trunk cord reads as a crowd, at District view a thousand read as a river, at Stack view five thousand read as light. The palette and the seven rules from Pass 1 are unchanged (Pass 1 tab, Art direction). What changes is the ceiling.

**Depth.** The sewer is a cross-section with three parallax planes: foreground pipes and cables that occlude the view and drip, the mid plane where everything lives, and a background plane of cavern wall and distant tunnels lit by the district colours in front of them. Camera motion separates the planes; the world stops being a flat tile grid the moment it moves.

**Light as data.** Every emitter is a real light. A district's tag colour is its light; traffic on a trunk brightens it; a starving district dims; white is the Oathkeepers. The Stack view is this rule taken to the limit: no sprites, only emitters and traffic, a city drawn in light on black. Normal maps on structures and characters so a 16-px figure takes a hard shadow from a lamp carrier passing.

**Motion.** Thousands of things move every tick: cargo pulses, slimo glyphs, the Mat's signal pulse, water, drips, rain pouring down shafts during Flood, smoke off cords in white light. Motion density is the maximalism the brief asked for, delivered by count rather than by sprite detail.

**Hero resolution.** Chapter panels and the Strainbook cards render at 960 × 540, twice the world's native resolution, so faces and hands exist there and only there; the world stays at 480 × 270 so the density stays legible.

**Shaders added to the Pass 1 list.** Parallax plane compositor; traffic heat to light (Stack view); fluid surface for standing and rising water (reflects the nearest emitter, one-pixel highlight preserved); rain and drip particles bound to shafts and pipe joints; district tag paint (a stencil in the tag colour on claimed tiles, visible at all three zooms).

**What a screenshot has to show at each view**, as the acceptance test for the art: Street, a lit crowd with readable Mod-Depth silhouettes and one cargo pulse in frame; District, three districts in three colours with a visible trunk between them; Stack, the whole city as a circuit of light with one white point moving through it.

## Engine status: M1 built and verified

The Rust core exists, the Godot shell loads it, and a headless scene test runs 3,000 slimos through the extension and passes. Repository `rose-engine-m1.zip` (delivered); `./run.sh <godot>` does the whole chain.

| Piece | State | Proof |
| --- | --- | --- |
| `matsim` crate | Mat CA + nutrient field (Pass 1 rules), relays, signal BFS, flow fields, cargo packets with 4-per-tile capacity and demand-weighted routing, agent columns, PNG renders | 12 tests: determinism, bounded nutrient, growth, Runner > default reach, die-back, mass conservation on a chain, salt blocks growth, flow-field distances, capacity never exceeded, city determinism |
| `matsim` CLI | `grow`, `evidence`, `scale`, `bench` with in-place progress and a stats table | `out/evidence_seed11.png`, `out/scale_stack_seed11.png`, `out/scale_street_seed11.png` |
| `rose-godot` GDExtension | Class `RoseSim`: start, step, every player verb, cell and agent arrays, stats dictionary | Loads in Godot 4.5.1 headless (API 4.5 feature) |
| Godot greybox | Image-texture world renderer, traffic overlay, glyph LOD at Street view, camera with three zoom stops, tools on mouse, clock with pause and 1/2/4 Hz | `tests/smoke.gd`: 240 frames, 978 ticks, 3,000 agents, 8,169 deliveries, every verb applied, views cycled, step 0.66 ms, PASS |

**Measured cost, release build, one thread, 192 × 48 stratum.** CA alone 0.3–0.5 ms per tick; CA plus 3,000 slimos and cargo 0.7 ms; plus 20,000 slimos 1.0 ms. The six-stratum stack at the Empire-tier counts fits inside the 2 ms the Scale section budgeted with room to spare, which means the budget can be spent on districts and job scoring rather than on optimisation.

**Where the Rust run differs from the Python one.** Different random streams give a different Mat on the same seed, so counts are not the Python counts: the Rust default strain reaches 31 tiles with 185 cords at t=350, the Runner reaches 33 with 123 cords. The regime claims hold (fed growth, die-back on cut, Runner reaching farther) and are what the tests assert; exact-count parity would need the same RNG in both, which is not worth building since the Rust core is the reference from here.

**Next (M2):** jobs beyond hauling with per-district utility scoring, quotas and the feed-or-sell gauge, market orders, and the Stack view's traffic drawn from the GPU-scaled heat texture instead of per-pixel writes.

## Carried over, and the calls made here

**Unchanged from Pass 1** (its tab stays as the reference): the Mat cellular-automaton spec and parameters with their prototype evidence; the SEEPAGE palette, the seven rendering rules and `SEEPAGE.gpl`; the lore-to-mechanics map and the invented-register; scrap tiers, Mod-Depths, structures, organs, the Attention stages and the Cleansing phases; strata, chapters and endings; Godot pixel-perfect settings and the lighting setup. Where this tab names a different number or a different owner for a system, this tab wins.

**Superseded from Pass 1**: the framing of the game as the Mat alone; the Godot scene tree's unit layer (now the agent columns and flow fields above); the milestone list (M0 is now two prototypes; M1 is the Rust core with segment graph, flow fields and agents; M2 logistics and quotas; M3 districts, Charter and unrest; M4 threats; M5 lighting and the three views; M6 chapters 1–3).

**Calls made without asking, all reversible:**

- Cargo capacity of 4 packets per tile and half speed over it. The number is a starting point for the jam mechanic; the mechanic is the point.
- Demand routing picks the highest demand per tile of distance rather than nearest. Nearest is what the prototype runs; the difference is the first thing the Rust core adds.
- Authority as a non-scrap currency for laws. The alternative, laws for eddies, would let a trade empire buy its Charter.
- Population tiers at 20 / 100 / 500 / 2,000. The Empire card at 2,000 is the title made literal; the earlier tiers are spaced for a first run of 12–18 hours.
- Hero art at 2× native for panels and cards only. Mixing resolutions in the world would break the density argument.
- The handoff (the Mat proposing its own reroutes) is gated on two genes above 0.7 rather than a tech unlock, so it is bred.

**Open questions, two:**

- [ ] Do slimo jobs include combat as a job (militia as a standing role) or is combat always a Charter state (Conscription)? Written as Charter above.
- [ ] Is the Stack view the default view at game start, or does the player start at Street and unlock Stack at Quarter tier? Written as always available above; starting at Street for the first ten minutes is the alternative.
