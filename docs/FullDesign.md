# Cyberpunk: ROSE — Full Design

Nov 7, 2026 · @heat · v3.3 Nov 8: Expedition mode written to the numbers Phase B fixed (PAC-020–025, research 07–11): the Route Board, the shift clock, the Run HUD

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

**Traversal.** Slimos are ooze: they cling to and crawl along any surface, so a Runner moves in four directions through every passable tile of the cross-section (open floor and air, water, grates) and shafts need no ladders. Gravity is a rule for dropped items and bodies, not for slimos. Movement is continuous at a speed in tiles per second (sneak 2.5, walk 4.5, sprint 7.0; overweight halves sprint; legs at 0 halve everything and forbid sprint). Collapse and deposit tiles are solid until dug.

**The Route Board** is the one screen between the Heart and a run, and it embarks in one click: three route cards across the top, the memory strip of the selected route under them, the crew on the left, the rig on the right, DEPART. A route is a stratum and a region (Sump East in S1, Culvert North in S2, Storm Drain in S3 at launch; a Relay reaching a stratum opens its card — Phase E wires the Relay, until then the three launch cards are open) and its seed is fixed by the save and the stratum, so re-running a route learns it. A card states what can be known before a run and nothing that cannot: length as chunks and shift minutes; danger as an index of the authored spawns — each burrow brings the mean pack of 2 drawn from the stratum's mix, each creature weighted crawler 1, burster 1.5, maw 3; each chalk-marked chunk brings two scouts (weight 3) at the stratum's patrol chance — shown as the number and banded by the index per chunk, LOW < 1.5, MEDIUM < 2.5, HIGH ≥ 2.5 (measured over 60 seeds per launch stratum: S1 ≈ 1.2 per chunk, S3 ≈ 2.1, S2 ≈ 2.4 because the culvert chunks carry the most chalk); the share of the route remembered from earlier runs; the lockers and crates seen on them (never the total); the water share of its tiles and the grates on it (the Flood timing joins this line in Phase D); our corpses by Runner and day; followers stranded there. The memory strip draws the selected route's remembered tiles at one pixel per tile (448 × 24 at most), the Mat edge in Ooze Glow, grates in Acid, known containers in Cyan, corpses in Hot Pink; unseen tiles are Void. The crew panel lists the Runner and the colony slimos who can follow (Phase C brings the records; the three named Phase B slimos stand in), two follower toggles. The rig panel packs from the Stash itself — the hand weapon, four quick slots, backpack items as a row of icons to click in, the weight against carry in Cyan and Hot Pink when over — and what is packed leaves the Stash at DEPART and comes home with the rig. Entry is at the Mat's farthest cord in that direction or at a grate you hold.

**Map.** Assembled per run from authored 32 × 24-tile chunks, 8–14 per run, seeded with the save so a route can be re-run and learned. Chunks are glyph grids in `data/chunks.toml` (19 shipped across S1–S3; 7 per stratum at launch) with exits in canonical windows — rows 11–13 on the west and east edges, columns 14–17 on the north and south — so any two facing exits join; unjoined exits compile to brick. A chunk's weight is both its pick weight and its cap per route (a vault once, a culvert up to three times); nothing is ever placed west of the entry column, which is where the Mat is. Validation rejects partial exit windows, open border cells, entries without an east exit, and any chunk whose exits do not all reach one another without digging; a route must reach ≥ 95 % of its passable tiles from the Mat edge without digging and 100 % with every collapse dug. Water level follows the city's Flood clock.

**Light** is a 0–15 integer field per tile, dark by default. Static sources: broken fixtures emit 6, street grates 9 (Night City leaking down), Mat tiles 2 + Lume gene × 6. Dynamic sources: the Runner's lume, scouts' lamps (10, white), muzzle flashes (10 for a tick). Light spreads by an 8-connected integer Dijkstra (orthogonal 2, diagonal 3, water ×3); solids take the arriving level and stop it so lit wall faces read. Three bands: dark ≤ 2, dim 3–6, lit ≥ 7.

**Lume and sight.** A Runner's lume = veins (Mod-Depth 1–5) + gear light (lantern 6, lamp pole 8, visor −1) + the Mat under their feet, −3 while dimming veins (held, 6 stamina/s), halved while Soaked, never below 1 for any slimo. A tile is visible when it is in the Runner's symmetric field of view and either lit ≥ dim or within the see-radius 2 + lume + Eye/2: dark rooms are seen as far as your own glow, a lit pool from across the chunk. Symmetry means anything that can see the Runner can be seen by a Runner looking that way.

**Exposure**, the light gem: max(light at your tile, lume), one number on the HUD. It sets the **notice range**, the distance at which anything can notice you: 2 + exposure tiles (dark ≈ 3, dim ≈ 7, lit ≈ 14), standing still −1, sprinting +2, digging +1.

**Noise** is a 0–15 footprint stamped over the tiles a sound reaches — open 2 per tile (diagonal 3), pipes 1 (a run carries footsteps far), brick 3 (through a one-tile wall), water 4 (halved reach), rubble and deposits 4, rock and concrete block — and it fades 2 per tick. Loudness: sneak step 2, walk 4, sprint 7, dig 8 per tick, container open 3, body pickup 4, dropped item 5, extraction channel 9, melee swing 3 + gear noise, gunshot 8 + gear noise; water adds 1, grates 2. Sewerats hear at 2, scouts at 4. The footprint is drawn as a brief cyan dither on the tiles it reached, pipes included, so nobody learns the rules through failure.

**Fog** has three states per tile — unseen, remembered, visible — drawn as a Void ordered dither at the art's own pixel grid (solid, heavy, by light). Remembered tiles persist per route in the save (`route_memory`), so a route is learned across runs and a death still pays in map. A creature outside visibility but inside hearing range is drawn as a one-pixel stir in its tile's dither for the tick it moved.

**Loot sources**: lockers and crates (tiered by chunk), scrap deposits (dig 4–10 s, loud, into a pile on the tile), newcomer corpses (what a person carries down here; the body itself carried out for a recruit, filling half the rig and slowing you), Oathkeeper drops after kills, sewerat salvage, caches hidden behind a Sever (your Mat can grow into a sealed room on a later run if you leave a Bait), and the Vault chunks in S5 behind locks. Tables are data (`data/loot.toml`): each source names its picks, a NoDrop chance, a condition band, a category pool with weights and one tier row per stratum; a pick resolves NoDrop, category, tier, then an item uniformly from that category and tier. Rows are steep and gate the top the way level does: Rare enters at tier 3, Blackline at 4, Relic only in caches; in S1–S2 a crate's +1 tier buys picks and condition, not a higher row, so S2 tops out at Uncommon. Notable picks (Rare and up) run on a pseudo-random distribution — the chance ramps by a constant each miss and resets on a hit — so dry streaks and hot streaks both stay short; the constant is solved per row so the average still lands on the listed share. The first three container opens of a route's first run roll one row up. Contents roll from (route seed, run nonce, source index): where a locker is can be learned from the route card, what is in it cannot. Measured: mean value per container doubles per stratum (S1 9 → S2 19 → S3 37 eddies).

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

**Followers.** Up to two colony slimos come along and fight without being told: fists (4) or the melee they carry, one HP pool of 60 % of a Runner's total, the same Down and Dead timers, and creatures hunt them as readily as the Runner (a creature takes the nearest target it can see). Three orders: Follow (the default: stay within three tiles, path around walls), Hold (stand), Guard (hold the tile they stand on and strike anything within three tiles, then return). A Runner who goes Down is picked up by the nearest Follow-order follower within six tiles, carried toward the nearest extraction tile at 0.6× walk speed for up to twenty tiles and set down there; a carrier standing on an extraction tile holds the channel for both. Items dropped by the Runner (the last thing taken, onto the tile as a pile) are followers' business only when ordered to guard them.

**The Runner's body** is four sections — head 20, torso 50, arms 30, legs 30, scaled by Body — and every attack type lands on a section by its own odds (a bite: legs 40 / torso 40 / arms 15 / head 5; a shot: torso 50 / arms 20 / legs 20 / head 10). Armour on the matching slot subtracts. A section at 0 transfers further damage to the torso; head or torso at 0 is Down; arms at 0 double dig and reload time and halve melee; legs at 0 halve speed, forbid sprint and double the extraction channel. Statuses: Bleeding (bites and blades; 1 HP per 4 s to the torso for 60 s or until antiseptic), Blinded (sight radius 1 for 4 s), Chrome Shock (Blackline weapons; locks one quick slot until a Bench clears it), Soaked (20 s after leaving water; lume halved), Fungal Fever (from a Burster; persists into the colony).

**Extraction.** Reach your own Mat's edge, or a grate you hold, or a neutral grate for a toll (eddies or an item). Channel 10 s; noise spikes (9 per tick); followers within 4 tiles extract, the rest are stranded at the extraction tile and recorded on the route card for the next run. A run ends in an outcome record: extracted or died, ticks, the rig or the corpse, followers extracted / stranded / dead, the Mat haul, regrown gear, kills, chunks seen, route memory — the colony reads this, the run never reaches into the colony.

**The shift clock.** A run is one shift: 8 min + 70 s per chunk beyond eight, so 8:00 for the shortest route and 15:00 for a fourteen-chunk one, counted down on the HUD and printed on the route card. Past zero the clock counts overtime in Hot Pink and the sewer answers every minute of it: another pack crawls out of a random burrow, and from S2 a patrol of two scouts rolls onto a chalk-marked chunk that has none (the Flood joins the overtime in Phase D). Nothing ends the run but the Runner; overtime only makes staying cost more.

**The Run HUD** is minimal and puts the senses on the Runner, where the eye is. Around the Runner, in world space: the exposure arc (left half-ring, Ooze Glow, filled by exposure / 15), the noise arc (right half-ring, Cyan, filled by the loudest sound at the Runner's tile in the last 0.3 s / 15), the notice ring (a dotted circle at the true notice range, so the distance at which anything can notice you is a shape on the floor), and during a channel the channel ring (filled by progress: Ooze Glow for extraction, Dust for digging). Top centre: the shift clock in Cyan mono, Acid under a minute, Hot Pink in overtime; under it the route strip, one square per chunk in placement order, lit when seen, Cyan for the one you are in. Bottom-left: the body as a silhouette of four sections (head, torso, two arms, two legs), each filled by its HP in Ooze Glow, Hot Pink at half, flashing at zero; the stamina ooze bar beside it; statuses as tags under it (Hot Pink for damage states, Acid for alerts); the Down countdown when down. Bottom-right: the hand weapon and the four quick slots as icons with their keys, the used slot flashing. Right edge: a chip per follower with name, HP bar and order letter, Hot Pink when down. Bottom centre: one prompt line, the key in Cyan and the verb in Chalk — OPEN, PRY, TAKE ALL, DIG, HOLD TO EXTRACT — and the controls hint for the first six seconds of a run only. The loot panel is grid to grid: on open it slides in at the right (the camera eases east so the Runner stays clear of it) with the source's items unrolling one per 0.22 s as tiles at their real size in their tier colour on the left and the rig's 8 × 5 grid with its placed items on the right; TAKE ALL moves what fits across and tints what does not Hot Pink. Event labels (damage, DOWN, EXTRACTED) flash above the body they belong to.

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

**The four verbs** are on every item's card, in this order, as buttons: Sell (queues to the market of choice, resolves at shift end), Feed (choose the Pit, queues a hauler), Craft (opens the Bench with this item pre-slotted), Keep (pins it; pinned items are excluded from bulk actions and count toward sets and the Scrapdex). Bulk actions: sell all unpinned Common Chrome, feed all Parts below condition 20.

**Bench.** A Tweaker Bench has recipe tiers 1–4 (research) and a queue. A recipe takes items by craft tag rather than by name (Blade + Housing + Conductor = scrap sword, whose stats come from the parts' tiers and condition), so there are 60 recipes and thousands of outcomes. Crafted gear carries the Runner's name and the parts' provenance. Products: gear, implants, Downer, cold-light cells, filter cartridges, salt bags, Spored tags, repairs.

**Appraise** is a Bench job and a slimo skill; a D1 trader with Appraise 4 identifies a Blackline item instantly, which makes that slimo worth protecting.

**Hoarding pressure.** The Stash is finite, prices drift, the Mat is hungry and the Bench wants parts. The design never forces a sale; it lets a full Stash with three unidentified Rare items in it sit on the screen when the next run is loading.

## Runners and slimos

Every slimo is a full character record; a Runner is a slimo you have taken on a run. The record is what the player loses when one dies, so it has to be worth losing.

**Stats (1–10):** Body (HP, carry), Nerve (stamina, dodge, lume control), Hands (melee, dig speed, craft quality), Eye (ranged, appraisal, fog radius), Mat-sense (signal reading, Shepherd bonus, cord riding; set by Mod-Depth), Voice (trade, foreman ability, Mood aura).

**Skills (0–10, XP by doing):** Scav, Fight, Tweak, Shepherd, Trade, Appraise, Medic, Guard. A skill at 5 unlocks its mid-tier job; at 8 its mastery perk (Scav 8: containers show their tier through fog; Tweak 8: one free part per craft; Shepherd 8: tips within 6 tiles extend at double rate).

**Mod-Depth** sets the body, the role ceiling and where they can go (D1 topside; D4 Mat-bound; D5 cannot act), as in Pass 1. **Pasts** (Gang, Nomad, Corpo, Badge, Civ) give a faction bonus and a field bonus.

**Traits**, two per slimo, drawn at creation from 40: Hoarder (+2 rig rows, −1 Nerve), Night Eyes, Loud, Steady Hands, Claustrophobic (Mood falls in 1-wide tunnels), Devout (Mood rises when white light is near, defects easier), Chromehead (feeds Chrome to themselves, +Body, −Mood), Teacher (others gain skill faster in their district), Glutton, Cold Blood (no panic in white light), Pack Rat (notices Curios), and so on. Traits are the reason two D2 Gang scavengers play differently.

**Injuries** persist: Bleeding (treat with antiseptic), Fracture (splint; −Hands or −legs for 3 shifts), Chrome Shock (locks an implant socket until a Bench clears it), Soaked, Blinded, Fungal Fever (from a Burster; Ferality gain doubles until a Lung treats it). A Medic job at a Lung Housing treats injuries over shifts; without it they heal at a quarter rate.

**Gear slots**: head, body, feet, two hands, chest rig, backpack, four implants. Implants are permanent and visible on the sprite.

**Death.** HP 0 is Down; 60 s untreated is Dead. Dead Runners go on the memorial at the Heart with their record; their corpse and gear stay in the run chunk. A district whose foreman dies holds a one-shift wake (no jobs); Mood falls by the slimo's Voice.

**Recruitment.** Newcomers at drains (free, random record), rescued bodies on runs (random record, loyalty bonus), defectors from raider groups you spared (Gang Past, Fight-heavy), hires through faction contacts at reputation +2 (expensive, specialised). Population is the growth cap of everything else, which is why bodies are worth carrying out.

**Bonds.** Two slimos who complete three runs together gain a bond: +1 Nerve when within 6 tiles of each other, and the survivor of a bonded pair takes a Mood hit twice the normal size. No romance system; bonds are a run mechanic.

## Colony mode

The city has every system a player of the genre expects to find, and each one is bent by the fact that the ground is alive.

**Feature baseline and the ROSE version of each**

| Genre feature | In ROSE |
| --- | --- |
| Zones | Stockpile zones with filters (by category, tier, condition), hab zones, quarantine zones, kill zones; a zone must sit on signalled Mat |
| Jobs and priorities | Nine jobs (Haul, Build, Scav, Tweak, Tend, Guard, Cook, Appraise, Research) with a 1–4 priority per slimo per job in a grid; district priorities override when the foreman's are higher |
| Needs | Fed, Lit, Dry, Signal, Rest, Mood; each 0–100 with thresholds that change behaviour (Rest below 20: half speed; Mood below 20: refuses jobs) |
| Schedules | Three shifts per day, Dim / Dark / Flood; per-slimo schedule rows (work / rest / any); Curfew law overrides |
| Storage and logistics | The Mat carries cargo; haulers carry what cords cannot (off-Mat zones); quotas pull; 4 packets per tile capacity; jams visible |
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
- The Charter: laws propagate as a signal wave; eight laws at launch (Rationing, Open Grates, Conscription, Quarantine Protocol, Dissolution Rite, Tithe, Curfew, Foreman's Cut) with costs in Mood and Authority.
- Nerve logic: up to four when/then rules per Relay over signal, nutrient, Attention, water, traffic, white light; actions sever, gate, quota flip, alarm, dim, dump.
- Attention: wealth and light you show at a grate bring the white light down; three stages, each with salvage.
- The handoff: a strain with Plasticity and Conductance above 0.7 proposes its own reroutes.

**Structures at launch (24)**: Heart (4 tiers), Pit, Hab-Pod, Bunkhouse, Scrap Forge, Tweaker Bench, Lume Farm, Lung Housing, Seep Market, Sphincter Gate, Sewerat Pen, Downer Still, Reliquary Vault, Relay Shrine, Stockpile Rack, Cold Store, Scrap Turret, Watch Post (extends danger overlay), Memorial, Charter Board, Appraisal Table, Route Board (run entry), Lantern Post (light without Lume), Salt Works (makes salt from Lung brine).

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
| Telemetry (local) | A per-run cadence log of rewards, losses and decisions, written to the save folder; this is the balancing instrument and the player can read it |

## UI and screens

Twelve screens, one grid, one frame rule. Every panel is a 9-slice whose background ends at the frame line; every label is kinetic (slams in, resolves from a two-frame glitch); numbers in Cyan monospace, alerts in Acid Yellow, Oathkeeper anything in Vestment White. Diegetic labels pin to world objects at Street and District zoom; the Stack view carries only district names and the Attention gauges at grates.

| Screen | Layout | Key interactions |
| --- | --- | --- |
| Colony HUD | Top-left: clock, shift, speed, view. Right edge: alert queue. Bottom: tool bar (Pit, Bait, Salt, Sever, Relay, Build, Zone, Overlay). Bottom-right: minimap of the Stack view | Hover any slimo, structure or cord for its card; click to select; right-click to order |
| Build menu | Categories as tabs; ghost footprint snaps to Cord/Node; cost shown in items the Stash holds, red where short | Rotate, place, queue; upgrade in place shows the next tier's sprite as the ghost |
| Slimo card | Portrait (the sprite), stats, skills, traits, needs bars, gear slots, record | Set job priorities; assign to a run; rename |
| Job grid | Rows slimos, columns jobs, 1–4 priority cells; district filter | Drag to paint priorities |
| Stash | Grid left, item card right with the four verbs, filters and sort on top, containers as tabs | Drag, split, pin, bulk actions |
| Bench | Recipe tiers left, slots by craft tag centre, outcome preview right with provenance | Queue, prioritise, cancel |
| Market | Vendor tabs; stock grid; sell queue; demand bars per category; glut marks; contracts panel | Queue orders; accept a contract; Parley if Ivan is present |
| Strain Lab | Ten gene dials (read-only) with drift arrows from the current feed plan; lineage tree; organ thresholds; Strainbook | Set the feed plan per Pit (tier order and quota) |
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
    crates/logistics         segment graph, flow fields, cargo, quotas, jams
    crates/items             item table, instances, condition, identification, pricing, sets
    crates/inventory         grids, containers, rigs, Stash tiers
    crates/people            slimo records, stats, skills, traits, injuries, needs, jobs, bonds
    crates/colony            zones, structures, tiers, districts, foremen, Charter, unrest, events, Attention
    crates/run               chunk · assemble · map · fov · light · noise · fog · senses · loot · creature · runner · follower · board · sim (the 20 Hz run)
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

**Determinism.** One PCG32 stream per sim; UI and VFX draw from a second; a save replays from seed and input log. The run is `run::sim::RunSim`: every rule of a run — movement, senses, loot, creatures, followers, combat, digging, extraction, the outcome — advances in `step(input)` at 20 Hz on integer and fixed-point state, the input word is logged (`Input::RunInput`), and Godot's Run screen is a view that gathers input and draws state and events. Save format v3 carries `route_memory` per (seed, stratum) and `corpses` (route, tile, day, the rig); migrations begin with the first shipped save format. `run::board::RouteCard` computes a card from the compiled route, the creature file and the route's memory bits, so the Board never counts anything itself; `RunSim` carries the shift clock (`shift_ticks`, overtime spawns from `[shift]` in `data/creatures.toml`). The Stash is one `RoseStash` held by the `Sim` autoload for the whole session; screens are views of it.

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

Research is written into `docs/research/` before each stage's design freeze: extraction-genre loot economies and loss design (01), survivors-genre cadence (02), colony-builder job and need systems (03), pixel-art craft (04), PixelLab constraints (05), Ivan Ooze's canon (06), light, noise and fog as stealth systems (07), loot tables and streak control (08), threat awareness and per-section bodies (09), followers and what loss keeps (10), the embark screen and the run HUD (11). Each research note ends with the decisions it changed, and those decisions are written into this document in the same pass — this document is the plan; a stage that leaves its numbers only in a research note or a verification log has not finished.
