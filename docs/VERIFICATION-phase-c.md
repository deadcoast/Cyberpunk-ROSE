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
