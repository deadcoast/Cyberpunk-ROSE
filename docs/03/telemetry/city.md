# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-13 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 6.6 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:16 | 27 | 9 | 9.7 s | 7.6 s | 7 | 50 | 0 | 0 | 50 (+54 −4) | 2 | 256 | 425.4 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 2:44 | 24 | 15 | 8.0 s | 5.7 s | 7 | 62 | 0 | 0 | 53 (+18 −15) | 7 | 321 | 576.7 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:32 | 31 | 13 | 7.9 s | 7.5 s | 7 | 76 | 0 | 0 | 62 (+24 −15) | 7 | 363 | 765.4 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:10 | 27 | 21 | 7.9 s | 6.2 s | 8 | 80 | 0 | 0 | 60 (+15 −17) | 8 | 423 | 940.3 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 3:01 | 25 | 21 | 8.8 s | 6.1 s | 11 | 82 | 1 | 0 | 60 (+15 −15) | 7 | 488 | 1103.8 | 0 | held | Bunkhouse |
| 6 | 6 | Nia 'Oathless' | extracted | 3:00 | 24 | 23 | 8.8 s | 6.5 s | 14 | 83 | 0 | 0 | 71 (+20 −9) | 4 | 553 | 1253.1 | 0 | held | Charter Board |
| 7 | 7 | Nia 'Oathless' | extracted | 2:45 | 26 | 23 | 7.9 s | 6.4 s | 14 | 90 | 5 | 4 | 97 (+27 −1) | 0 | 594 | 1393.9 | 0 | held | Salt Works |
| 8 | 8 | Nia 'Oathless' | extracted | 3:18 | 31 | 19 | 7.5 s | 7.1 s | 14 | 111 | 6 | 21 | 120 (+24 −1) | 0 | 660 | 1561.5 | 0 | held | - |
| 9 | 9 | Nia 'Oathless' | extracted | 3:29 | 33 | 20 | 7.8 s | 7.2 s | 14 | 119 | 8 | 28 | 129 (+10 −1) | 0 | 715 | 1735.3 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 2:46 | 28 | 16 | 7.7 s | 7.4 s | 14 | 131 | 8 | 37 | 176 (+48 −1) | 0 | 751 | 1870.8 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 3:34 | 34 | 16 | 7.5 s | 7.4 s | 14 | 149 | 8 | 56 | 200 (+25 −1) | 0 | 786 | 2041.0 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 3:16 | 28 | 19 | 8.0 s | 6.6 s | 14 | 163 | 8 | 70 | 217 (+18 −1) | 0 | 840 | 2189.6 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:20 | 33 | 22 | 7.5 s | 6.3 s | 14 | 177 | 8 | 82 | 231 (+15 −1) | 0 | 897 | 2348.4 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 2:57 | 28 | 21 | 7.5 s | 6.8 s | 14 | 181 | 8 | 87 | 256 (+26 −1) | 0 | 965 | 2488.9 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 2:50 | 25 | 13 | 7.9 s | 5.3 s | 14 | 199 | 8 | 103 | 281 (+26 −1) | 0 | 1026 | 2618.1 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:06 | 26 | 19 | 9.3 s | 7.4 s | 14 | 211 | 8 | 115 | 306 (+26 −1) | 0 | 1098 | 2758.4 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 2:58 | 28 | 16 | 7.9 s | 7.3 s | 14 | 227 | 8 | 132 | 339 (+34 −1) | 0 | 1145 | 2887.9 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | extracted | 3:11 | 29 | 15 | 7.5 s | 6.5 s | 14 | 243 | 8 | 147 | 373 (+35 −1) | 0 | 1185 | 3024.5 | 0 | held | - |
| 19 | 19 | Nia 'Oathless' | extracted | 3:10 | 31 | 22 | 7.5 s | 6.7 s | 14 | 255 | 8 | 160 | 405 (+13 −1) | 0 | 1256 | 3160.3 | 0 | held | - |
| 20 | 20 | Nia 'Oathless' | extracted | 3:20 | 32 | 18 | 7.5 s | 5.7 s | 14 | 271 | 8 | 175 | 433 (+29 −1) | 0 | 1333 | 3302.8 | 0 | held | - |

**Alerts standing at the last shift's end**: 175 on the Heart's floor: no room in the grid; 4 newcomers wait at a grate: a free bed shelters them

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
