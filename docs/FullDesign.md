# Cyberpunk: ROSE — Full Design

Nov 7, 2026 · @heat · v3.10 Nov 9: Phase B written to its numbers (PAC-020–028, research 07–13); Phase C: the slimo record (PAC-040), the colony's structures, zones, homes, districts, foremen and petitions (PAC-041, research 14), and the Charter and unrest carried in full from Pass 2 with numbers (PAC-042, research 15) · v3.11 Nov 9: cargo, quotas, jams, the Mat's clock and the movers (PAC-043, research 16)

## The game

ROSE is an extraction colony-builder. You send Runners up into the dark for loot and bodies, haul it back to a city that lives on a purple ooze network, decide item by item what gets sold, fed, crafted or kept, and spend the surplus to climb six strata to Night City before the Oathkeepers burn you out. The city runs itself on laws you sign while you are away on a run; the run is where you can lose everything you carried.

**The master loop**, one cycle of which is 20–40 minutes:

1. **Prepare** at the Heart: pick a route, a Runner, two followers, pack the rig from the Stash.
2. **Run** (real time, 8–15 min): unlit tunnels, containers, deposits, bodies, ferals, patrols, raiders. Extract through a grate or onto your own Mat.
3. **Haul** into the Stash: unidentified items, condition, weight. Appraise what you can; the rest waits for a skilled slimo or a Bench.
4. **Decide**, per item: sell (eddies now), feed (growth and mutation now), craft (gear or product), keep (collection, set bonus, later value).
5. **Build and govern**: place structures on the Mat, set quotas and job priorities, sign or repeal laws, answer the foreman.
6. **Breed**: the feed plan steers the genome; organs come up; the Strainbook fills.
7. **Withstand**: the Attention your wealth generates arrives as a Patrol, a Sweep or a Cleansing; the city fights with what you built.
8. **Rise**: a Relay reaches the next stratum; new routes, new items, new factions, new white light.

Steps 5–7 keep running while you are inside step 2. The run clock and the city clock are the same clock; a Flood that starts during your run floods your city too.

**What a session leaves behind.** A player who stops after one evening owns: a Stash with items they chose to keep, a Runner who levelled or died, a Mat that reached somewhere new, a law on the Charter, two or three new Scrapdex and Strainbook entries, and a district they named. Every one of those is visible on the Stack view the next time they load.

**The two feelings the loop is tuned for.** Hoarding: the Stash grid fills with things of uncertain value, prices move, and the sell-or-keep call is never finished. Amassing: the city on the Stack view gets denser, brighter and more armoured, and the player can point at the part of it each run paid for.

**Perspective.** Side-on cross-section in every mode. Runs are played at Street zoom with direct control; the colony at Street, District and Stack zooms; Stash, Market, Strain Lab and Charter are full-screen panels drawn in the same pixel style over a dimmed view of the city.

## Expedition mode

A run is a real-time, side-on traversal of unlit sewer under direct control, with the Mat's edge as the only safe ground. Everything in it is built to make the player weigh weight. The run is one Rust simulation (`run::sim`) stepped at 20 Hz from a logged input word; Godot draws it and never decides anything.

**Traversal.** Slimos are ooze: they cling to and crawl along any surface, so a Runner moves in four directions through every passable tile of the cross-section (open floor and air, water, grates) and shafts need no ladders. Gravity is a rule for dropped items and bodies, not for slimos. Movement is continuous at a speed in tiles per second (sneak 1.5, walk 3.0, sprint 5.0 — a walk crosses the 20-tile view in about seven seconds; the cadence instrument set these down from 2.5 / 4.5 / 7.0, which cleared a route in under three minutes; overweight halves sprint; legs at 0 halve everything and forbid sprint). Collapse and deposit tiles are solid until dug.

**The Route Board** is the one screen between the Heart and a run, and it embarks in one click: three route cards across the top, the memory strip of the selected route under them, the crew on the left, the rig on the right, DEPART. A route is a stratum and a region (Sump East in S1, Culvert North in S2, Storm Drain in S3 at launch; a Relay reaching a stratum opens its card — Phase E wires the Relay, until then the three launch cards are open) and its seed is fixed by the save and the stratum, so re-running a route learns it. A card states what can be known before a run and nothing that cannot: length as chunks and shift minutes; danger as an index of the authored spawns — each burrow brings the mean pack of 2 drawn from the stratum's mix, each creature weighted crawler 1, burster 1.5, maw 3; each chalk-marked chunk brings two scouts (weight 3) at the stratum's patrol chance — shown as the number and banded by the index per chunk, LOW < 1.5, MEDIUM < 2.5, HIGH ≥ 2.5 (measured over 60 seeds per launch stratum: S1 ≈ 1.2 per chunk, S3 ≈ 2.1, S2 ≈ 2.4 because the culvert chunks carry the most chalk); the share of the route remembered from earlier runs; the lockers and crates seen on them (never the total); the water share of its tiles and the grates on it (the Flood timing joins this line in Phase D); our corpses by Runner and day; followers stranded there. The memory strip draws the selected route's remembered tiles at one pixel per tile (448 × 24 at most), the Mat edge in Ooze Glow, grates in Acid, known containers in Cyan, corpses in Hot Pink; unseen tiles are Void. The crew panel lists the Runner and the colony slimos who can follow (Phase C brings the records; the three named Phase B slimos stand in), two follower toggles. The rig panel packs from the Stash itself — the hand weapon, four quick slots, backpack items as a row of icons to click in, the weight against carry in Cyan and Hot Pink when over — and what is packed leaves the Stash at DEPART and comes home with the rig. Entry is at the Mat's farthest cord in that direction or at a grate you hold.

**Map.** Assembled per run from authored 32 × 24-tile chunks, 8–14 per run, seeded with the save so a route can be re-run and learned. Chunks are glyph grids in `data/chunks.toml` (19 shipped across S1–S3; 7 per stratum at launch) with exits in canonical windows — rows 11–13 on the west and east edges, columns 14–17 on the north and south — so any two facing exits join; unjoined exits compile to brick. A chunk's weight is both its pick weight and its cap per route (a vault once, a culvert up to three times); nothing is ever placed west of the entry column, which is where the Mat is. Validation rejects partial exit windows, open border cells, entries without an east exit, and any chunk whose exits do not all reach one another without digging; a route must reach ≥ 95 % of its passable tiles from the Mat edge without digging and 100 % with every collapse dug. Water level follows the city's Flood clock.

**Light** is a 0–15 integer field per tile, dark by default. Static sources: broken fixtures emit 6, street grates 9 (Night City leaking down), Mat tiles 2 + Lume gene × 6. Dynamic sources: the Runner's lume, scouts' lamps (10, white), muzzle flashes (10 for a tick). Light spreads by an 8-connected integer Dijkstra (orthogonal 2, diagonal 3, water ×3); solids take the arriving level and stop it so lit wall faces read. Three bands: dark ≤ 2, dim 3–6, lit ≥ 7.

**Lume and sight.** A Runner's lume = veins (Mod-Depth 1–5) + gear light (lantern 6, lamp pole 8, visor −1) + the Mat under their feet, −3 while dimming veins (held, 6 stamina/s), halved while Soaked, never below 1 for any slimo. A tile is visible when it is in the Runner's symmetric field of view and either lit ≥ dim or within the see-radius 2 + lume + Eye/2: dark rooms are seen as far as your own glow, a lit pool from across the chunk. Symmetry means anything that can see the Runner can be seen by a Runner looking that way.

**Exposure**, the light gem: max(light at your tile, lume), one number on the HUD. It sets the **notice range**, the distance at which anything can notice you: 2 + exposure tiles (dark ≈ 3, dim ≈ 7, lit ≈ 14), standing still −1, sprinting +2, digging +1.

**Noise** is a 0–15 footprint stamped over the tiles a sound reaches — open 2 per tile (diagonal 3), pipes 1 (a run carries footsteps far), brick 3 (through a one-tile wall), water 4 (halved reach), rubble and deposits 4, rock and concrete block — and it fades 2 per tick. Loudness: sneak step 2, walk 4, sprint 7, dig 8 per tick, container open 3, body pickup 4, dropped item 5, extraction channel 9, melee swing 3 + gear noise, gunshot 8 + gear noise; water adds 1, grates 2. Sewerats hear at 2, scouts at 4. The footprint is drawn as a brief cyan dither on the tiles it reached, pipes included, so nobody learns the rules through failure.

**Fog** has three states per tile — unseen, remembered, visible — drawn as a Void ordered dither at the art's own pixel grid (solid, heavy, by light). Remembered tiles persist per route in the save (`route_memory`), so a route is learned across runs and a death still pays in map. A creature outside visibility but inside hearing range is drawn as a one-pixel stir in its tile's dither for the tick it moved.

**Loot sources**: lockers and crates (tiered by chunk), scrap deposits (dig 6 s, 3 s with a Dig Claw, loud, into a pile on the tile), newcomer corpses (what a person carries down here; the body itself carried out for a recruit, filling half the rig and slowing you), Oathkeeper drops after kills, sewerat salvage, caches hidden behind a Sever (your Mat can grow into a sealed room on a later run if you leave a Bait), and the Vault chunks in S5 behind locks. Tables are data (`data/loot.toml`): each source names its picks, a NoDrop chance, a condition band, a category pool with weights and one tier row per stratum; a pick resolves NoDrop, category, tier, then an item uniformly from that category and tier. Rows are steep and gate the top the way level does: Rare enters at tier 3, Blackline at 4, Relic only in caches; in S1–S2 a crate's +1 tier buys picks and condition, not a higher row, so S2 tops out at Uncommon. Notable picks (Rare and up) run on a pseudo-random distribution — the chance ramps by a constant each miss and resets on a hit — so dry streaks and hot streaks both stay short; the constant is solved per row so the average still lands on the listed share. The first three container opens of a route's first run roll one row up. Contents roll from (route seed, run nonce, source index): where a locker is can be learned from the route card, what is in it cannot. Measured: mean value per container doubles per stratum (S1 9 → S2 19 → S3 37 eddies).

**Tools buy certainty.** Crates are sealed corporate shipping: prying one with a Pry Bar is quiet (noise 3) and adds a pick, forcing it by hand is loud (6) with no bonus. Lockers are 30 % locked at tier 1, +5 % per tier; a Pry Bar opens one at noise 4, forcing by hand is noise 8 and half the contents lose 30 condition. The bar costs a 1 × 3 slot and 2 kg carried in before anyone knows if it will be needed.

**Opening and taking.** A source under the Runner shows its prompt; opening unrolls the manifest one item per 0.22 s, common first, the notable last on a longer beat, each line in its tier colour. Found items are unidentified: tier colour, category, size and weight show, never the name, and they sell blind at 40 % until appraised. Taking is into the rig by auto-place; what does not fit stays, so weight and grid are the limits, not a count. On extraction the rig is handed to the Stash and the intake is reported.

**Threats.** Every creature carries one awareness value 0–1: sight raises it by distance inside the Runner's notice range and only inside the creature's own field of view (the same symmetric shadowcast, so a creature sees the Runner only where the Runner could see it); sound at or above its hearing threshold sets a floor of 0.5 and a remembered position; it decays slowly and passes through every state on the way down. Four phases with visible tells: Calm (patrol or prowl), Suspicious ≥ 0.4 (walk to the last stimulus and look), Alert ≥ 0.8 (hunt the last known position, attack in reach), Search (Alert with no stimulus for 12 s: circle the last known position, then Calm). Ferals skip Suspicious.

| Threat | Body | Behaviour | Counter |
| --- | --- | --- | --- |
| Crawler (sewerat) | HP 18, prowl 2.5 / hunt 5.0 tiles/s, hears 2 | Hunts noise and lume in packs of 1–3 from a burrow; bites 6 (legs or torso) after a 0.5 s wind-up; flees to the burrow under 30 % HP | Darkness, melee from behind (×1.5), salt lines (never crossed) |
| Burster (sewerat) | HP 12, 3.5 tiles/s, hears 3 | Walks straight at you; 0.6 s fuse when adjacent, then bursts: 10 damage within 1 tile, Blinded 4 s, Soaked, Fungal Fever; killing it bursts it smaller (5, radius 1) | Range, salt, a thrown item to set it off |
| Maw (sewerat) | HP 60, 1.5 tiles/s, hears 1 | Holds a corridor and never leaves its chunk; bite 14 after 0.8 s; grabs and holds 1 s unless dodged | Go around; a tamed sewerat; two Runners |
| Oathkeeper scout | HP 30, 4.0 tiles/s, hears 4, lamp emits 10 | Two per patrol walking their chalk marks; map and leave after the last mark or 90 s unless Alert; shoot (9, one shot per 1.2 s, magazine 6, reload 3 s, noise 12) or strike with the pole (7); 1–2 Blackline and +15 Attention on death | Break line of sight; kill only when the salvage is worth the gauge |
| Lamp carrier | Sweep-tier | White light burns cords and slimo skin 1 HP/s inside its pool | Gates, corners, sewerats |
| Raiders | Surface gangs down a drain | Loot the same containers you want; fight each other | Let them thin out; loot their dead |
| Water | Flood shift floods low chunks over 90 s; drowning in 15 s | | Watch the clock on the route card |
| Gas pockets, collapses | Authored hazards | Lantern shows gas; collapses block a route for the run | Dig, or go around |

Creatures path by a distance field over passable tiles to their target and move continuously at their speed; a kill leaves a salvage source on the tile (scout drops from the `scout_drop` table, sewerats from `rat_salvage`).

**Combat** is deliberate. Stamina 100: sprint 12/s, swing 15, dodge 20, dimming veins 6/s; regen 10/s after a second without spending; overweight halves regen and sprint. Melee (pipe, chrome fist, shiv, sword) has a wind-up (0.3 s for a pipe) then an active window on the tile ahead (sword reach 2); damage is the gear's; from behind the target's facing ×1.5. Dodge is 0.35 s of invulnerability across two tiles in the facing direction. Ranged scrap guns are a line of sight up to 12 tiles with small magazines, a reload, and the loudest sound a Runner makes. Salt from a bag lays a line sewerats will not cross. Ooze-time is Ivan's and only on Mat tiles: 0.2× for 6 s.

**Followers.** Up to two colony slimos come along and fight without being told: fists (4) or the melee they carry, one HP pool of 60 % of a Runner's total, the same Down and Dead timers, and creatures hunt them as readily as the Runner (a creature takes the nearest target it can see). Three orders: Follow (the default: stay within three tiles, path around walls), Hold (stand), Guard (hold the tile they stand on and strike anything within three tiles, then return). A follower backs off below 30 % HP — it stops seeking creatures and only strikes one on an adjacent tile — a Coward below 60 %, a Fearless never. A Runner who goes Down is picked up by the nearest Follow-order follower within six tiles, carried toward the nearest extraction tile at 0.6× walk speed for up to twenty tiles and set down there; a carrier standing on an extraction tile holds the channel for both. Items dropped by the Runner (the last thing taken, onto the tile as a pile) are followers' business only when ordered to guard them.

**The Runner's body** is four sections — head 20, torso 50, arms 30, legs 30, scaled by Body — and every attack type lands on a section by its own odds (a bite: legs 40 / torso 40 / arms 15 / head 5; a shot: torso 50 / arms 20 / legs 20 / head 10). Armour on the matching slot subtracts. A section at 0 transfers further damage to the torso; head or torso at 0 is Down; arms at 0 double dig and reload time and halve melee; legs at 0 halve speed, forbid sprint and double the extraction channel. Statuses: Bleeding (bites and blades; 1 HP per 4 s to the torso for 60 s or until antiseptic), Blinded (sight radius 1 for 4 s), Chrome Shock (Blackline weapons; locks one quick slot until a Bench clears it), Soaked (20 s after leaving water; lume halved), Fungal Fever (from a Burster; persists into the colony).

**Extraction.** Reach your own Mat's edge, or a grate you hold, or a neutral grate for a toll (eddies or an item). Channel 10 s; noise spikes (9 per tick); followers within 4 tiles extract, the rest are stranded at the extraction tile and recorded on the route card for the next run. A run ends in an outcome record: extracted or died, ticks, the rig or the corpse, followers extracted / stranded / dead, the Mat haul, regrown gear, kills, chunks seen, route memory — the colony reads this, the run never reaches into the colony.

**The shift clock.** A run is one shift: 8 min + 70 s per chunk beyond eight, so 8:00 for the shortest route and 15:00 for a fourteen-chunk one, counted down on the HUD and printed on the route card. Past zero the clock counts overtime in Hot Pink and the sewer answers every minute of it: another pack crawls out of a random burrow, and from S2 a patrol of two scouts rolls onto a chalk-marked chunk that has none (the Flood joins the overtime in Phase D). Nothing ends the run but the Runner; overtime only makes staying cost more.

**The Run HUD** is minimal and puts the senses on the Runner, where the eye is. Around the Runner, in world space: the exposure arc (left half-ring, Ooze Glow, filled by exposure / 15), the noise arc (right half-ring, Cyan, filled by the loudest sound at the Runner's tile in the last 0.3 s / 15), the notice ring (a dotted circle at the true notice range, so the distance at which anything can notice you is a shape on the floor), and during a channel the channel ring (filled by progress: Ooze Glow for extraction, Dust for digging). Top centre: the shift clock in Cyan mono, Acid under a minute, Hot Pink in overtime; under it the route strip, one square per chunk in placement order, lit when seen, Cyan for the one you are in. Bottom-left: the body as a silhouette of four sections (head, torso, two arms, two legs), each filled by its HP in Ooze Glow, Hot Pink at half, flashing at zero; the stamina ooze bar beside it; statuses as tags under it (Hot Pink for damage states, Acid for alerts); the Down countdown when down. Bottom-right: the hand weapon and the four quick slots as icons with their keys, the used slot flashing. Right edge: a chip per follower with name, HP bar and order letter, Hot Pink when down. Bottom centre: one prompt line, the key in Cyan and the verb in Chalk — OPEN, PRY, TAKE ALL, DIG, HOLD TO EXTRACT — and the controls hint for the first six seconds of a run only. The loot panel is grid to grid: on open it slides in at the right (the camera eases east so the Runner stays clear of it) with the source's items unrolling one per 0.22 s as tiles at their real size in their tier colour on the left and the rig's 8 × 5 grid with its placed items on the right; TAKE ALL moves what fits across and tints what does not Hot Pink. Event labels (damage, DOWN, EXTRACTED) flash above the body they belong to.

**The cadence budget and its log.** Pillar one is measured, not asserted. Every run writes a cadence log — one line per event with its clock time, in three classes: **landings** (a reward arrives: a container, deposit pile or body opened — a *pull*, the slot-machine moment; a notable revealed; items taken; a kill; a Mat-edge drop; extraction with its blind value), **losses** (damage to a section, bleed ticks, Down, Dead, a follower down or dead, overtime) and **decisions** (an open, an engagement — the first swing or shot in ten seconds, a dig, a salt line, an order, a drop, the extraction). The report at the foot of the log gives the run's length, counts per class, the median and 90th-percentile gap between landings and between pulls, notables, damage taken, blind value and value per minute. The budget the log is checked against, per route at launch: a pull every 60–120 s (median); at least one notable per run from S3 and one per three runs in S1–S2 (S1–S2 miss this by the loot rows' own rule — Rare enters at tier 3 — so the S1–S2 notable is the first-run generosity and the route's one crate, measured 0 for the bot: a lever for the loot pass, not the cadence one); a loss event at least every two minutes while the Runner is off the Mat (pressure never lapses); and a bot death rate of 10–30 %. The landing budget of research 02 — something lands every 10–30 s — is the whole game's: the run's landings are mostly its pulls, the rest come from the colony's cargo and crafts, so the landing gap is reported per run and judged on the combined run-and-colony log from Phase C — a route that kills a cautious bot more than a third of the time is a wall, one that never kills it is a corridor. The bot walks the lit route from source to source and never waits, reads a manifest or hesitates, so the design takes it as three times faster than a cautious player and checks it against the budget divided by three (pulls every 20–40 s, landings every 3–10 s, quiet under 40 s); that factor is an assumption until a recorded human run calibrates it in the Phase F playtest, and it is printed on every report so nobody mistakes it for a measurement. It also carries a design meaning: a route offers about three times the pulls a shift consumes, so a run is a choice of what to leave and a route stays worth re-running.

Measured at PAC-028, twenty bot runs per stratum: at the Phase A speeds (walk 4.5) and dig 4 s the bot cleared a route in 2:25–3:20 with a pull every 7–8 s, half of them deposits (16 per route in S1); after walk 3.0 / dig 6 s and deposits thinned to one per chunk (two in collapse and hazard chunks) a run is 3:37–4:58 (×3 = 11–15 min, the shift), a pull every 11–12 s against the bot band of 20–40 s, deaths 20–30 %, longest quiet 42–70 s against 40 s. The instrument also caught the loot roll walking a map in process order, so two runs of one seed disagreed; the roll is sorted and a seed replays bit-exact again. The standing finding is source density: routes offer 10–12 containers and 10–12 deposits over 11 chunks (~2 pulls per chunk) where the budget wants about one; the chunk library's growth to forty is where it closes (target ≤ 1 container and ≤ 1 deposit per chunk on average, entry and nest chunks above), and the player factor is calibrated in Phase F. The log is written to the save folder (`user://telemetry/`), the player can read it, and `rose cadence` plays seeded runs with the bot (`run::bot`: open what it sees, take what fits, fight what is adjacent at over 40 % HP and back off under it, extract when the rig is nearly full, the shift is under 90 s, or the body is under 30 %) and writes the aggregate against the budget to `docs/telemetry/cadence.md`. Measured figures live in that file and the ones the budget hangs on are repeated here when a stage changes them.

**Death and recovery.** Down lasts 60 s untreated, then Dead. A dead Runner leaves a corpse at the tile of death holding the whole rig, identified; the route card marks it, and on the next run of that route it is a loot source like any other. Raiders roll once per colony day since the death at 10 % + 5 % per stratum; a raided corpse is stripped and the body stays. Several corpses may lie on one route. Items dropped on Mat tiles (the entry window and the two cord tiles beside it) come home as a separate Mat haul whether the Runner lives or dies. Gear flagged **Spored** at the Bench (a craft using Spore Sac tissue) regrows at the Heart on the Runner's death, once, and is removed from the corpse so nobody recovers it twice. Everything else is gone.

**What a run pays**: loot, newcomers, route memory, Runner XP, chunk intel (Corpo contacts buy maps), and salvage of the things that tried to kill you.

## Items

Every item has the same eleven fields, so every item can be sold, fed, crafted or kept and the UI never special-cases.

| Field | Values | What it drives |
| --- | --- | --- |
| Category | Chrome, Part, Organic, Tech, Gear, Contraband, Key, Curio, Relic | Which verbs make sense; which vendor wants it |
| Tier | Common, Uncommon, Rare, Blackline, Relic | Base value, drop weight, mutation drift when fed |
| Size | w × h grid cells (1×1 to 4×3) | Rig and Stash space |
| Weight | 0.1–12 kg | Stamina drain, noise, extraction speed |
| Condition | 0–100 | Sell price multiplier, craft yield, gear stats; decays with use and water |
| Identified | no / partial / yes | Unknown items show a silhouette and a tier guess; appraisal reveals name, value and feed vector |
| Found-in-run | tag | Contract eligibility, Corpo premium; stays until sold |
| Feed value | nutrient + 10-gene bias vector | What it does to the strain in a Pit |
| Craft tags | e.g. Conductor, Blade, Optic, Enzyme, Housing | Which Bench recipes accept it |
| Set | optional set id | Three of a set in the Stash gives a passive (see Curios) |
| Provenance | stratum, chunk, day | Codex entry, trophy text |

**Categories at launch**

- **Chrome (38 types)**: junk implants, street cyberware, corpo hardware, Blackline components. Feed value scales with tier; Chrome Affinity bias on anything with a Conductor tag.
- **Parts (24)**: pipe, wire spool, lens, servo, pump, filter cloth, cell, board, bearing. The craft backbone; low sell value, high craft value, which is the first sell-or-keep lesson.
- **Organics (14)**: food paste, rat jerky, clean water, mould culture, antiseptic, splint, stim, Downer dose. Needs and medicine.
- **Tech (16)**: data shards (sell to Corpo or read for route intel), blueprints (unlock research), access cards, maps.
- **Gear (31)**: melee (6), ranged (7), armour (6), tools (lantern, dig claw, pry bar, salt bag, lure, repair kit), implants (6, installed at the Bench, permanent, raise a stat, add a weakness).
- **Contraband (6)**: Downer, cold light, filtered water, ooze enzyme, Oathkeeper vestment cloth, Night City comms gear. Highest margins, highest Attention.
- **Keys (8)**: grate keys per zone, Vault cards, a lamp-carrier badge that lets one Runner pass a Patrol once.
- **Curios (22)**: the collection layer: pre-vaccine street signage, a corpo nameplate, a child's ooze toy, a chaplain's rosary of lunar stone. Sets of three give passives (Street set: +1 scav yield; Arasaka set: Vault locks open 2 s faster; Reliquary set: Attention decays +1 per shift).
- **Relics (5)**: lunar reliquary fragments. Feed value 1.0 with Plasticity bias; unsellable; one of them is the key to the Hive ending.

**Identification.** An unidentified item shows its silhouette, size and weight only. Appraisal happens by a slimo with Appraise skill ≥ the item's tier (instant, in the Stash), at a Bench with an Optic part (one shift), or by selling blind to a Gang fence at 40% of value. The identify moment is a reveal with a stinger per tier.

**Condition.** Water and use lower it; a Repair Kit and a Bench raise it. Below 20, gear jams and chrome loses half its feed value. A dropped item on a wet tile loses 5 per shift.

**Price.** base × condition/100 × faction demand (0.6–1.6) × glut (down 5% per unit sold at a grate per shift, recovering 20% per shift) × found-in-run premium (1.25 at Corpo). The Stash shows three numbers on every item: sell now, feed now (cells of growth at current intake), craft into (best recipe). Keep is the fourth verb and it has no number, which is the point.

## Stash, inventory and the Bench

**Rig.** Each Runner carries a backpack grid (6×4 to 10×6 by backpack tier; 8×5 until backpacks are gear), a chest rig with 4–8 quick slots (items of one or two cells, usable in combat; the 1 × 3 pry bar rides in the backpack and is found there), two hands, a head, a body and a feet slot, and four implant sockets. Weight above the Runner's carry stat (18 kg at Body 5) halves sprint and doubles noise. The rig is the run's inventory: loot is taken into it by auto-place and it is handed to the Stash whole on extraction.

**Stash.** The city's inventory lives at the Heart as a grid that grows with Heart tier: 12×10, 16×14, 20×18, 24×22. Containers (crates, lockers, a Vault) are items that hold items: they sort, filter and protect (a Vault hides Blackline from Attention). Sort by category, tier, value, weight, condition; filter to unidentified; a search box. Hovering an item shows the three numbers and its codex line.

**The four verbs** are on every item's card, in this order, as buttons: Sell, Feed, Craft, Keep.

- **Sell** queues the item to a market of choice. A market is where the Mat has reached a grate (the colony's market sites); with none reached the verb is closed and the card says why. At shift end the queue leaves the Heart as cargo (Colony mode, Cargo) and each item is paid when it lands at its market, at base × condition/100 × demand × glut × blind: demand is 1.0 until Phase D's factions move it; glut is −5 % per unit already sold at that market this shift (counted as they land), floor 25 %, recovering 20 % per shift; an unidentified item sells blind at 40 %. The quote counts what is queued and riding to that market ahead of it. A queued item sits in the queue panel and can be pulled back into the grid until the shift ends. Eddies land in the ledger on the save.
- **Feed** takes Chrome and Relics — the items with a feed value; the Mat does not eat the rest — to a Pit of choice (the Heart or a built Pit). At shift end it leaves the Heart as cargo, and when it lands the Pit's stock rises by the item's nutrient (halved below condition 20) and the genome drifts by the item's bias vector × the tier's drift (Common 0.01, Uncommon 0.03, Rare 0.07, Blackline 0.15, Relic 0.40), each gene clamped to 0–1. The card's "feed now" is cells of growth: nutrient / what a sprout costs its parent cell (grow cost 0.04 + tip starter 0.12 = 0.16), about six cells per unit of nutrient. A Pit's quota feeds it without a click (Colony mode, Cargo).
- **Craft** opens the Bench with the item in the first slot that takes one of its craft tags (recipes land with the Bench, PAC-027).
- **Keep** pins it: a pinned item is outside every bulk action and counts toward sets and the Scrapdex.

**Bulk actions.** The filter bar (category, tier, condition below N, unidentified) selects a view of the grid; the bulk bar applies Sell or Feed to every unpinned item in that view, showing the count and the eddies or nutrient it totals before the click — sell all unpinned Common Chrome, feed all Chrome below condition 20. Sort by category, tier, value, weight or condition. The grid draws every item as a tile of its real size in its tier colour with the icon centred, 32 px cells, scrolling when the Stash outgrows the screen; nothing is ever stretched.

**Shift end.** The run clock and the city clock are one clock, so the shift ends when a run ends: the queue and the quotas' pulls leave the Heart as cargo, and the Stash shows the shift report on the way back from the run — what landed during the shift (sold, earned, fed, drifted, spilled) and what has just left.

**Bench.** A Tweaker Bench has recipe tiers 1–4 (research) and a queue. A recipe takes items by craft tag rather than by name (Blade + Housing + Conductor = scrap sword, whose stats come from the parts' tiers and condition), so there are 60 recipes and thousands of outcomes. Crafted gear carries the Runner's name and the parts' provenance. Products: gear, implants, Downer, cold-light cells, filter cartridges, salt bags, Spored tags, repairs.

- **Recipes** live in `data/recipes.toml`: sixty over four tiers (15 / 17 / 16 / 12). Tier 1 is open at launch; tiers 2–4 open when a blueprint of that tier is researched (Phase E). A recipe is two to four slots; each slot names a craft tag and the stat it drives (dmg, armour, noise, light, mag, condition, or nothing — a binder). Any item carrying the tag fills the slot, whatever its category: a Rare Gold Trace is a conductor the same as a Common Wire, and that is the difference between the two swords.
- **Stats from parts.** A part's tier moves the stat its slot drives, measured against the recipe's own tier: ±15 % per tier above or below for dmg, armour, light and mag (clamped to 0.7–1.6 × base); −1 noise per tier above, +1 per tier below (floor 0). Product condition is the mean of the parts' condition with the housing slot counted twice. The outcome preview shows every number against the base before the click (SWORD dmg 13 → 15, noise 2 → 1).
- **Provenance.** A crafted instance records who made it (the Bench until Phase C names a slimo with Tweak), the day, and every part by name and tier; it keeps its own stats, and the run reads the instance, not the def. Identified, from the city.
- **Products** are an item id (gear, implants, parts, organics, contraband) with a batch count for consumables (salt bags and cells come in threes), or one of three jobs on a piece of gear already in the Stash: repair (+40 condition, capped at 100), identify (an Optic part on an unidentified item, one shift), Spored (Spore Tissue or a Spore Cap on gear: it regrows at the Heart once).
- **Queue.** A job takes its tier in shifts (1, 1, 2, 3) and finishes at shift end into the Stash, because the shift is the one clock. Parts leave the Stash when the job is queued; cancelling returns them. A full Stash leaves the product at the Bench until there is room.

**Appraise** is a Bench job and a slimo skill; a D1 trader with Appraise 4 identifies a Blackline item instantly, which makes that slimo worth protecting.

**Hoarding pressure.** The Stash is finite, prices drift, the Mat is hungry and the Bench wants parts. The design never forces a sale; it lets a full Stash with three unidentified Rare items in it sit on the screen when the next run is loading.

## Runners and slimos

Every slimo is a full character record; a Runner is a slimo you have taken on a run. The record is what the player loses when one dies, so it has to be worth losing.

**Stats (1–10):** Body (HP, carry), Nerve (stamina, dodge, lume control), Hands (melee, dig speed, craft quality), Eye (ranged, appraisal, fog radius), Mat-sense (signal reading, Shepherd bonus, cord riding; set by Mod-Depth), Voice (trade, foreman ability, Mood aura).

**Skills (0–10, XP by doing):** Scav, Fight, Tweak, Shepherd, Trade, Appraise, Medic, Guard. A skill at 5 unlocks its mid-tier job; at 8 its mastery perk (Scav 8: containers show their tier through fog; Tweak 8: one free part per craft; Shepherd 8: tips within 6 tiles extend at double rate).

**Mod-Depth** sets the body, the role ceiling and where they can go. D1 Skin-deep passes for human (veins 1; topside with a coat on; staffs a Seep Market; no Mat bonuses; Mat-sense 1). D2 Limbed (veins 2; anywhere underground; the scavenger body: Body +2, Hands +1, Scav +1; Mat-sense 3). D3 Core-modded (veins 3; anywhere underground; builds at 2× on Node tiles and senses σ; Hands +2, Nerve +1, Tweak +2; Mat-sense 5). D4 Deep (veins 5; Mat-bound — σ below 0.2 hurts them — and rides cords at 3× walk; Nerve +2, Body −2, Voice −1, Shepherd +3; Mat-sense 8; cannot run, since a run leaves the Mat). D5 Dissolved (a flesh pile; every stat 1; cannot act; can be fed to a Pit as a tier-2 unit at −5 city Mood and a line from Ivan). **Pasts** (Gang, Nomad, Corpo, Badge, Civ) give a stat lean and a field bonus: Gang Nerve +1, Body +1, Fight 2; Nomad Eye +1, Nerve +1, Scav 2, Shepherd 1; Corpo Eye +1, Voice +1, Appraise 2, Tweak 1; Badge Body +1, Voice +1, Guard 2, Fight 1; Civ Hands +1, Trade 1, Medic 1. A faction's reputation moves with its Past's slimos (Phase D).

**Creation.** A record rolls from the save's seed: depth by the campaign's weights at launch (D1 30 %, D2 40 %, D3 20 %, D4 8 %, D5 2 %; chapter 5 shifts the weight to the deep), a Past (Civ 30, Gang 25, Nomad 20, Corpo 15, Badge 10), every stat at 4 then the depth's and the Past's leans then three random points moved one at a time, clamped 1–10 (Mat-sense is the depth's); the field bonuses as skill levels; two traits from the forty with no excluded pair; a name from `data/names.toml` and, one time in four, an epithet; needs full and Mood at 60.

**Skills and XP.** Skill levels cost 10 + 5 × level XP each (325 XP from 0 to 10) and XP comes from doing, in the run outcome and the shift report: Scav 2 per pull and 1 per item taken; Fight 3 per kill and 1 per hit survived; Guard 2 per follower strike; Tweak 4 per craft finished; Trade 2 per sale resolved; Appraise 3 per item identified; Medic 3 per treatment; Shepherd 1 per shift on Tend. Teacher in the district multiplies XP by 1.25; a trait's own multiplier stacks. A skill at 5 unlocks its mid-tier job (Research needs Tweak 5, the Appraisal Table Appraise 3, the clinic Medic 5); at 8 the mastery perk (Scav 8: containers show their tier through fog; Fight 8: wind-up −0.1 s; Tweak 8: one free part per craft; Shepherd 8: tips within 6 tiles extend at double rate; Trade 8: glut recovers in one shift at that slimo's market; Appraise 8: identifies Relics; Medic 8: fractures heal in one shift; Guard 8: strikes two creatures a swing).

**Traits**, two per slimo, drawn at creation from forty in `data/traits.toml`, each a typed effect the crates read, never flavour alone: stat leans, a need rate (Fed, Lit, Dry, Signal, Rest multipliers), a Mood lean, a loved and a hated job (working the loved job +10 Mood target, the hated −15 — the genre's passion, research 03), a skill XP multiplier, rig rows, and flags the run and colony check. The forty: Hoarder (rig +2 rows, Nerve −1, loves Scav), Night Eyes (see radius +1, Lit need ×0.5), Loud (noise +1, Voice +1), Steady Hands (Hands +1, melee noise −1, Tweak XP ×1.5), Claustrophobic (Mood −10 in one-wide tunnels, hates Scav), Devout (Mood +10 near white light, defects at 25 not 20), Chromehead (feeds Chrome to themselves: Body +1 per five fed, Mood −5), Teacher (district XP ×1.25, loves Tweak), Glutton (Fed ×1.5, Mood +5 above Fed 80), Cold Blood (no panic in white light, Nerve +1), Pack Rat (Curio picks ×1.5, loves Scav), Iron Gut (Fed ×0.7), Light Sleeper (Rest regen ×0.7, loves Guard), Heavy Sleeper (Rest regen ×1.4, hates Guard), Brawler (Fight XP ×1.5, melee +1, hates Tend), Dead Shot (ranged +2, Eye +1), Quick (sprint +0.5, stamina regen ×1.2), Lumbering (walk −0.3, Body +1, carry +4), Mule (carry +6, hates Research), Frail (Body −2, Fed ×0.8, Appraise XP ×1.3), Webbed (wades and swims at walk speed, Dry ×0.5), Dry Skin (Dry ×1.5, hates Tend), Mat-born (Signal ×0.5, Mat-sense +1, loves Tend), Topsider (Signal ×1.5, Trade XP ×1.5, loves Appraise), Gloomy (Mood −8), Sunny (Mood +8, Voice +1), Fence (sells at +10 %, loves Haul), Tinkerer (Tweak XP ×1.5, loves Tweak), Healer's Hands (Medic XP ×1.5, treats at 2×), Coward (followers back off under 60 % HP, Nerve −1, Rest regen ×1.2), Fearless (never backs off, Nerve +1), Scrounger (deposit piles +1 item, Hands −1, loves Scav), Sharp Nose (notices creatures a tile farther, Eye +1 in the dark), Strong Back (carry +8, Body +1, Hands −1), Soft Voice (Voice −2, Appraise +1), Orator (Voice +2, election weight ×2, hates Scav), Hermit (Mood +10 alone in a Pod, −10 in a Bunkhouse, hates Guard), Pack Animal (Mood +10 in a Bunkhouse, bonds in two runs), Old Chrome (Body −1, Tweak +1, one more implant socket), Sewer Lungs (immune to Fungal Fever, Dry ×1.2). Pairs that cannot coexist are excluded in the data (Glutton/Iron Gut, Light/Heavy Sleeper, Coward/Fearless, Gloomy/Sunny, Hermit/Pack Animal, Quick/Lumbering, Mat-born/Topsider, Loud/Soft Voice). Traits are the reason two D2 Gang scavengers play differently.

**Needs** are six 0–100 gauges that move once per shift from what the slimo did and where it was, each rate multiplied by its traits: Fed −15 (a meal from the Mat on signalled tissue with nutrient, or two eddies of paste, fills it); Lit −10 in the dark and +20 in light at or above dim (D4 has no Lit need; a D1 loses 15); Dry −25 wet and +15 dry; Signal −20 off the Mat (D4 −40) and +25 on signalled Mat; Rest −12 working, −6 idle, +30 asleep in a Pod during Dark, +15 asleep anywhere else. Thresholds change behaviour: Rest below 20 halves speed; Fed at 0 costs a point of Body per shift; Signal at 0 on a D4 is a wound; Mood below 20 refuses jobs. **Mood** is a target the gauge moves toward at +4 / −3 per shift (research 03): the target starts at 60, every need below 40 takes 10 and below 20 another 10, every need above 80 gives 5, the loved job +10 and the hated −15, each injury −5, a death this shift −the dead slimo's Voice (×2 for a bonded partner), the trait's Mood lean, and from Phase C's structures the city-tier expectation and the Charter's laws. The unrest lines at 30 / 20 / 10 — strike, defection, petition — are read off the gauge by the Charter (PAC-042); the record only says which line it stands below.

**Jobs by utility.** Nine jobs (Haul, Build, Scav, Tweak, Tend, Guard, Cook, Appraise, Research), each read through one skill (Haul and Scav through Scav; Build, Tweak and Research through Tweak; Tend through Shepherd; Guard through Guard; Cook through Medic; Appraise through Appraise), and a 1–4 priority per slimo per job, 0 for never. A run is the Scav job for its Runner and the Guard job for its followers: the shift's Rest cost and the loved and hated leans read it. A slimo's utility for a job is None when the depth forbids it (D5 anything; D4 anything off the Mat — Scav, Guard at a gate, Cook; D1 Tend), when the skill gate is unmet, when Mood is below 20, or when Rest is below 20 and the job is not rest; otherwise priority × (0.6 + skill / 10) × the trait's affinity (loved 1.4, hated 0.5) × the need pressure (Fed below 30 doubles Cook, Rest below 40 halves everything but Tend). The colony hands each open job to the highest utility in reach; a district's priorities replace a slimo's when the foreman's are higher (PAC-041).

**The record in a run.** The Route Board hands the run a `RunnerSetup` read off the record, and the run's numbers bend to it by one rule per stat. Body scales the four sections and the stamina pool's twin, carry: sections × (0.8 + 0.04 × Body) (Body 5 is the table's numbers, 10 is ×1.2, 1 is ×0.84) and a carry limit of 18 kg + 1 kg per point of Body above 5 (−1 below) + the traits' carry. Nerve scales stamina the same way: 100 × (0.8 + 0.04 × Nerve), so Nerve 10 is 120 stamina and Nerve 1 is 84; a bonded partner within 6 tiles is +1 Nerve, +4 stamina. Hands moves melee damage by (Hands − 5) / 2 rounded toward zero (Hands 10 +2, Hands 1 −2, never below 1) and dig time by × (1.25 − 0.05 × Hands) (Hands 5 the table's 6 s, Hands 10 4.5 s, Hands 1 7.2 s). Eye is already the see radius (2 + lume + Eye / 2) and moves ranged damage as Hands moves melee. Veins are the depth's and set lume. The backpack is 8 × (5 + the traits' rows), 3–8 rows. The trait flags the run reads: Night Eyes see radius +1; Loud every Runner sound +1; Steady Hands melee noise −1; Brawler melee +1; Dead Shot ranged +2; Quick sprint +0.5 tiles/s and stamina regen ×1.2; Lumbering walk −0.3; Webbed wades and swims at walk speed; Scrounger a dug pile rolls one more item; Sharp Nose Eye +1 on a tile darker than dim; Sewer Lungs no Fungal Fever; Coward and Fearless the follower back-off line above. The rest of the forty are the colony's (PAC-041/042) or the Bench's. Dodge and dimming read Nerve in Phase D's combat pass; until then they are the table's.

**Injuries** persist: Bleeding (treat with antiseptic), Fracture (splint; −Hands or −legs for 3 shifts), Chrome Shock (locks an implant socket until a Bench clears it), Soaked, Blinded, Fungal Fever (from a Burster; Ferality gain doubles until a Lung treats it). A Cook at a Lung Housing — the Cook job reads the Medic skill, and the clinic's slot asks Medic 5 — treats injuries over shifts; without it they heal at a quarter rate.

**Gear slots**: head, body, feet, two hands, chest rig, backpack, four implants. Implants are permanent and visible on the sprite.

**Death.** HP 0 is Down; 60 s untreated is Dead. Dead Runners go on the memorial at the Heart with their record; their corpse and gear stay in the run chunk. A district whose foreman dies holds a one-shift wake (no jobs); Mood falls by the slimo's Voice.

**Recruitment.** Newcomers at drains (free, random record), rescued bodies on runs (random record, loyalty bonus), defectors from raider groups you spared (Gang Past, Fight-heavy), hires through faction contacts at reputation +2 (expensive, specialised). Population is the growth cap of everything else, which is why bodies are worth carrying out.

**Bonds.** Two slimos who complete three runs together (two for a Pack Animal) gain a bond: +1 Nerve when within 6 tiles of each other, and the survivor of a bonded pair takes a Mood hit twice the normal size. No romance system; bonds are a run mechanic. The roster, its records, its priorities and its bonds live in the save (v6) and the Route Board's crew is the roster's eligible slimos: D1–D3, up, not already out.

## Colony mode

The city has every system a player of the genre expects to find, and each one is bent by the fact that the ground is alive.

**Feature baseline and the ROSE version of each**

| Genre feature | In ROSE |
| --- | --- |
| Zones | Stockpile zones with filters (by category, tier, condition), hab zones, quarantine zones, kill zones; a zone must sit on signalled Mat |
| Jobs and priorities | Nine jobs (Haul, Build, Scav, Tweak, Tend, Guard, Cook, Appraise, Research) with a 1–4 priority per slimo per job in a grid; district priorities override when the foreman's are higher |
| Needs | Fed, Lit, Dry, Signal, Rest, Mood; each 0–100 with thresholds that change behaviour (Rest below 20: half speed; Mood below 20: refuses jobs) |
| Schedules | Three shifts per day, Dim / Dark / Flood; per-slimo schedule rows (work / rest / any); Curfew law overrides |
| Storage and logistics | The Mat carries cargo from and to the Heart; quotas pull; 4 packets per tile; jams white on the Stack view (Cargo, below); haulers carry what cords cannot, to stockpile zones off the Mat (PAC-044) |
| Power, water, light | Signal is power (structures need σ ≥ 0.3); Lungs are water; Bulbs and Lume Farms are light; all three are grown or housed, never plopped |
| Research | Three branches, four tiers, blueprints from runs gate tier entry, scrap pays the rest; researched at a Bench by a slimo with Tweak |
| Crafting | Bench recipes by craft tag (see Stash) |
| Medical | Lung Housing as clinic; Medic job; injuries from runs are the main patient source |
| Animals | Tamed sewerats: guard (pen), pack (carry 2×2 of cargo on runs), hunter (clears a Bloom's crawlers) |
| Defense | Sphincter Gates, salt lines, scrap turrets (Chrome tier 2 research: a mounted scrap gun with a Guard slimo), tamed sewerats, Toxicity gene, Ivan's ooze-time; militia from Conscription |
| Events | Newcomer flush, Flood roll, cave-in (blocks a culvert, yields Parts), trader convoy, Patrol/Sweep/Cleansing, Bloom, defection, a Relic signal (a Reliquary carrier spotted), gang war topside (T2 prices drop), clinic day (bigger flush) |
| Mood and unrest | District Mood from needs, laws, deaths, Toxicity; strikes, petitions, defections |
| Alerts | A queue on the right edge: red for loss in progress, yellow for a decision, white for Oathkeepers; each alert jumps the camera |
| Overlays | Nutrient, signal, light, noise, danger (last known threats), zones, traffic |
| Building | Footprints on Cord/Node tiles only; ghosts snap; tweakers build at Hands speed; structures have HP, three visual tiers, upgrade in place |
| Camera | Street / District / Stack; follow a slimo; bookmarks 1–9 |

**What the genre does not have**

- The ground grows, starves, mutates, speciates and eats; Pits, Bait, Salt and Severing are the steering tools; organs grow themselves at gene thresholds.
- Districts self-govern: a hub Node with three structures elects a foreman; the foreman posts jobs and cargo demand and petitions you.
- The Charter: laws propagate as a signal wave; eight laws at launch (Rationing, Open Grates, Conscription, Quarantine Protocol, Dissolution Rite, Tithe, Curfew, Foreman's Cut) with costs in Mood and Authority (the table under Colony mode).
- Nerve logic: up to four when/then rules per Relay over signal, nutrient, Attention, water, traffic, white light; actions sever, gate, quota flip, alarm, dim, dump.
- Attention: wealth and light you show at a grate bring the white light down; three stages, each with salvage.
- The handoff: a strain with Plasticity and Conductance above 0.7 proposes its own reroutes.

**Building.** A structure stands on the Mat by its anchor — the top-left tile of its footprint — which must be Cord or Node carrying signal at or above the structure's need (0.3 for most, 0.5 for the organs: Lume Farm, Lung Housing, Reliquary Vault, Scrap Turret); the Mat is filaments and nodes, not floors, so the rest of the footprint rests on walkable sewer floor (open, pipe, brick, grate, deposit; never rock, concrete, water or a collapse) or on more Mat. The ones marked *Node* need a Node under the anchor, a Seep Market a grate within two tiles, a Relay Shrine a Relay. Footprints never overlap. Placing makes a ghost; the first Build shift pays the cost out of the Stash — Parts are any Parts-category items (the cheapest first), Chrome any Chrome-category items, eddies from the ledger — and a ghost the Stash cannot pay for waits and says what it lacks. Building is the Build job: every slimo on Build adds their Hands in build points per shift (a Scrap Forge in the district doubles them); a structure is up when its points are met. Three visual tiers (the Heart four), upgraded in place: tier 2 costs twice tier 1 and needs the Quarter, tier 3 three times and needs the City; the Heart's tier is the city's. HP is 100 × tier (the Heart 200 × tier); floods and raiders take it (Phase D), a Build slimo mends 10 a shift.

**The 24 structures** (`data/structures.toml`; cost is tier 1 in Parts P, Chrome C, eddies E; the slots are the job the structure hosts and how many per tier):

| Structure | Footprint | Cost · build points | Hosts | Tier 1 / 2 / 3 |
| --- | --- | --- | --- | --- |
| Heart | 3 × 3 Node, prebuilt | — | — | Stash grid 10 × 6 / 12 × 8 / 14 × 10 / 16 × 12; Pit stock 40 / 80 / 160 / 320; lights 4 tiles; a Vault slot per tier from 2 |
| Pit | 1 × 1 | 4 P · 8 | Tend 1 | stock 20 / 40 / 60; a tended Pit releases × (1 + Shepherd / 10) |
| Hab-Pod | 2 × 2 | 6 P · 10 | — | one bed; Rest +30 asleep in Dark; a home a tier under the city's costs 5 Mood target per tier |
| Bunkhouse | 3 × 2 | 10 P · 16 | — | 6 / 8 / 10 beds; Rest +20 asleep in Dark; Pack Animal +10, Hermit −10 |
| Scrap Forge | 2 × 2 | 8 P 2 C · 14 | Build 2 / 3 / 4 | build points × 2 in its district; mends 10 HP a shift per builder |
| Tweaker Bench | 2 × 2 | 8 P 2 C · 14 | Tweak 1 / 2 / 3 | a slot finishes one Bench job a shift; tier 2 opens recipe tier 2, tier 3 tier 3 (with the blueprint) |
| Lume Farm | 2 × 2 Node, σ 0.5 | 6 P 4 C · 16 | Tend 1 | light within 4 / 6 / 8 tiles; +2 Attention a shift at a grate within 10 |
| Lung Housing | 2 × 2 Node, σ 0.5 | 10 P 4 C · 18 | Cook 1 / 2 / 3 (Medic 5) | the clinic: each Cook here treats one injury a shift (Medic XP); brine 1 / 2 / 3 a shift; homes within 6 tiles count dry |
| Seep Market | 2 × 2 at a grate | 8 P · 12 | Haul 1 / 2 / 3 | the ledger's market; a slot sells (Trade XP); glut recovers +5 % a tier faster |
| Sphincter Gate | 1 × 1 | 4 P 2 C · 8 | Guard 1 | HP 100 / 200 / 300; shut to creatures (Phase D) |
| Sewerat Pen | 2 × 2 | 6 P · 10 | Tend 1 | 2 / 4 / 6 tamed sewerats (Phase D) |
| Downer Still | 2 × 2 | 10 P 4 C 20 E · 18 | Cook 1 / 2 / 3 | a slot turns 1 brine + 1 Enzyme-tag Part into 1 dose a shift; +8 Attention a shift |
| Reliquary Vault | 2 × 2 Node, σ 0.5, Heart 2 | 12 P 6 C · 20 | — | 4 / 8 / 12 Relics or Blackline, outside Attention |
| Relay Shrine | 1 × 1 on a Relay | 6 P 2 C · 10 | — | the Relay's Nerve rules (4, Phase D); the hub of its district; members +5 Mood target |
| Stockpile Rack | 2 × 1 | 4 P · 6 | — | anchors a stockpile zone of 8 / 12 / 16 cells |
| Cold Store | 2 × 2 | 8 P 2 C · 12 | — | 8 / 12 / 16 cells; Organics inside lose no condition |
| Scrap Turret | 1 × 1, σ 0.5 | 6 P 4 C · 12 | Guard 1 | 6 / 8 / 10 damage at 6 tiles with a Guard in it (Phase D) |
| Watch Post | 1 × 1 | 4 P · 6 | Guard 1 | danger overlay 12 / 16 / 20 tiles |
| Memorial | 2 × 1 | 4 P · 6 | — | the dead's records; a death's Mood hit is halved for members of a district with one; 10 / 20 / 40 names |
| Charter Board | 2 × 1 | 6 P · 8 | — | the district's voice on the Charter: its foreman votes on laws and petitions; one per district |
| Appraisal Table | 2 × 1 | 6 P 2 C · 10 | Appraise 1 / 2 / 3 (Appraise 3) | a slot identifies 2 items a shift |
| Route Board | 2 × 1 at the Heart, prebuilt | — | — | crew 1 + 1 / 1 + 2 / 1 + 2 and a pack sewerat; cards per stratum open |
| Lantern Post | 1 × 1 | 2 P · 4 | — | light within 3 tiles without Lume; 1 E a shift for oil, dark when the ledger is dry |
| Salt Works | 2 × 2 | 8 P · 12 | Cook 1 | 1 brine into 1 / 2 / 3 salt a shift |

**Zones** are painted cells on signalled Mat (σ ≥ 0.3, Cord or Node) or on walkable floor touching it: a **stockpile** (around a Rack or Cold Store, with a filter — categories, a tier floor, a condition floor — that decides what haulers bring; one stack a cell; the store and its haulers land with the Colony screens, PAC-044), a **hab** zone (beds inside it are assigned before any other), a **quarantine** zone (a slimo with Fungal Fever sleeps and works inside it), a **kill** zone (Guards strike and turrets fire at anything inside without an order). A cell belongs to one zone.

**Homes, meals, light, water, sleep.** Each slimo takes a bed — in a hab zone first, then the nearest free bed to the Heart; a Hermit takes a Pod over a Bunkhouse and a Pack Animal the reverse — and sleeps on the Mat when there is none (Rest +15, never +30). A shift is Dim, Dark or Flood in turn (shift *s* of the day is *s* mod 3), and the Pod's +30 is Dark's. **Schedules**: each slimo's row gives the three shifts Work, Rest or Any — Work takes a slot, Rest sleeps, Any takes a slot while Rest is 40 or more and sleeps below it; the default row is Any / Any / Any, so a city works until it is tired and sleeps when it is, at any hour; Curfew sends it home in Dark. A slimo whose shift was a run worked it and takes no slot. A slimo eats when the Mat under its home carries nutrient ≥ 0.2 on signalled tissue (the meal takes 0.05 from the cell); when it does not, two eddies of paste from the ledger feed it; when the ledger is dry it goes hungry. A home is lit when it is within a Lume Farm's, a Lantern Post's or the Heart's light; dry when its cell is not water or a Lung stands within 6; signalled at σ ≥ 0.3. These are the `ShiftContext` the record's needs read.

**Districts and foremen** (research 14). A district forms where a Node has three built structures within 12 cords (Pass 2: the distance walked along Cord and Node from the hub to the nearest tile of the structure's footprint on the Mat); the Node is its hub, the structures within 12 cords that no nearer hub claims are its own, the members are the slimos whose bed is in it, else whose job is. The city tier caps districts at 1 / 3 / 8 / 20 (the Heart's own is the first). A district holds an election the shift it forms and every ten shifts after, and at once when the foreman dies or leaves: each member's weight is Voice + Mood / 20 (Orator × 2; a slimo on a run or stranded cannot stand), the highest wins, ties to the longer-serving. The foreman's job priorities stand over a member's where they are higher, the foreman posts the district's cargo demand (Cargo, below), and the foreman petitions. A district whose foreman dies holds a one-shift wake (no jobs) and its members' Mood target falls by the dead slimo's Voice.

**Petitions** are the foreman's mandates, and a district petitions only through a Charter Board (the Heart's district through the Heart's own board). When a district's members average under 30 in a need for two shifts running the foreman asks for the structure that answers it — Fed a Pit, Lit a Lume Farm (a Lantern Post will do), Dry a Lung Housing, Rest a Bunkhouse (a Pod will do), Signal a Relay Shrine — with five shifts to build it in the district. One petition stands at a time; met, the members' Mood target gains 10 for three shifts; missed, it loses 10 until the structure stands; refused, it closes and counts toward the district's dissolution (below). The petition is a line on the Colony HUD's alert queue, on the Charter and on the Slimo card of the foreman.

**The Charter** (Pass 2, carried in full; research 15). Ivan signs laws. A law is a signal pattern that leaves the Heart and takes effect in each district as the wave reaches it — the wave walks Cord and Node at 16 + 32 × Conductance tiles a shift, so a district within that many cords of the Heart takes a law the shift it is signed and a far one a shift or more later; a slimo in no district takes it when the wave reaches its bed (the Heart, if it has none). Laws cost Authority to sign and Mood to keep. A repeal is free, refunds nothing and travels the same wave; signing again costs again. There is no signing cooldown: Authority paces the Charter.

| Law | Authority | Effect | Cost | Live from |
| --- | --- | --- | --- | --- |
| Rationing | 1 | Slimo nutrient draw halved: a meal takes 0.025 from the Mat, paste costs 1 e | Mood target −5 while active | now |
| Open Grates | 1 | Markets trade with any faction at +1 reputation | Attention +50 % at every claimed grate | the factions and Attention (Phase D) |
| Conscription | 2 | Every D2 is militia (Guard at priority 4) during a Patrol or worse | Scav yield −20 % during alarms | Patrols at the colony (Phase D) |
| Quarantine Protocol | 2 | Relays auto-sever any region that loses signal for a shift | Loses outlying cords to false positives | Blooms and feral strains (Phase D) |
| Dissolution Rite | 2 | D5 feeding allowed without the per-event Mood hit | A standing Mood target −3; Ivan's barks change for the run | now (barks Phase E) |
| Tithe | 0 | The chapter-3 accord: 10 % of T1 to the Oathkeepers each shift | They map the Heart | offered only by the accord (chapter 3, Phase E) |
| Curfew | 1 | Slimos stay in Pods during Dark shift: the Dark is Rest whatever the row, and nobody asleep defects | Output down (no Dark shift); flood damage down (Phase D) | now |
| Foreman's Cut | 2 | Foremen take 5 % of district cargo as discretionary stock | Districts self-repair faster; your quotas fill slower | the cut now; the self-repair with damage (Phase D) |

The Authority column is this design's (Pass 2 gives none): a first law costs one, so the first district formed pays for the first law, inside Pass 2's 20–40 minute beat. A law that is not yet live can be signed; its Mood and Authority are real at once and its effect arrives with its system. **Authority** is earned by deeds — a district formed +1 now, repelled attacks (Phase D) and chapter beats (Phase E) later — and spent on laws and Parley tiers; it never comes from scrap. Ivan's four Authority states (his sprite, his portrait, his bark pools) follow the Authority earned over the run, never the balance — 0–2, 3–6, 7–12, 13 and over — so signing does not shrink him. **The vote**: when a law is signed, each district with a Charter Board votes through its foreman, against when the law costs its members Mood and for when it does not, and the Charter keeps every law with its shift and its vote (Pass 2's board at the Heart).

**Dissolution.** A D5 (Mod-Depth 5, cannot act) can be dissolved into a Pit from its Slimo card: 0.30 nutrient of Street grade (Pass 2; today's Uncommon) at the Heart's Pit and its name on the Memorial. Without the Rite it is the per-event Mood hit: every slimo's target −10 the shift it happens (the number is this design's); with the Rite, no hit and the standing −3.

**Unrest** (Pass 2), read off a district's Mood — the average of its members':
- below 30 for two shifts: **strike**, the members take no jobs until the average is back at 30;
- below 20: **defection** — each shift each awake member below 20 leaves for good with 1 in 4 to a feral strain if one borders the district, or to the Oathkeepers if a Patrol is in its zone; both arrive in Phase D, and until then the district shows it is ready to defect; nobody asleep in a Pod during Dark defects (research 03);
- below 10: **the foreman petitions** to repeal the law that costs the district the most Mood, or, with no law to blame, for the structure its worst need lacks; answered (the repeal or the structure), the Mood target gains 10 for three shifts; **refusing twice** dissolves the district — its hub is removed, its foreman stands down, its structures lose signal priority (the cargo serves them after every other consumer) and the Node cannot hub again for ten shifts.

A single slimo with Mood below 20 refuses jobs whatever the district does (the needs-below-20 override). Unrest is how the city tells the player a law or a quota is wrong, in the only language a self-governing system has.

**The city's clock** (research 16). The city ticks at 2 Hz in every mode: cargo moves a tile a tick, a slimo a tile every two. The Mat steps once every 12 ticks, 125 steps in a 12.5-minute shift — the shift Pass 1 tuned the CA's rates on (0.004 nutrient a tick is 0.5 a shift), now that a shift is a run of 8–15 minutes and not Pass 2's 60 seconds; at 2 Hz the Mat lived eleven of its shifts in one run. The Mat eats only what the city feeds it: its Pits (a feed, a quota's pull, a dissolved D5), the Deposits it covers (+0.004 a step each, Pass 1) and Bait — the tool bar's Bait smears the cheapest Common Chrome in the Stash onto the 3 × 3 around the click, its nutrient spread over the nine, so the Mat grows toward it on nothing the Stash did not give (this design's; Pass 1's Bait Smear painted 0.3 onto five tiles for one Junk unit, eight times its cost). The tool bar's other verbs take their costs with the Colony HUD (PAC-044). **The founding** is the 500-step growth of the scale proof's stratum — the Heart, seven Relays, a Pit on each — which is more Mat than one Runner can feed: the core around the Heart holds on about 4 nutrient a shift (twenty Common Chrome), and unfed the outer Mat recedes about 5 % a shift to what its Deposits carry. Pass 2's chapter 1 (60 cords at ten minutes, the Mat one haul a shift keeps) replaces the founding in Phase E. The scale proof's three thousand slimos and synthetic market hauls (Phase A) stay in `rose scale` as the load test of the arrays; the game's city has the roster's slimos and the Stash's cargo, nothing else.

**Cargo** (Pass 2's Automation pillar, carried in full; research 16). The Mat is the conveyor: Scrap, water, Downer and cold light move along cords as discrete packets, one tile per tick, routed by demand. The player never carries anything; they decide where demand is and watch the network answer it.

- **What rides.** The Stash is at the Heart, so a haul arrives at the Heart (the run hands its rig to the Stash) and every trip starts or ends there. At shift end the Stash's queue leaves as cargo — a Feed order to its Pit, a Sell order to its market — and so do the quotas' pulls; what a Still or a Salt Works makes rides home to the Stash. One item is one packet, and its effect lands with it: nutrient and drift into the Pit; eddies into the ledger at that market's glut at that moment; a Part into the Still's store; a product into the Stash grid (it waits at the Heart while the grid is full). What is on the cords is in the save and rides again from the Heart on load. Brine is the city's, not cargo: a Still uses the brine the city's Lungs made that shift (v3.9). The Bench and the Appraisal Table work the Stash where it is.
- **Consumers** publish demand, Pass 2's list: a Pit below its quota, a Still short of its Enzyme Parts, a market with an order riding to it. Pass 2's Vault takes cargo when Attention makes hiding Blackline matter (Phase D); its Pod cluster takes none, because water is a Lung's reach (a home within 6 of a Lung is dry, v3.9). Demand is the share of a quota not yet met, 0–1, counting what is already riding to it (research 16: a request that ignores what is in transit over-fills); an order's demand is 1.
- **Routing.** Each consumer has a signal-weighted distance field over the Mat (Tip, Cord and Node; a tile costs 1 + 2 × (1 − σ), so a packet takes the bright trunk over the dim shortcut — the weights are this design's), rebuilt every 20 ticks from the Mat's current shape. A quota's pull goes item by item to the consumer with the highest demand per tile of distance from the Heart, ties toward the Heart; an order goes where it was sent. When a field rebuilds and a packet's consumer cannot be reached, it re-chooses: the reachable consumer of its kind (Pit, market, Still) with the highest demand per tile of distance from it, ties toward the Heart; with none it waits on its tile, and cargo for the Stash waits until the Heart can be reached. A packet on a tile that stops being Mat steps onto Mat beside it, or is spilled and lost (a line on the shift report). So a Severing reroutes traffic within a shift, and a necrosed trunk shows as a jam before anything starves.
- **Capacity and jams.** A tile carries 4 packets and nothing enters a full one. A full tile with cargo waiting to enter it (Pass 2's trunk above capacity) is a jam: its packets move at half speed, so the jam backs up the trunk until it drains. Traffic per tile is slimos + 2 × packets; the Stack view draws it as light (Mat Lit, Mat Glow from 3) and a jam white (Chalk) — Pass 2's prompt to grow a parallel cord or raise Conductance. Patrols pathing toward a white-hot trunk arrive with the Patrols (Phase D).
- **Quotas** (Pass 2: "keep 40 units, Junk first, never Blackline"). Each Pit and Still carries one, set on its card in the Colony (click it). A Pit's is *keep N nutrient*: at shift end it pulls until its stock plus what is riding to it reaches N, N at most its stock (Heart 40 / 80 / 160 / 320, Pit 20 / 40 / 60); feedable items only (Chrome and Relics), never pinned, the lowest tier first and the cheapest within it, none above its tier ceiling (Uncommon unless raised). A Still's is *keep N Enzyme-tag Parts* in its store, N defaulting to its Cook slots (a Still is built to use them). A Pit's own default is off, except where a foreman posts the district's demand: a Pit in a district with a foreman and no quota of the player's carries the foreman's — keep what the members' meals took from the Mat last shift (0.05 a meal, half under Rationing), shared among the district's Pits, so a district asks the Stash to put back what its people ate. Hauls arrive at the Heart, not at a market, so Pass 2's "priority order set on the market" is the routing's own rule here (demand per tile of distance, ties toward the Heart); the one priority above it is the Charter's: a dissolved district's structures are served after every other consumer. What no quota takes is the surplus the Sell queue and the bulk bar draw on.
- **Foreman's Cut.** One packet in twenty landing at a consumer in a district the Cut has reached is the foreman's discretionary stock: it leaves the economy and its count shows on the district (your quotas fill slower); the stock pays the district's self-repair when structures take damage (Phase D).
- **Movers.** The slimos on the map are the roster: each is at its bed while it rests, walks to its structure and works there while its job has it (the shift's plan, from the jobs the colony will pay at shift end), and is off the map on a run or stranded. They walk Mat and walkable floor by flow field.

## Economy

Two currencies and one tax. Eddies buy from the surface; Authority buys laws and parley tiers and is earned only by deeds; Attention is the tax every eddy and every lume pays at the grate.

**Vendors.** Five factions, each with stock tiers 1–4 unlocked by reputation (−3 to +3) and by sales volume, stock that rotates every day with a few fixed items, and a buy list with multipliers. Gangs: weapons, street chrome, protection, buy Downer at 1.3. Nomads: food, salt, hauling contracts, buy cold light and water. Corpo remnants: Blackline fencing at tier 3, maps, intel, buy data shards at 1.5 and found-in-run items at 1.25. Badges: patrol timings, buy Downer at 1.2, sell lamp-carrier badges at tier 4. Oathkeepers: chapter 3 only.

**Contracts.** Each vendor offers three at a time: deliver (n of category at condition ≥ c), clear (a Bloom or a raider camp on a route), recover (a named corpse or item), host (shelter a named newcomer for 5 shifts). Rewards: eddies, reputation, a stock unlock, a unique item. Chains of three to five with rising asks; a miss breaks the chain and costs one reputation.

**The Downer chain** is the mid-game engine: Lung brine + ooze enzyme (a Still with a slimo on Cook) + a Part with an Enzyme tag → Downer dose. Sells to every faction, highest to Corpo; every Still adds 8 Attention per shift to its zone; the Oathkeepers target Stills first in a Cleansing. The decision the chain forces: how many Stills before the gauge beats the income.

**Pricing** follows the Items formula; the Market screen shows each vendor's demand multiplier per category as a bar and the glut as a decaying mark, so a player can see the price they are about to get and the one they would get next shift.

**Attention** per grate zone: Lume within 10 tiles, trade value, Blackline and Relics outside Vaults, Stills, kills of scouts; decays 2 per shift; Corpo intel reads it exactly. Thresholds 30 / 60 / 90 for Patrol / Sweep / Cleansing; each stage is also salvage.

**Sinks** so eddies stay scarce: food paste (2 per slimo per shift when the Mat cannot feed them), tolls at neutral grates, hires, protection, vendor repairs, insurance-like Spored tags (craft cost), research scrap, Stash expansions. Early game the player is eddy-poor and chrome-rich; mid-game eddy-rich and Attention-poor; late game the question is what to do with wealth that cannot be shown.

## Progression

Nine tracks run in parallel over a 20-hour campaign; no track blocks another for more than a session, so a player stuck on one is moving on three.

| Track | Unit | Pacing target | What it changes |
| --- | --- | --- | --- |
| Runner levels | Skill XP by doing, 0–10 per skill | A Runner reaches 5 in their main skill by hour 4, 8 by hour 12 | Mid-tier jobs, mastery perks, visible gear tier |
| Strata | Six, by Relay reach or key | S2 hour 1, S3 hour 3, S4 hour 6, S5 hour 10, S6 hour 14 | Routes, items, factions, hazards, white light |
| Tech | 36 nodes, 3 branches × 4 tiers, blueprint-gated | Tier 2 by hour 3, tier 3 by hour 8, tier 4 by hour 15 | Structures, recipes, tools, the handoff |
| Strain | 10 genes; classes at thresholds; 12 organ types | First organ hour 2; first class hour 7; second class hour 15 | The Mat's look and behaviour, organ housings, endings |
| City tiers | 20 / 100 / 500 / 2,000 slimos | Hamlet hour 1, Quarter hour 5, City hour 12, Empire hour 18 | District cap, market count, title card, Heart tier |
| Ivan's Authority | Deeds: chapter beats, repelled attacks, districts formed, contracts chained | One law per 2–3 hours | Laws, Parley tiers, ooze-time duration |
| Faction reputation | −3 to +3 × 5 | One vendor at +2 by hour 6; alliances in chapter 6 | Stock tiers, contracts, chapter-7 militia |
| Stash and Heart | Four tiers | Heart tier 2 hour 4, 3 hour 10, 4 hour 16 | Grid size, Vaults, appraisal speed, the chamber's look |
| Collections | Scrapdex (150 items), Strainbook (12 organs, 5 classes, 20 notable mutations), Curio sets (7), Memorial, Trophies | 40% Scrapdex by campaign end on a normal run | Set passives, codex lore, cosmetics |

**Difficulty curve.** Threat scales with stratum and Attention, never with time alone, so a careful player can stay small and safe in S2 for as long as they like and feel the ceiling of it: S2 loot tops out at Uncommon, and the Oathkeepers only arrive when invited by wealth.

**Meta.** A finished run exports a strain code (genome + class + lineage) and a seed. New Game+ starts with the strain as the Heart's genome, the Charter's laws unlocked, the Scrapdex kept and the Stash empty. Endless Sump on a shared seed ranks by Empire-tier shift count. Daily seed: one route, one Runner, fixed rig, leaderboard by extracted value.

**Endings** read off the dominant genes and the alliance score at the Great Cleansing: Garden, Hive, World-Eater, Scattered, as in Pass 2, plus **Accord** (added here): a Thrift and Toxicity strain that the Oathkeepers cannot cleanse and cannot approach signs the only treaty in the game's history; Night City gets its utility, Slimecity gets its border, Ivan narrates from a sealed city.

## Visual progression

Every upgradable thing has visible states, listed here so the asset manifest carries them from day one and no system ships with one look.

| Thing | States | What changes on screen |
| --- | --- | --- |
| Heart | 4 tiers | Sprout: a bulb in a culvert. Cathedral: a chamber of ooze arches with captured lamps hung white, the memorial wall, the Charter board |
| Structures (24) | 3 tiers each | Tier 1 scrap and tarp; tier 2 welded plate and ooze housing; tier 3 chrome-plated with lume trim in the district colour |
| Mat by class | Base + 5 classes | Garden: bulbs along cords. Hive: chrome plating and conduit lines. Runner: long thin filaments. Bramble: thorned junctions. World-Eater: maws at nodes |
| Mat by health | 4 | Fed (ooze glow pulse), lean (dim, slow pulse), starving (grey-violet, no pulse), necrotic (bruise to silt) |
| Slimo body | 5 depths × vein intensity by Lume | Veins glow brighter in Lume strains; D4 bodies are translucent ooze with chrome inclusions |
| Slimo gear | Head, body, feet, hands layered | Every equipped item is a sprite layer; implants show as plates or lenses |
| Runner rank | 3 marks | A sash glyph at skill 5 and 8; a Chalk stripe for ten runs survived |
| Districts | 4 by size | Tag paint on claimed tiles in the district colour, then string lights, then signage, then a lit hub marker on the Stack view |
| Grates | 2 | Unclaimed: dark bars. Claimed: a painted sign on the street above, leaking the district colour down |
| Ivan | 4 by Authority | Plain ooze; a robe; chrome fragments at the shoulders; the saturated Ooze Glow end of the ramp with Hot Pink highlights |
| Stash | 4 | A crate; racks; a vault door; a lit treasury |
| Vendors | 4 stock tiers | The market stall gains a canopy, a sign, a guard, a generator |
| Items | Condition 3 bands | Scuff and rust overlays at < 60 and < 20 |
| Night City above (S6) | By chapter | Neon leaks become sparse and white as the occupation advances |

Rule: nothing in this table is a palette swap alone; each state adds or removes shapes. A palette swap is how states are prototyped in placeholders, never how they ship.

## Art bible

One style, named **Sewer Noir 32**, applied to every pixel on screen including placeholders. The rules below are the whole style; anything not covered is decided by them.

**Grid and scales.** World base 640 × 360, integer-scaled (3× at 1080p, 6× at 4K). Tiles 32 × 32. Slimos 32 × 48; Ivan and lamp carriers 48 × 64; sewerats 32 × 32; items 32 × 32 icons in a 32 px inventory grid; UI on a 4 px rhythm with 8 px margins; chapter panels and codex art at 1280 × 720 (exactly 2× the world, so panel pixels are world pixels doubled).

**Line.** 1 px selective outline in Void on every sprite, item icon and UI frame; outline breaks where a form turns into shadow (selective, never a full silhouette trace). Tiles carry no outline; their edges are read from value contrast. This is the one rule that makes the world photographed and the people drawn.

**Value and colour.** Three-step ramps, hue-shifted: shadows pull toward Drain Blue, lights pull toward the nearest emitter's hue. Two-tone ordered dither at 50% and 25% for gradients; three-tone gradients never. Clusters of pixels follow form (no single-pixel noise, no banding bands). Saturated colours are emitters only.

**SEEPAGE v2** (the ooze ramp replaces the green Mat ramp; green survives as sewer water and mould).

| Ramp | Hex | Role |
| --- | --- | --- |
| Noir | 07070C Void · 0E1018 Sump · 171A26 Culvert · 232737 Concrete Night · 343A4F Wet Concrete | Ambient, shadow, concrete |
| Industrial | 4A4F5E Rust Gray · 6B6F7B Rebar · 8E9097 Dust · B5B4B0 Chalk | Steel, chrome, bone |
| Mud | 2A1F19 Silt · 4A3526 Sewer Brown · 6E4E34 Rust · 9A7049 Oxide | Brick, muck, leather |
| Bile | 2E3A1E Bile · 4E5E27 Vomit Green · 7A8A3A Pond Scum | Standing water, mould |
| Drain | 122238 Drain Blue · 1F3D63 Cistern · 35679C Flood Light | Moving water, shadow hue |
| Flesh | 5B3A4A Bruise · 9A6070 Flesh Dim · D99AA6 Flesh | Slimo skin at D1–D3 |
| Ooze (emits) | 2A1140 Ooze Dark · 5A1E8C Ooze · A44BE0 Ooze Lit · F0A6FF Ooze Glow | The Mat, D4 bodies, veins, Ivan, the signal pulse |
| Neon (emits) | FF2E97 Hot Pink · 19E6FF Cyan · F6FF2E Acid Yellow · FF6A1A Muzzle Orange | Night City leaks, UI, Ivan's highlights, fire |
| White (emits) | F2EFE4 Vestment White · D4A52A Reliquary Gold | Oathkeepers only |

**Per-module plans**

- *Tiles*: 47-tile blob autotile per material (concrete, brick, pipe, water, grate, collapse), wet highlight 1 px in the nearest emitter's hue on every horizontal, parallax planes behind (cavern wall, distant tunnel) and in front (pipes, cables).
- *Mat*: cords 8–12 px wide at 32 px tiles with an Ooze Dark core and an Ooze Lit edge, pulse in Ooze Glow outward from the Heart; translucency faked with a dither of the tile beneath; organs and class decorations as node overlays.
- *Slimos*: five depth bases × skin × inclusion overlay × vein overlay × gear layers; D4 bodies use the Ooze ramp with Chalk inclusions. Walk 8 frames, idle 4, work 6, carry 6, down 1, dig 6, swing 4, aim 2.
- *Ivan*: the only 48 × 64 slimo; Ooze ramp at full saturation, Hot Pink rim light, four Authority states.
- *Oathkeepers*: 48 × 64, Vestment White with Reliquary Gold trim, bulky Y2K hardware, lamps on poles; their light is the only pure white and it casts hard shadows from cords.
- *Items*: 32 × 32 icons on a shared 24 × 24 inner silhouette grid, tier shown by a 1 px corner mark in the tier colour, condition by rust overlay bands.
- *UI*: 9-slice frames whose background ends exactly at the frame line; two fonts (a 5 × 7 pixel monospace for numbers and terminal text, a 7 px-cap condensed grotesk for labels and narration); labels slam in with a two-frame glitch; Cyan numbers, Acid Yellow alerts, Hot Pink confirmations, Vestment White for anything Oathkeeper.
- *VFX*: sparks, ooze drips, bleach foam, flame, bullet trails, dig dust, extraction channel ring; all 2-tone, all on the emitter rule.
- *Panels*: 1280 × 720 pixel stills, hard ink, high contrast, text boxes in the grotesk.

**PixelLab pipeline.** Three style anchors are made first (one brick culvert tile panel, one D2 slimo, one Rare chrome item), curated by hand, and every later asset is generated with `generate_image_bitforge` against them at high style strength; `rotate` for the side-on flip, `animate_with_skeleton` for the eight-frame cycles, `inpaint` for gear layers over a base body. Generation sizes 32 / 64 / 128 only. Every import passes the palette-lock validator (quantise to SEEPAGE v2, reject if more than 2% of pixels moved more than one ramp step), the size check and the outline check (1 px Void on the alpha edge). Tiles are generated as 64 × 64 panels and cut.

**Placeholders.** One language, generated by `tools/placeholder` from the manifest: exact size, a flat fill in the slot's ramp (tiles Noir, Mat Ooze, slimos Flesh, items Industrial, UI Sump), a 1 px dashed Void border, the slot's three-letter tag in the pixel monospace, and a silhouette hint (capsule for a body, rounded rect for an item, hatch for a tile). A placeholder obeys the grid and the outline rule, so a screen of placeholders already reads as one style; it is obviously unfinished because of the dashes and the tag, never because it breaks the grid.

## Narrative and Ivan

Ivan Ooze is the voice of the game and the one character the player cannot lose. He is purple ooze with chrome fragments in it, bald, ridged, theatrical, and he talks to the player the way the lore's framing talks: profane, sardonic, fond of his own jokes, with contempt for the Oathkeepers that never becomes a speech. He is a D4 who cannot leave the Mat; the Runners go where he cannot, and he narrates their runs from the Heart over the comm (barks during runs are his, pitched to the Runner's situation).

**On the map** he is a hero unit with Rally, Parley, Feed the Heart and ooze-time (Pass 2). His Authority states change his sprite and his bark pools.

**Chapters** follow Pass 2 (Seepage, Tweakers, The Hand, Sewerats, Warchest, Common Ground, Rise); each opens and closes with 3–5 panels at 1280 × 720 and sets one new system in motion. The campaign is 20 hours at the pacing in Progression.

**Bark pools**: 14 event classes (first haul, Rare identified, Blackline identified, Runner down, Runner dead, newcomer wakes, D5 fed, Patrol sighted, Sweep, Cleansing begins, Bloom, Flood, law signed, stratum breached) × 4 Authority states × the strain's dominant class = pools of 8–12 lines each, about 500 lines at launch. Lines are two sentences at most.

**Written samples** in his register:

- First Blackline identified: *That's Moon-make. Hold it close and they'll find you by the smell of their own money.*
- A Runner dies on a run: *He's in the east drain. I'll tell his bunk. You tell me what he was carrying.*
- Dissolution: *Dropping one of ours in the Pit. Write the name on the wall first.*
- Stratum breach to S6: *You hear that? That's traffic. We're under their feet.*

**Endings** are five (Pass 2's four plus Accord) and each has its own panel set and a final Stack-view shot of the city the player built, held for as long as they want to look at it.

## Game shell

The parts every finished game has, specified so they are built with the systems and never bolted on.

| Part | Spec |
| --- | --- |
| Title and menu | Stack view of the last save's city behind the menu, live; Continue, New Run, Endless Sump, Daily Seed, Codex, Settings, Quit |
| New Run | Seed (random or typed), strain code (blank or imported), difficulty, Ivan's opening panels |
| Saves | One save per run, autosave every shift and on run entry and exit; three manual slots; save = seed + input log + snapshot every 10 shifts, so a save is small and a replay is possible |
| Difficulty | Sump (Attention decays 3, Runner corpses never looted by raiders), Culvert (the design values), Undercroft (Attention decays 1, one Spored tag per run, Flood every day) |
| Onboarding | The first ten minutes are the tutorial (Pass 2); Ivan speaks, the UI highlights, no modal text; a Codex entry unlocks each time a system first fires |
| Pause | Pause stops the shared clock in colony mode; in a run it opens the rig and the map, the world keeps moving at 0.2× for 3 s then stops, so looting under pressure stays pressure |
| Settings | Video (scale factor, fullscreen, vsync, CRT and scanline off by default, screen shake), Audio (five buses), Controls (rebind, gamepad, mouse sensitivity), Gameplay (tick speed defaults, auto-pause on alerts, confirm sells over 200 €$), Accessibility |
| Accessibility | Colour-blind palette variants that keep the emitter rule (the ooze ramp has a Cyan-leaning variant), text scale 1–2×, reduced flash, screen reader strings on every label, hold-to-toggle for channels, no timed input in menus |
| Controls | Mouse and keyboard primary; gamepad full parity with radial tool menus; run mode uses a twin-stick layout on pad |
| Codex | Scrapdex, Strainbook, Charter history, Memorial, Trophies, Lore (the framing and definitions, unlocked by play), Keybinds |
| Seeds and codes | Seed code and strain code visible on the pause screen and copyable |
| Achievements | 40, tied to the collections and the endings, no grind-only entries |
| Localisation | All strings in tables from day one; fonts cover Latin extended; right-to-left out of scope at launch |
| Telemetry (local) | A per-run cadence log of landings, losses and decisions with clock times and a report (median gap between landings and between pulls, notables, damage, value per minute), written to `user://telemetry/`; `rose cadence` aggregates seeded bot runs against the budget (Expedition mode: The cadence budget) | Read the file; the Run's end summary quotes the two medians |

## UI and screens

Twelve screens, one grid, one frame rule. Every panel is a 9-slice whose background ends at the frame line; every label is kinetic (slams in, resolves from a two-frame glitch); numbers in Cyan monospace, alerts in Acid Yellow, Oathkeeper anything in Vestment White. Diegetic labels pin to world objects at Street and District zoom; the Stack view carries only district names and the Attention gauges at grates.

| Screen | Layout | Key interactions |
| --- | --- | --- |
| Colony HUD | Top-left: clock, shift, speed, view. Right edge: alert queue. Bottom: tool bar (Pit, Bait, Salt, Sever, Relay, Build, Zone, Overlay). Bottom-right: minimap of the Stack view | Hover any slimo, structure or cord for its card; click to select; right-click to order |
| Build menu | Categories as tabs; ghost footprint snaps to Cord/Node; cost shown in items the Stash holds, red where short | Rotate, place, queue; upgrade in place shows the next tier's sprite as the ghost |
| Slimo card | The roster down the left (name, depth, status, Mood), the selected record on the right: portrait by depth, stats as gauges, skills as ten-step bars with XP to next, traits with their effects in words, needs as six bars with the Mood target marked, injuries, the record line, bonds | Pick a slimo; set job priorities 0–4 in a row of cells; send on a run (opens the Route Board with them as the Runner) |
| Job grid | Rows slimos, columns jobs, 1–4 priority cells; district filter | Drag to paint priorities |
| Stash | Filter and sort bar on top; the grid left (32 px tiles at real size, scrolls); right column: item card with the three numbers and the four verbs, the bulk bar, the queue panel with the ledger | Click to select, the verbs, pull a queued item back, bulk Sell / Feed on the filtered view, pin |
| Bench | Recipe list by tier left (tier 1 open, 2–4 locked with the blueprint they need); the chosen recipe's slots by craft tag centre with a picker of Stash items that carry the tag; outcome preview right: product, every stat against the base, condition, provenance; the queue below with shifts left | Pick a recipe, click a slot then an item, CRAFT queues, CANCEL returns the parts |
| Market | Vendor tabs; stock grid; sell queue; demand bars per category; glut marks; contracts panel | Queue orders; accept a contract; Parley if Ivan is present |
| Strain Lab | Ten gene dials (read-only) with drift arrows from the current feed plan (every Pit's quota and its tier ceiling); lineage tree; organ thresholds; Strainbook | Read the feed plan; each Pit's quota is set on its card in the Colony |
| Charter | The board; laws as cards with cost and effect; districts list with foremen and Mood; petitions | Sign, repeal, answer a petition |
| Route Board | Three route cards (chunks and shift minutes, danger band, remembered share, known containers, water share, corpses, stranded) over the memory strip of the selected route; crew left, rig right; packs from the Stash | Select a card (click or 1–3), toggle followers, click items into the hand, quick and pack slots, DEPART |
| Run HUD | Minimal: exposure and noise arcs, the notice ring and the channel ring on the Runner; shift clock and route strip top; body silhouette, stamina and status tags bottom-left; hand and quick slots bottom-right; follower chips right; one prompt line bottom centre; the map on hold (Phase F) | Loot panel on container open (grid to grid, camera eases east), TAKE ALL, the used quick slot flashes |
| Codex | Scrapdex grid with silhouettes for the unseen; Strainbook; Memorial; Trophies; Lore | Browse |

Text rules: labels under 24 characters; numbers never animated past 0.3 s; one alert per second at most; no modal that cannot be dismissed with the same key that opened it.

## Content scope at launch

| Content | Count |
| --- | --- |
| Strata | 6 |
| Authored run chunks | 42 (7 per stratum), 32 × 24 tiles each |
| Items | 164 across 9 categories |
| Recipes | 60 by craft tag |
| Structures | 24, three tiers each |
| Organs | 12 |
| Strain classes | 5, plus the 3-class Relic state |
| Tech nodes | 36 |
| Laws | 8 |
| Traits | 40 |
| Threat types | 3 sewerats, 3 Oathkeeper roles, 2 raider groups, 4 hazards |
| Factions | 5, four stock tiers each |
| Contracts | 60 templates, chained |
| Chapters | 7, with 56 panels |
| Endings | 5 |
| Barks | \~500 |
| Codex entries | \~260 |
| Achievements | 40 |
| Sprite frames | \~6,000 (tiles 1,200; Mat 600; slimos 2,400 with gear layers; Ivan 120; Oathkeepers 240; sewerats 108; structures 430; items 164 × 3 bands; UI 200; VFX 300) |
| Campaign length | 20 hours; Endless and Daily unbounded |

## Technical architecture and repository

Rust owns every rule and every number; Godot owns the clock, the input, the camera and the picture; the asset system sits between them so art can land without touching either.

```markdown
rose/
  core/                      Rust workspace (no engine dependency; headless-testable)
    crates/mat               CA + nutrient field, organs, strains, signal       (M1, re-gridded to 32)
    crates/logistics         the city's clock (Mat step every 12 ticks), consumer fields (signal-weighted), cargo packets with capacity and re-routing, jams, the roster's movers; the scale proof
    crates/items             item table, instances, condition, identification, pricing, sets, crafted stats and provenance
    crates/bench             recipes by craft tag (data/recipes.toml), slotting, outcome from parts, the Bench queue
    crates/people            slimo records: creation, stats, skills and XP, traits (data/traits.toml), needs and Mood, job utility, bonds, the roster
    crates/inventory         grids, containers, rigs, Stash tiers
    crates/colony            the 24 structures (data/structures.toml), placement and building, zones and filters, homes, meals, light, districts, foremen, petitions; the Charter (data/laws.toml), its wave, Authority and unrest; events and Attention follow
    crates/run               chunk · assemble · map · fov · light · noise · fog · senses · loot · creature · runner · follower · board · sim (the 20 Hz run) · telemetry · bot
    crates/economy           vendors, stock tiers, contracts, chains, glut, Downer chain
    crates/progress          tech tree, strain classes, Authority, city tiers, collections, endings
    crates/save              seed + input log + snapshots, replay, strain codes
    crates/render            headless PNG contact sheets for every module (the visual regression source)
    crates/cli               `rose` headless runner: scenario, bench, sheet, validate
  bridge/                    GDExtension `rose-godot`: typed arrays, dictionaries, events; no logic
  game/                      Godot 4.5 project
    modes/                   colony/ run/ stash/ bench/ market/ strain/ charter/ route/ codex/ menu/
    ui/                      frames, fonts, kinetic labels, alert queue, grid widget
    shaders/                 palette lock, emitter light, traffic heat, parallax, water, glitch, ooze-time
    assets/                  AssetSlot resources only; every drawable is a slot id, never a path
  assets/
    palette/SEEPAGE_v2.gpl
    manifest/*.yaml          one file per slot family: id, kind, size, frames, ramps, state tiers, anchor, prompt, status
    anchors/                 the three style anchors, curated
    placeholders/            generated; never edited by hand
    art/                     PixelLab output after validation; same ids as placeholders
  tools/                     placeholder generator · palette-lock validator · manifest -> prompt sheets · importer/cutter · contact-sheet differ
  data/                      items, chunks, loot, creatures, recipes, structures, laws, traits, barks, vendors, tech, as TOML; the file beside the project wins in debug, the compiled copy ships
  docs/                      design bible (this doc, exported) · art bible · module specs · todo.PACER.csv
  tests/                     Rust scenario tests · Godot headless smoke · contact-sheet regression per commit
```

**Asset slots.** A drawable in Godot references `AssetSlot("slimo.d2.walk")`. The loader resolves `assets/art/<id>.png` if present and validated, else `assets/placeholders/<id>.png`. The manifest entry carries size, frame count, state tiers, the anchor to match and the PixelLab prompt, so the prompt sheet for the artist is generated from the same file the engine reads. A slot cannot be referenced that is not in the manifest (build fails), and a manifest entry cannot lack a placeholder (tool regenerates them).

**Data over code.** Items, chunks, loot tables, creatures, recipes, structures, laws, traits, barks, vendors and tech are data files with schema validation; balancing never recompiles. Chunks are authored as glyph grids (`tools/gen/gen_chunks.py` writes `data/chunks.toml`); a Godot tile-editor export to the same format lands with Phase F tooling.

**Determinism.** One PCG32 stream per sim; UI and VFX draw from a second; a save replays from seed and input log. The run is `run::sim::RunSim`: every rule of a run — movement, senses, loot, creatures, followers, combat, digging, extraction, the outcome — advances in `step(input)` at 20 Hz on integer and fixed-point state, the input word is logged (`Input::RunInput`), and Godot's Run screen is a view that gathers input and draws state and events. Save format v3 carries `route_memory` per (seed, stratum) and `corpses` (route, tile, day, the rig); migrations begin with the first shipped save format. `run::board::RouteCard` computes a card from the compiled route, the creature file and the route's memory bits, so the Board never counts anything itself; `RunSim` carries the shift clock (`shift_ticks`, overtime spawns from `[shift]` in `data/creatures.toml`). The Stash is one `RoseStash` held by the `Sim` autoload for the whole session; screens are views of it. `inventory::ledger` holds the sell and feed queues, the eddies and per-market glut, and resolves a shift into a report (eddies earned, deliveries of nutrient and genome drift per Pit) that the colony applies; save v4 carries the ledger. The `bench` crate loads `data/recipes.toml`, validates a slotting, computes the outcome (an `ItemInstance` with `crafted` provenance and `mods` over its def) and runs the Bench queue to shift end; save v5 carries the queue and the instance fields. The `people` crate owns the slimo record (creation from the seed, the forty traits from `data/traits.toml`, names from `data/names.toml`, needs and Mood per shift, XP from the run outcome and the shift report, job utility, bonds) in a `Roster`; save v6 carries it, the Route Board reads its eligible slimos and the run takes a `RunnerSetup` (body, nerve, hands, eye, veins, rig rows, the run flags) from the record.

**Tests.** Every crate has scenario tests (seeded, asserting regimes); the `render` crate emits a contact sheet per module that the differ compares per commit; the Godot smoke test exercises every mode shell headless. A pull request that changes a contact sheet carries the diff image.

## Build plan

One thread, six phases, each ending with the whole repo green and a contact sheet that shows what changed. No phase ships a system without its placeholders, its data file, its tests and its row in the manifest.

| Phase | Builds | Proves |
| --- | --- | --- |
| A Foundations | Repo restructure; SEEPAGE v2 and the art bible as `docs/art-bible.md`; manifest schema and the first 60 slots; placeholder generator and validator; item and inventory crates with the 164-item table as data; save/replay; the 12 mode shells in Godot drawn entirely from placeholders at 32 px | A contact sheet of every placeholder in one style; a headless pass through all 12 screens; items round-trip through save |
| B The run | `run` crate: chunk assembly from the authored chunks (19 shipped), fog, light and noise, loot tables, three sewerat types, scouts, combat, extraction, corpses, the 20 Hz run sim; Route Board and Run HUD; Stash and the four verbs; Bench tier 1 | Seeded runs with assertions on loot value distribution and death rate; a recorded run replays bit-exact |
| C The city | `people`, `colony`, `logistics` to spec: jobs, needs, schedules, zones, districts, foremen, Charter, unrest, quotas, cargo jams, 24 structures with three tiers of placeholders; Colony HUD, Build menu, Job grid, Charter | 20-shift headless city with assertions; the reward-cadence log shows an event every 10–30 s |
| D Pressure | Attention, Patrol / Sweep / Cleansing, Blooms, Floods, raiders, scrap turrets, militia; economy: vendors, stock tiers, contracts, Downer chain; Market screen | A seeded Cleansing with and without gates shows the designed survival spread; price drift stays within bands over 30 shifts |
| E Progression and content | Tech tree, strain classes and organs, Authority and laws, city tiers, collections, chapters 1–7 with panel slots, barks, endings, Endless and Daily, difficulty tiers | A scripted 20-hour campaign runs headless at 100× and reaches each ending by its genome |
| F Art and feel | PixelLab generation against the manifest, validation, animation, visual tiers, parallax, water, lighting, audio, accessibility palettes | Contact-sheet regression against anchors; the acceptance screenshots in the Pass 2 Visual section |

Research is written into `docs/research/` before each stage's design freeze: extraction-genre loot economies and loss design (01), survivors-genre cadence (02), colony-builder job and need systems (03), pixel-art craft (04), PixelLab constraints (05), Ivan Ooze's canon (06), light, noise and fog as stealth systems (07), loot tables and streak control (08), threat awareness and per-section bodies (09), followers and what loss keeps (10), the embark screen and the run HUD (11), selling, feeding and bulk management (12), recipes by component and stats from parts (13). Each research note ends with the decisions it changed, and those decisions are written into this document in the same pass — this document is the plan; a stage that leaves its numbers only in a research note or a verification log has not finished.
