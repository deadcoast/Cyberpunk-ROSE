# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-09 (Full Design v3.12, The city's log; Recruitment; Build plan, Phase C). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, the build plan (pod, pit, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart but the Pit, which goes at the nearest anchor to the first grate and keeps 2 nutrient too (Pass 2's first session), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at 2 Hz (the run's clock and the city's are one), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: shift 18: 4 starving; shift 19: 4 starving; shift 20: 4 starving (the founding Mat recedes and takes the cord to the first grate with it; PAC-045).

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 7.7 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:31 | 29 | 5 | 8.0 s | 8.0 s | 7 | 29 | 0 | 0 | 0 (+0 −0) | 0 | 1533 | 520.3 | 0 | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 4:05 | 36 | 14 | 8.8 s | 7.8 s | 7 | 46 | 0 | 0 | 53 (+61 −8) | 4 | 1573 | 459.0 | 0 | Pit |
| 3 | 3 | Nia 'Oathless' | extracted | 3:38 | 31 | 12 | 8.1 s | 7.9 s | 7 | 56 | 0 | 0 | 47 (+3 −9) | 4 | 1578 | 417.3 | 0 | Lantern Post |
| 4 | 4 | Nia 'Oathless' | extracted | 2:33 | 22 | 17 | 8.1 s | 7.4 s | 7 | 69 | 0 | 0 | 48 (+10 −9) | 4 | 1568 | 393.8 | 0 | Stockpile Rack |
| 5 | 5 | Nia 'Oathless' | extracted | 3:37 | 33 | 7 | 7.7 s | 7.5 s | 7 | 75 | 0 | 0 | 43 (+4 −9) | 4 | 1512 | 379.0 | 0 | Hab-Pod |
| 6 | 6 | Nia 'Oathless' | extracted | 3:50 | 37 | 20 | 7.9 s | 7.7 s | 7 | 98 | 6 | 11 | 36 (+0 −7) | 3 | 1472 | 386.4 | 0 | - |
| 7 | 7 | Nia 'Oathless' | extracted | 3:30 | 31 | 14 | 7.9 s | 7.7 s | 7 | 118 | 7 | 30 | 29 (+0 −7) | 3 | 1458 | 394.2 | 0 | - |
| 8 | 8 | Nia 'Oathless' | extracted | 3:34 | 32 | 14 | 7.7 s | 7.5 s | 7 | 138 | 7 | 50 | 22 (+0 −7) | 3 | 1453 | 384.0 | 0 | Bunkhouse |
| 9 | 9 | Nia 'Oathless' | extracted | 2:54 | 25 | 11 | 7.9 s | 7.5 s | 7 | 158 | 8 | 69 | 15 (+0 −7) | 3 | 1461 | 369.8 | 0 | - |
| 10 | 10 | Nia 'Oathless' | extracted | 3:33 | 34 | 10 | 7.5 s | 7.5 s | 7 | 174 | 8 | 85 | 8 (+0 −7) | 3 | 1457 | 344.5 | 0 | Charter Board |
| 11 | 11 | Nia 'Oathless' | extracted | 3:17 | 28 | 9 | 7.9 s | 7.7 s | 8 | 185 | 8 | 95 | 1 (+0 −7) | 3 | 1440 | 324.7 | 0 | - |
| 12 | 12 | Nia 'Oathless' | extracted | 3:35 | 30 | 8 | 7.9 s | 7.7 s | 8 | 214 | 8 | 124 | 0 (+0 −1) | 0 | 1401 | 319.5 | 0 | Salt Works |
| 13 | 13 | Nia 'Oathless' | extracted | 3:03 | 26 | 4 | 8.2 s | 7.9 s | 8 | 231 | 8 | 141 | 0 (+0 −0) | 0 | 1379 | 321.2 | 0 | - |
| 14 | 14 | Nia 'Oathless' | extracted | 3:13 | 27 | 5 | 7.9 s | 7.7 s | 8 | 253 | 8 | 163 | 0 (+0 −0) | 0 | 1337 | 319.0 | 0 | - |
| 15 | 15 | Nia 'Oathless' | extracted | 3:20 | 32 | 7 | 7.4 s | 6.5 s | 8 | 280 | 8 | 190 | 0 (+0 −0) | 0 | 1300 | 329.9 | 0 | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:03 | 25 | 7 | 8.8 s | 7.7 s | 8 | 303 | 8 | 213 | 0 (+0 −0) | 0 | 1282 | 338.7 | 0 | - |
| 17 | 17 | Nia 'Oathless' | extracted | 3:24 | 29 | 8 | 7.9 s | 7.5 s | 8 | 325 | 8 | 235 | 0 (+0 −0) | 0 | 1260 | 329.1 | 0 | - |
| 18 | 18 | Nia 'Oathless' | extracted | 2:50 | 25 | 7 | 8.9 s | 8.0 s | 8 | 343 | 8 | 253 | 0 (+0 −0) | 0 | 1247 | 330.4 | 0 | - |
| 19 | 19 | Nia 'Oathless' | extracted | 3:32 | 33 | 9 | 7.5 s | 7.3 s | 8 | 366 | 8 | 276 | 0 (+0 −0) | 0 | 1226 | 322.0 | 0 | - |
| 20 | 20 | Nia 'Oathless' | extracted | 3:01 | 25 | 6 | 9.3 s | 8.0 s | 8 | 382 | 8 | 292 | 0 (+0 −0) | 0 | 1201 | 321.6 | 0 | - |

**Alerts standing at the last shift's end**: Pit has lost its signal; Fenn is starving; Xan is starving; Moss 'Oathless' is starving; Eske 'Lantern' is starving; 292 on the Heart's floor: no room in the grid; a Pit's quota: the Mat no longer reaches it

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
