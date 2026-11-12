# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-12 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 7.5 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:16 | 27 | 6 | 9.7 s | 9.7 s | 7 | 28 | 0 | 0 | 0 (+0 −0) | 0 | 101 | 348.1 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 2:44 | 24 | 12 | 8.0 s | 7.7 s | 7 | 43 | 0 | 0 | 49 (+54 −5) | 2 | 112 | 496.1 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:32 | 31 | 10 | 7.9 s | 7.5 s | 7 | 51 | 0 | 0 | 62 (+18 −5) | 2 | 130 | 685.4 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:10 | 27 | 17 | 7.9 s | 7.5 s | 7 | 57 | 0 | 0 | 81 (+24 −5) | 2 | 132 | 856.1 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 3:01 | 25 | 20 | 8.8 s | 7.5 s | 7 | 60 | 0 | 0 | 93 (+15 −3) | 1 | 135 | 1013.6 | 0 | held | - |
| 6 | 6 | Nia 'Oathless' | extracted | 3:00 | 24 | 17 | 8.8 s | 7.7 s | 7 | 76 | 0 | 0 | 105 (+15 −3) | 1 | 138 | 1159.4 | 0 | held | Bunkhouse |
| 7 | 7 | Nia 'Oathless' | extracted | 2:45 | 26 | 15 | 7.9 s | 7.4 s | 7 | 77 | 1 | 0 | 123 (+23 −5) | 2 | 138 | 1300.6 | 0 | held | Charter Board |
| 8 | 8 | Nia 'Oathless' | extracted | 3:18 | 31 | 21 | 7.5 s | 7.4 s | 9 | 91 | 5 | 1 | 125 (+7 −5) | 2 | 142 | 1467.0 | 0 | held | Salt Works |
| 9 | 9 | Nia 'Oathless' | extracted | 3:29 | 33 | 14 | 7.8 s | 7.4 s | 9 | 100 | 5 | 12 | 140 (+22 −7) | 3 | 142 | 1641.2 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 2:46 | 28 | 15 | 7.7 s | 7.3 s | 9 | 114 | 6 | 23 | 142 (+9 −7) | 3 | 147 | 1778.7 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 3:34 | 34 | 17 | 7.5 s | 7.5 s | 9 | 128 | 7 | 36 | 165 (+30 −7) | 3 | 147 | 1951.8 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 3:16 | 28 | 20 | 8.0 s | 7.5 s | 9 | 144 | 8 | 51 | 197 (+39 −7) | 3 | 148 | 2097.1 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:20 | 33 | 16 | 7.5 s | 6.3 s | 9 | 155 | 8 | 61 | 206 (+16 −7) | 3 | 148 | 2254.5 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 2:57 | 28 | 21 | 7.5 s | 7.4 s | 9 | 173 | 8 | 80 | 214 (+15 −7) | 3 | 148 | 2392.9 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 2:50 | 25 | 13 | 7.9 s | 7.5 s | 9 | 192 | 8 | 99 | 224 (+17 −7) | 3 | 148 | 2516.5 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:06 | 26 | 11 | 9.3 s | 6.9 s | 9 | 202 | 8 | 106 | 221 (+4 −7) | 3 | 148 | 2658.1 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 2:58 | 28 | 16 | 7.9 s | 7.5 s | 9 | 217 | 8 | 121 | 267 (+53 −7) | 3 | 148 | 2792.0 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | extracted | 3:11 | 29 | 14 | 7.5 s | 7.5 s | 9 | 231 | 8 | 135 | 294 (+34 −7) | 3 | 148 | 2923.6 | 0 | held | - |
| 19 | 19 | Nia 'Oathless' | extracted | 3:10 | 31 | 17 | 7.5 s | 7.0 s | 9 | 244 | 8 | 148 | 309 (+23 −8) | 3 | 148 | 3063.2 | 0 | held | Lantern Post |
| 20 | 20 | Nia 'Oathless' | extracted | 3:20 | 32 | 17 | 7.5 s | 7.4 s | 9 | 263 | 8 | 167 | 313 (+12 −8) | 3 | 148 | 3209.6 | 0 | held | - |

**Alerts standing at the last shift's end**: the Flood is up 3 tiles; 167 on the Heart's floor: no room in the grid; Sump Row petitions: lit

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
