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
the table's numbers, so `docs/telemetry/cadence.md` is unchanged by this PAC (checked: same aggregate).

The game: `RosePeople` (bridge) holds the roster for the session; `Sim.ensure_people()` founds six from the save's
seed. The Route Board's crew is the roster's eligible slimos — click a name for the Runner, COMES ALONG / STAYS HOME
for up to two followers — and the rig is packed to the Runner's rows and carry. The Run starts from the record's
setup and hands back a `RunReport`; `Sim.take_outcome` applies it (XP, bonds, injuries, the dead to the memorial)
and `Sim.end_shift` moves every record's needs and Mood, giving the shift's crafts, sales and identifications to the
highest utility for Tweak, Haul and Appraise. The Slimo card is on the nav: roster left (name, depth, status, Mood),
the record right (stats as gauges, skills as ten-step bars with XP to next, traits with their text, needs with the
Mood target marked, injuries, the record line, bonds, a row of priority cells, the memorial), SEND ON A RUN opens the
Route Board with them as the Runner.

Reviewed by eye (`docs/shots/slimo.png`, `route.png`, `play_11_slimo.png`): the placeholder font had no `'` or `;`
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

Reviewed by eye (`docs/shots/colony.png`, `colony_district.png`, `build.png`, `play_11_slimo.png`): the first
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
