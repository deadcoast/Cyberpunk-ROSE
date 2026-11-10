# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-10 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 7.5 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:31 | 29 | 5 | 8.0 s | 8.0 s | 7 | 29 | 0 | 0 | 0 (+0 −0) | 0 | 255 | 198.8 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 3:17 | 28 | 13 | 7.7 s | 7.5 s | 7 | 44 | 0 | 0 | 60 (+61 −1) | 0 | 263 | 231.1 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:11 | 27 | 11 | 7.9 s | 7.5 s | 7 | 59 | 0 | 0 | 62 (+3 −1) | 0 | 276 | 262.1 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:14 | 27 | 11 | 7.9 s | 7.5 s | 7 | 66 | 0 | 0 | 65 (+4 −1) | 0 | 293 | 293.2 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 3:25 | 31 | 14 | 7.7 s | 7.5 s | 7 | 73 | 0 | 0 | 73 (+9 −1) | 0 | 293 | 326.9 | 0 | held | - |
| 6 | 6 | Nia 'Oathless' | extracted | 2:59 | 24 | 18 | 8.8 s | 7.5 s | 7 | 84 | 7 | 2 | 78 (+6 −1) | 0 | 295 | 355.3 | 0 | held | Bunkhouse |
| 7 | 7 | Nia 'Oathless' | extracted | 2:58 | 26 | 21 | 7.9 s | 7.5 s | 7 | 95 | 8 | 10 | 84 (+7 −1) | 0 | 298 | 383.7 | 0 | held | Charter Board |
| 8 | 8 | Nia 'Oathless' | extracted | 2:11 | 19 | 14 | 8.1 s | 7.5 s | 7 | 97 | 8 | 10 | 88 (+5 −1) | 0 | 298 | 404.4 | 0 | held | Salt Works |
| 9 | 9 | Nia 'Oathless' | extracted | 3:27 | 31 | 14 | 7.9 s | 7.5 s | 7 | 117 | 8 | 29 | 92 (+5 −1) | 0 | 306 | 435.1 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 3:45 | 34 | 10 | 7.7 s | 7.5 s | 7 | 129 | 8 | 38 | 93 (+2 −1) | 0 | 312 | 468.8 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 2:56 | 25 | 21 | 8.0 s | 7.7 s | 8 | 147 | 8 | 56 | 100 (+8 −1) | 0 | 312 | 494.9 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 3:36 | 33 | 12 | 7.5 s | 7.5 s | 8 | 161 | 8 | 69 | 101 (+2 −1) | 0 | 312 | 526.0 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:24 | 28 | 18 | 8.1 s | 7.7 s | 8 | 180 | 8 | 89 | 104 (+4 −1) | 0 | 312 | 555.3 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 3:00 | 27 | 13 | 7.9 s | 7.5 s | 8 | 195 | 8 | 104 | 104 (+1 −1) | 0 | 312 | 580.7 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 3:22 | 30 | 14 | 7.9 s | 7.5 s | 8 | 215 | 8 | 124 | 106 (+3 −1) | 0 | 324 | 607.6 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:17 | 28 | 11 | 8.8 s | 7.9 s | 8 | 234 | 8 | 143 | 108 (+3 −1) | 0 | 326 | 634.3 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 3:29 | 32 | 13 | 7.7 s | 7.5 s | 8 | 256 | 8 | 165 | 109 (+2 −1) | 0 | 326 | 662.6 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | extracted | 2:58 | 27 | 10 | 7.7 s | 7.4 s | 8 | 272 | 8 | 181 | 109 (+1 −1) | 0 | 326 | 686.4 | 0 | held | - |
| 19 | 19 | Nia 'Oathless' | extracted | 3:22 | 30 | 13 | 7.9 s | 7.5 s | 8 | 289 | 8 | 198 | 110 (+2 −1) | 0 | 326 | 712.7 | 0 | held | - |
| 20 | 20 | Nia 'Oathless' | extracted | 3:26 | 27 | 13 | 8.0 s | 7.7 s | 8 | 309 | 8 | 218 | 114 (+5 −1) | 0 | 326 | 739.0 | 0 | held | - |

**Alerts standing at the last shift's end**: 218 on the Heart's floor: no room in the grid

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
