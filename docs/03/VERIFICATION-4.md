# VERIFICATION — Phase C (the city), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-040 — `people` crate: records, stats, skills, traits, injuries, needs (2026-11-09)

Full Design v3.7–3.8 first (Runners and slimos, rewritten to its numbers): Mod-Depth leans and veins, five Pasts,
creation from the save's seed, skills at 10 + 5 × level XP from the run outcome and the shift report, the forty
traits as typed effects, six needs with per-shift rates and a Mood target the gauge chases at +4 / −3, job utility,
bonds in three runs, and — new in v3.8 — what the record does in a run: one rule per stat (Body sections and carry,
Nerve stamina, Hands melee and dig time, Eye sight and ranged, veins lume, the traits' rig rows), the trait flags
the run reads, and the follower back-off line (30 %; Coward 60 %; Fearless never). `data/traits.toml` holds the
forty with their leans, rates, loved and hated jobs, XP multipliers, rows, flags and excluded pairs; the loader
rejects a missing trait, an unknown stat, need, skill or job key, or an excluded pair that names nothing.
`data/names.toml` holds sixty names and thirty epithets.

`core/crates/people`: `Record` (stats, skills and XP, two traits, needs, Mood target, injuries with shifts left,
priorities 0–4 per job, status, the record line, bonds), `Roster` (create, found six, eligible, depart, apply_run,
end_shift, recover, the memorial), `RunnerSetup` (what the run takes) and `RunReport` / `ShiftWork` (what it gives
back). The `run` crate now takes a `RunnerSetup` in `RunSetup` and bends to it where the design says: sections and
stamina by `stat_scale`, melee by Hands and ranged by Eye through `stat_lean`, `Runner::dig_s`, the `Quirks` parsed
once from the flags (see radius, every sound, melee noise, melee and ranged leans, sprint and walk, regen, water at
walk, a bigger dug pile, Eye in the dark, no fever), followers' `back_off_pct`, and a bonded follower within six
tiles as +4 stamina. The Outcome carries what the roster reads (items taken, hits survived, follower strikes, the
statuses carried home). Save v6 carries the roster. The bot and the Phase B tests run on `RunnerSetup::plain(5)`,
the table's numbers, so `docs/03/telemetry/cadence.md` is unchanged by this PAC (checked: same aggregate).

The game: `RosePeople` (bridge) holds the roster for the session; `Sim.ensure_people()` founds six from the save's
seed. The Route Board's crew is the roster's eligible slimos — click a name for the Runner, COMES ALONG / STAYS HOME
for up to two followers — and the rig is packed to the Runner's rows and carry. The Run starts from the record's
setup and hands back a `RunReport`; `Sim.take_outcome` applies it (XP, bonds, injuries, the dead to the memorial)
and `Sim.end_shift` moves every record's needs and Mood, giving the shift's crafts, sales and identifications to the
highest utility for Tweak, Haul and Appraise. The Slimo card is on the nav: roster left (name, depth, status, Mood),
the record right (stats as gauges, skills as ten-step bars with XP to next, traits with their text, needs with the
Mood target marked, injuries, the record line, bonds, a row of priority cells, the memorial), SEND ON A RUN opens the
Route Board with them as the Runner.

Reviewed by eye (`assets/shots/slimo.png`, `route.png`, `play_11_slimo.png`): the placeholder font had no `'` or `;`
(epithets and trait texts rendered as boxes) — nine glyphs added to `tools/src/font.rs`; the card's trait texts ran
into the injuries line — moved; the Board's four crew rows overflowed their frame and the Stash rows ran into the
depart note — compacted to 32 px rows and the rows raised. One first play shot read a Mood target of 85 where the
rules give 65–75; two further traced runs (every record's target printed after the shift) matched the rules
exactly (65 and 70 for the injuries carried home); the first reading was not reproduced and the trace is removed.
Found and fixed while wiring: `Roster`'s derived `Default` started ids at 0 (the Godot class uses `Default`), so
`Default` is now `new()` and ids start at 100; the Cook job carried an invented Medic 2 gate — removed, and the
design now names the skill each job reads.

Tests: 6 in `people` (the forty validate and no excluded pair is drawn; creation follows the depth and Past leans;
needs move by the rates and Mood chases its target at bounded speed; utility follows priority, skill, traits and the
body's forbids; a run is the shift's Scav work and its injuries and the trait lean reach the target; XP levels at
10 + 5 × level), 1 new in `run` (the record bends the run by one rule per stat), the save round trip carries a
founded roster; 97 in the workspace. The smoke test checks the founders, a record, priority cells and utility, the
Board's crew from the roster, depart marking the crew out, and after the run the Runner's record (runs, hauls =
pulls, Scav XP, needs moved, bonds counted).

Known gaps: the roster lives for the session (the save file is not yet written to disk — the Game shell PAC);
Fracture never happens in the run yet; Nerve does not reach dodge or dimming until Phase D's combat pass; the
colony's zones, meals and sleep (PAC-041) will replace the flat shift context (everyone on the signalled Mat).

## PAC-041 — `colony` crate: zones, structures with tiers, districts, foremen (2026-11-09)

Full Design v3.9 first (Colony mode): Building (an anchor on Cord or Node with the structure's signal, the rest of
the footprint on walkable floor or Mat; the Stash pays on the first Build shift; a builder adds Hands a shift, a
Forge doubles it; tiers 2 and 3 cost ×2 and ×3 and need the Quarter and the City), the 24 structures as a table of
footprint, cost, build points, the job each hosts with its slots, and per-tier numbers; zones and filters; homes,
meals, light, water and sleep, which are the record's `ShiftContext`; schedules (Work / Rest / Any per shift of the
day, Work / Rest / Any by default, so a city sleeps in Dark); districts and foremen; petitions. Research 14 (the
Dwarf Fortress mayor) decided the election (Voice + Mood / 20, Orator ×2, a term of ten shifts, an election at
once when the foreman dies or leaves) and turned the mayor's mandates into petitions with a deadline and a Mood
consequence, with no punishments and no rooms for the office. `data/structures.toml` holds the 24; the loader
rejects a missing core structure, an unknown hosted job, a free or pointless build.

`core/crates/colony`: `StructureFile`, `Colony` (found, can_place / place / place_built / cancel / upgrade,
paint_zone with `Filter`, assign_homes, form_districts, elect, priority_for, assign_jobs, end_shift → `ShiftOutcome`),
saved in v7. The founding stands the Heart on its node, a Route Board and a tier-1 Bench beside it, and a Pit on every
Relay. `people` gained `Duty` and a schedule row on the record, `utility_at` (the foreman's priority where higher)
and `on_duty`. `bench` gained `end_shift_slots` so the Bench's Tweak slots set how many jobs advance.

The game: `RoseSim` founds the city with the Mat and runs the whole shift in one call (`colony_end_shift`): the
colony hands out the slots and pays ghosts out of the Stash (the cheapest Parts first), the Stash resolves its queues
and the Bench's slots, the Pits take the nutrient and the genome drifts, the shift's salt, doses and identifications
land, lanterns and paste come out of the ledger, and the roster takes the work, the clinic's treatments and every
slimo's context. The Build menu reads the 24 from the crate with their numbers and arms the Colony's build tool; the
Colony draws structures and ghosts on their footprints with a build bar, a ghost cursor that names why a tile is
refused, right-click to cancel an unpaid ghost, a city line (tier, day and shift, built, ghosts, beds, districts, what
a ghost waits for) and, in District view, each district's reach, name, foreman and petition. The Slimo card shows the
slimo's bed, last shift's job and district (FOREMAN), and the schedule row.

Reviewed by eye (`assets/shots/colony.png`, `colony_district.png`, `build.png`, `play_11_slimo.png`): the first
placement rule asked every footprint tile to be Mat, and the probe found no Bunkhouse anchor anywhere on a grown
Mat (cords are one tile wide) — the rule became anchor-on-Mat, footprint-on-floor, written into the design; zones
follow the same logic. The Colony's text ran over lit tiles — framed; district labels drew at world scale and
floated outside the reach — drawn at screen scale inside it on a dark backing; the Build previews were cropped and
the Heart's text broke mid-word — fitted and wrapped; the Heart has four tiers and the art three — `t4` added. The
Runner's card showed a Tend shift on the shift they ran: a run is the shift's work, so the colony skips anyone whose
shift job a run set, and they do not count as asleep (tested). The smoke then read Rest 52 where 88 was expected:
ten slots and six slimos meant everyone worked every shift — the rules were right and the design's schedules were
missing, so the schedule went in (tested: Dark sleeps by default, Any sleeps under Rest 40). The card's Mood target
of 65 for the Runner is the shift's carried-home injuries (Bleeding, Soaked, Fever at −5 each) counted before the
one-shift ones heal. `.gitignore`'s Python `build/` rule had kept `game/modes/build/` out of the repo since Phase A;
anchored to the root, and the Build screen is committed for the first time.

Tests: 9 in `colony` (the 24 load and the founding; placement by Mat, signal and footprint; building pays once then
adds Hands and needs the Quarter to upgrade; homes, meals, light and sleep reach the contexts; a district forms,
elects by Voice and petitions for a failing need, answered by a lantern; zones by cell and filter; the schedule;
jobs to the highest utility one slot each; a Runner home from a run takes no slot), the save round trip carries a
colony; 106 in the workspace. The smoke places a lantern ghost from the Colony's tool, checks a Vault refuses at
Heart tier 1, and after the run finds the lantern paid for and built, the Heart's district formed with a foreman
who is a member, and the Runner with no job on the shift they ran.

Known gaps: haulers do not yet carry Stash items to stockpiles by filter, and a foreman's cargo demand waits for
the routing pass (PAC-043); unrest lines are read off the gauge but strike, defection and the Charter are PAC-042;
structures take no damage until floods and raiders (Phase D); the Heart's tier does not yet grow the Stash grid.

## PAC-042 — the Charter and unrest (2026-11-09)

The source: `docs/02/Pass 2 (pillars).md` holds the Charter's eight laws with their effects and costs, the unrest
lines, Authority and the Dissolution row. FullDesign had compressed all of it into one line and lost it (the Phase A
placeholder's law effects were Pass 2's; the plan I had started from was not). Full Design v3.10 carries Pass 2's
table in full with numbers and a column saying which effects are live now and which wait for their system, the wave
(16 + 32 × Conductance cords a shift), Authority (a district formed +1; first law costs 1, so the first district pays
for the first law inside Pass 2's 20–40 minute beat; the Authority column is this design's, Pass 2 gives none), the
vote (each district with a Board votes through its foreman), Ivan's four states from Authority earned, Dissolution
(0.30 nutrient, −10 city-wide without the Rite, this design's number) and the unrest lines by district Mood. Research
15 (Frostpunk's Book of Laws) decided no signing cooldown (Authority already paces), no exclusive pairs, the
promise is the petition, and repeal is free without refund. Pass 2 corrected three of PAC-041's calls, written back:
a district reaches 12 cords along the Mat, not 8 tiles in a straight line; the default schedule is Any / Any / Any (my
Work / Rest / Any had already sent the city to bed in Dark and left Curfew nothing to do); and laws reach every district
by the wave — the Charter Board is the district's voice (its vote and its petitions), not a gate on the law.

`data/laws.toml` holds the eight with typed flags (an unknown flag is refused). `colony::charter`: `LawFile`,
`Charter` (sign, repeal, advance, in force at a cord distance, Ivan's state), `cord_distances` / `cord_reach`. The
colony now forms districts by cords, pays Authority for a district formed, reads unrest at the start of each shift
(strike after two shifts under 30, ready to defect under 20, the foreman's petition under 10 naming the costliest law,
two refusals dissolving the district and barring its Node for ten shifts), applies the laws that have reached each
slimo (Rationing halves the meal's draw and the paste and costs 5, Curfew empties the Dark shift, the Rite removes
Dissolution's hit for a standing 3), and announces each law once per district as the wave arrives. Save v8 carries
the Charter.

The game: the Charter screen draws the eight cards from the crate with their state (open, spreading, signed,
repealing, locked with the reason), where the wave has reached, the vote, SIGN and REPEAL; the districts with Mood,
strike and defection lines and their petitions (ANSWER repeals the law asked, or arms the Build tool with the
structure; REFUSE); the board's history; Ivan's portrait in his state. A D5's Slimo card shows its feed value and
DISSOLVE.

Reviewed by eye (`assets/shots/charter.png`): the effect lines lost their third line and the "waits for" notes were
cut mid-word — effect and cost now share four lines and the notes are short in the data; the buttons sat on the cards'
border and Ivan's portrait overran its frame — moved; a law signed between shifts read "in force: the Heart" while the
district's hub was inside the wave — the card now shows where the wave stands, the shift announces arrivals. The
smoke caught the Charter without art when I first dropped Ivan's portrait.

Tests: 2 in `colony::charter` (the eight from Pass 2 and a bad flag refused; a signed law walks out a shift at a time
and its repeal after it, without refund), 7 new in `colony` (a district holds what stands within 12 cords and its
forming pays the first law; Rationing halves the draw, costs 5 and reaches the Heart at once, and its repeal is free;
strike, defection, the petition at 10 naming Rationing, two refusals dissolving; answering a repeal petition; a D5
dissolved with and without the Rite; Any works while rested and Curfew empties the Dark), the save round trip carries
a signed law; 113 in the workspace. The smoke signs Rationing on the Charter screen once a district has paid the
Authority, checks Tithe refuses outside the accord, and after the run finds Rationing in force in the Heart's
district.

Known gaps: defection has no destination until feral strains and Patrols at the colony (Phase D), so the line shows
and nobody leaves; Open Grates, Conscription, Quarantine Protocol, Foreman's Cut and Tithe sign, cost and travel but
their effects arrive with their systems (named on each card); a Bunkhouse or Pod sleeper's defection pause needs
defection first; Ferality, named in Pass 2 and once in the design, is still undefined.

## PAC-043 — logistics: demand-weighted routing, quotas, jams on the Stack view (2026-11-09)

The source: Pass 2's Automation pillar (cargo routing, quotas, capacity) and its backend list, carried into Full
Design v3.11 with numbers (Colony mode: The city's clock, Cargo). What I found first: the game's colony was Phase A's
scale proof — three thousand slimos walking market → nearest Pit and feeding 0.3 on arrival, every market landing
12–30 synthetic units every 15 ticks, and a FEED tool that dropped 5 nutrient anywhere — so the Mat lived on food
nobody sent. Measured (research 16): without it the founding Mat loses a fifth in five minutes at 2 Hz; at Pass 1's
125-step shift it recedes about 5 % a shift unfed and its core holds on ~4 nutrient a shift. Decisions written back:
the Mat steps every 12 ticks (125 steps in a 12.5-minute shift) while cargo and slimos keep 2 Hz; the Mat eats only
its Pits, its Deposits and Bait paid from the Stash; the phantoms stay in `rose scale` only; the founding is named
for what it is (more Mat than one Runner feeds) until chapter 1 replaces it; hauls arrive at the Heart, so the split
to quotas happens there by Pass 2's routing rule; a quota counts what is riding to it (Factorio's in-transit rule).

`logistics`: `City::game` (no crowd, the Mat clock, markets are the grates the Mat reaches, index-stable), consumers
with signal-weighted Dijkstra fields (a tile costs 1 + 2 × (1 − σ)), `choose` by demand per tile with low priority
last and ties toward the Heart, packets with cargo ids released from their source at capacity, re-routing at each
rebuild, spills off dead Mat, jams as a full tile with cargo waiting to enter it (half speed there), the roster's
movers on walk fields; the scale proof behind `synthetic`. `inventory::Ledger`: shipments in flight, `dispatch_queue`,
`pull` (the quotas, lowest tier then cheapest, never pinned), `land` (a sale at the glut it finds, a feed's nutrient
and drift, a Part into a Still's store, a product home or held at the Heart), `spill`, `divert`, `close_shift`.
`colony`: quotas on Pits and Stills (`quota_of`: yours, the foreman's — what the members ate from the Mat last shift
— or a Still's slots; capped at the Pit's stock), the Foreman's Cut (one landing in twenty), the Pits as the Heart
then every built Pit, the Still and Salt Works per slot. Save v9 carries the cargo, the stores, the held products,
the report so far and the quotas. The bridge lands arrivals as the city ticks (`bind_stash`), dispatches at the shift
end, refreshes demands every rebuild, and returns the Stack view drawn by `render::stack_view`.

The game: SELECT replaces FEED; clicking a Pit or a Still opens its quota card (keep, whose, stock, riding, the tier
ceiling, DEFAULT); Bait takes a Common Chrome from the Stash or says it has none; packets ride as pulses in their
tier's colour (Street, District); the Stack view is the core's circuit board with jams Chalk white; the slimos on the
map are the roster by depth, walking to work or home; the Stash's ledger shows what landed this shift and what is
on the Mat; the Route Board takes the crew off the map.

Reviewed by eye (`assets/shots/colony*.png`): four packets on a tile overlapped into one square — drawn side by side
now; the Stack view at a thirty-second zoom only sampled the placeholder tiles' texels and the traffic vanished in the
Mat's own pinks — it is now the core's one-pixel-a-tile image; every convoy of four the Heart released counted as a
jam forever — a jam is now Pass 2's "above capacity", a full tile with cargo waiting behind it; the quota card sat
over the trunk the cargo rides — moved above the tool bar; the District view still drew PAC-041's 8-tile box though
PAC-042 made districts 12 cords — it draws the true reach now, and the Cut's stock when there is one.

Tests: 1 in `logistics::field` (the bright trunk beats the dim shortcut), 5 new in `logistics::city` (the game city
has no phantoms and keeps its own Mat clock and its markets are grates; cargo goes by demand per tile and lands once;
sixty at once jam and all land; a lost consumer's cargo re-routes; movers walk to work and home), 5 in the ledger
(a sale pays when it lands at the glut it finds; a feed lands where it arrives; a quota pulls cheapest-first, counts
what rides and skips the pinned; the pull serves demand per tile and low priority last; a Still keeps its Parts, a
product rides home, a spill is lost), 1 in `colony` (quotas default off / the foreman's / a Still's slots, yours
capped at the stock, the Cut takes one in twenty once it reaches the district); the save round trip carries cargo and
a quota; 123 in the workspace. The smoke checks the movers are the roster, the quota card, Bait's cost, the Stack
image, and that a feed and two sales leave at the shift end, pay nothing on the way and land (the genome drifts, the
eddies arrive) as the city ticks.

Known gaps: stockpile zones hold nothing yet and haulers carry nothing off the Mat (PAC-044, with the Colony
screens); Salt, Sever and Relay on the tool bar are still the Phase A verbs without their costs (PAC-044); the Vault
takes cargo when Attention exists (Phase D); Patrols do not yet path toward a white trunk (Phase D); the founding is
the scale proof's stratum until chapter 1 (Phase E); the Stack view shows one stratum, not the stack of six.

## PAC-044 — the Colony screens live, the shift in core, `rose city` (2026-11-09)

The source: Pass 2's UI rows for the Colony HUD, the Build menu and the Job grid, its stockpiles and haulers ("haulers
carry what cords cannot"), Pass 1's costs for Salt, Sever and the Relay, Pass 1's newcomer flushes, and the Build
plan's Phase C proof, carried into Full Design v3.12 with numbers (Colony mode: Stockpiles, The steering tools,
Orders, Alerts, Overlays; Stash; Recruitment; The city's log; UI and screens) and research 17 (RimWorld's storage
priority). Decisions written back: a stockpile is a zone against a built Rack or Cold Store and a room of the Stash
(1–4 on a job's scale, the Heart's grid between 2 and 3; a new Rack 2, a new Cold Store 3 with an Organic filter);
cargo lands on its Mat cells, a hauler shelves its Body in items a shift onto the rest; Organics spoil 2 a shift
outside a Cold Store; Salt is a bag per eight tiles, Sever a cut of 8 Build points, the Relay a 6 Chrome / 16 point
structure, Bait a Common Chrome; orders, alerts (a jam that has stood 120 ticks), overlays, bookmarks and follow;
the Heart's tier is the city's and the grid follows it; build work is taken first asked, first done, and a crew whose
ghost the Stash cannot pay goes on to the next.

The core: the whole shift moved out of the bridge into a `game` crate — `end_shift` (the colony's work, the Stash's
side, the cargo, the haulers, the settling, the roster), `run_home`, `alerts`, the city's log, `World` (the city owned
in one place) and `city` (`rose city`). The bridge calls the same functions with its parts. `inventory` has the rooms
(`Room`, `SHELF`), the Heart's floor (`FLOOR`) and every verb across them; `colony` has cuts, salt, orders, turned
footprints, stockpile painting, zone priorities and filters, the Relay as a structure; `logistics` sends cargo only
toward a consumer it can reach and lets head-on packets trade places; `people` has the flush.

The game: the Colony HUD per the UI table — the clock and city line, the tool bar (PIT, BAIT, SALT, SEVER, RELAY,
BUILD, ZONE, OVERLAY) with each verb's answer on a hint line that fades, the alert queue on the right edge (one new
line a second, click to jump), the minimap of the Stack view with the camera's frame, the selected thing's card above
the tool bar (a structure's upgrade, cancel, quota, stockpile priority and filter, its slots and who works them; a
slimo's job, needs, order and FOLLOW; a tile's Mat, signal, traffic, zone, salt and cut) and the same card small under
the cursor; right-click orders, cancels or puts the tool down; R turns a footprint; 1–9 and Ctrl + 1–9 bookmarks,
F follow, + / − speed, Tab view. The Stack view now frames the whole stratum at three pixels a tile with the district
names. The Build menu pages by category with costs against what the Stash holds; the Job grid is the roster's
priorities (click cycles, a drag paints, the foreman's mark, a district filter); the Stash pages between the grid,
the Heart's floor and each stockpile; the Strain Lab reads the genome and every Pit's quota (organs and the
Strainbook say PAC-080). The city's log is written to `user://telemetry/city.log`.

Reviewed by eye (`assets/shots/colony*.png`, `build.png`, `jobs.png`, `stash.png`, `strain.png`): every gauge in the
game drew full whatever its value — a Control clips its children, not its own drawing — the fill is now a strip
inside a clip (the Strain Lab's genes, the Run's stamina); the Stack view at one pixel a tile was a 192 × 48 stamp in
the middle of the screen, the same picture as the minimap — three pixels a tile now, the stratum across the screen;
the Build card's build-points line ran off the frame; a verb's answer stood on the hint line forever; the
Job grid's status read as Rust's debug text; the Stash's free UPGRADE ignored that the grid is the Heart's tier —
gone. The Phase A free verbs (free salt, free sever, free Relay, free feed) left the bridge.

`rose city` (`docs/03/telemetry/city.md`): twenty shifts in 7 s, played twice and compared. What it found and what
changed: the city parted from its replay (the haul took the backpack in its map's order, and build crews were paid
in a hash map's order) — ordered now, bit-exact; a bot stood on a source it could not take from for forty minutes —
it gives one up after five seconds; four sales for a grate the Mat had cut off sat on the Heart's tile and every
cargo after them waited behind (cargo now leaves only toward a consumer it can reach, else it is the Stash's again);
feeds riding from the Rack and its overflow riding to it held one cord shut for good (head-on packets trade places);
no home ever ate from the Mat, because the tissue round a fed Heart holds 0.13–0.19 and the meal wanted 0.2 (the meal
is now its own threshold); the run's overflow, two hundred items, sat out of the economy (the Heart's floor: the
quotas pull it, the verbs and the stockpiles reach it); and with a Runner dying one run in five the six founders
were two (the flush, Pass 1's numbers). Now the city replays, its cargo lands, the plan stands, the Heart keeps its
signal and the merged landing gap is 7.7 s against the bot band of 3.3–10 s. It still fails: the founding Mat
recedes, the cord to the first grate goes at the fifth shift with or without a steward, income stops, and four
slimos starve from the eighteenth shift — registered as PAC-045, where Phase C closes.

Tests: 8 in `game` (a full Rack's overflow and a Cold Store's Organics; haulers shelve off the Mat for Scav XP; salt,
cut and Relay take their costs; an order beats priorities; alerts name a waiting ghost, a petition and the floor; the
log; a stockpile's filter and priority are the zone's; two shifts of `rose city` replay bit-exact with their cargo
landed), and the ledger's, the colony's and the save's updated for the rooms, the floor and v10. The smoke drives
the HUD's verbs (select, the quota, bait's and salt's costs, sever and unmark, RELAY arms Build, right-click puts the
tool down, the overlays cycle, the alerts) and checks the Stash founds from core's haul.

Known gaps: the city does not yet hold twenty shifts (PAC-045); noise and danger overlays wait for the threats
(Phase D); rescued bodies, defectors and hires arrive with their systems (Phase D); a stockpile is painted whole
against its store and cannot be extended (erase and repaint); the organs, classes and Strainbook are PAC-080's.

## PAC-045 — the city holds twenty shifts; Phase C closes (2026-11-09)

The call between a smaller Mat, a different starting economy and a steward who feeds the cord to the market came
out as all three, each at its measured part (Full Design v3.13: The founding, The Mat's metabolism, What PAC-045
found). The founding is Pass 2's first session to 5:30 played before the first run: the Heart grows on Pass 1's
feed with half of it painted ahead of the Mat toward the first grate until the Mat reaches it (77–439 steps, 86–336
cells on seeds 11–26), then the Pit holds six Junk Chrome. The Mat's upkeep and decay are an eighth of Pass 1's,
which it tuned on a feed about fifty times what a haul brings the Heart. The steward steers with Relays: whenever the
cord to the first grate is lost, a Relay at a time on the anchor joined to the Heart nearest the grate along the
ground the Mat can grow on, the build plan waiting after the first Pod while it does; the Relay stands on any
signalled Cord or Node. Meals draw the tissue about the home, richest tile first.

Measured on the way (scratch probes, not shipped): at half Pass 1's rates the founded cord is lost within 2–8 shifts
on all sixteen seeds fed or not, at a quarter on nine (won back on five), at an eighth on two (both won back). From a
founding fifteen tiles out, the haul painted ahead of the Mat reached the grate on 1 seed of 8 in twenty shifts, three
hauls on 6, a Pit fed at the far end held a founded cord on 1 of 8, and a Relay a shift at the frontier reached it on
8 of 8 in 5–8 shifts; Relays held to Nodes mostly found none near the frontier. Unsteered, the Mat on Pass 1's full
feed for 6000 steps found the grate up its shaft on 2 seeds of 8. Left alone with the Deposits the founded cord holds
on 13 of 16 unfed and 14 fed; with the Deposits off the map, 0 and 7 (two hauls) — the feed's weight against the
ground is registered for PAC-061.

What else it found and what changed: `rose city` ticked the city at 2 Hz on the bot's clock, so the Mat lived about
35 of its 125 steps a shift — the city now takes three ticks for every ten of the bot's, the bot factor; the first
Relays went up a shaft that dead-ends under rock because the steward measured across, not along, the ground — it
walks the ground now; a Relay placed to win the cord back stood on Mat cut off from the Heart — it stands on the
Heart's side of the break now; two processes disagreed on a replay of seed 26 at the fourth shift (the Pits drawn
in a map's order, a rig's weight summed in a map's order) — both in a fixed order, and seeds 13, 21 and 26 replay
across processes; `err.txt`, a probe's output committed with PAC-044, left the repo.

`rose city --shifts 20` on seed 11: PASS — the cord held all twenty shifts, the plan stood by the eighth, eight
slimos at the end (the flush joined), 114 eddies, nobody starving, cargo landed, the merged landing gap 7.5 s against
the bot band of 3.3–10 s, bit-exact replay, 6 s. Over seeds 11–26 the cord holds or is won back on all sixteen and the
plan stands on eleven; on 13, 14, 15, 23 and 26 the first route card kills 4–8 of twenty Runners and the city thins
to 0–3 before its plan stands — the steward never changes card.

Reviewed by eye (`assets/shots/colony*.png`, the play frames): the Stack view shows the founded Mat as one climbing
strand from the Heart to the grate at the top edge; the District view the same strand in tiles; the Colony HUD reads
seven slimos and one district on day 2. Tests: 17 in `colony` and 8 in `game` (a cut found anywhere on the founded
Mat; the Heart's quota above the founding Pit), `mat`'s die-back check restated (no feed and no Deposits: 300 cells to 198 in forty shifts); the
smoke checks the founded Mat reaches its grate, three founding structures, and Rest 88 for a Runner who slept the
three shifts the founding's few slots left them; save v11 (the founding and the rates change what a seed and its
inputs replay into).

Known gaps: the feed-or-sell pressure on the Mat is the economy pass's (PAC-061); route choice when a card keeps
killing is the player's and the Route Board's, and the steward never makes it; a Relay is two founder-shifts of Build
at Hands 5–7, so a broken cord takes four to six shifts to win back.
