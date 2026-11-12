# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-12 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 7.3 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:16 | 27 | 9 | 9.7 s | 7.6 s | 7 | 45 | 0 | 0 | 54 (+54 −0) | 0 | 103 | 348.1 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 2:44 | 24 | 15 | 8.0 s | 7.9 s | 7 | 60 | 0 | 0 | 57 (+18 −15) | 7 | 119 | 496.8 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:32 | 31 | 11 | 7.9 s | 7.5 s | 7 | 68 | 0 | 0 | 66 (+24 −15) | 7 | 131 | 686.7 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:10 | 27 | 21 | 7.9 s | 7.5 s | 8 | 72 | 0 | 0 | 72 (+15 −9) | 4 | 135 | 857.3 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 3:01 | 25 | 23 | 8.8 s | 6.9 s | 14 | 75 | 1 | 0 | 58 (+15 −29) | 14 | 148 | 1015.4 | 0 | held | Bunkhouse |
| 6 | 6 | Nia 'Oathless' | extracted | 3:00 | 24 | 21 | 8.8 s | 7.3 s | 14 | 83 | 0 | 0 | 71 (+20 −7) | 3 | 150 | 1161.9 | 0 | held | Charter Board |
| 7 | 7 | Nia 'Oathless' | extracted | 2:45 | 26 | 21 | 7.9 s | 7.0 s | 14 | 85 | 4 | 2 | 79 (+27 −19) | 9 | 156 | 1303.2 | 0 | held | Salt Works |
| 8 | 8 | Nia 'Oathless' | extracted | 3:18 | 31 | 19 | 7.5 s | 6.7 s | 14 | 106 | 7 | 14 | 98 (+24 −5) | 2 | 160 | 1468.6 | 0 | held | - |
| 9 | 9 | Nia 'Oathless' | extracted | 3:29 | 33 | 18 | 7.8 s | 7.5 s | 14 | 116 | 8 | 24 | 91 (+10 −17) | 8 | 183 | 1642.0 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 2:46 | 28 | 15 | 7.7 s | 6.5 s | 14 | 130 | 8 | 37 | 116 (+42 −17) | 8 | 197 | 1779.7 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 3:34 | 34 | 14 | 7.5 s | 7.3 s | 14 | 144 | 8 | 50 | 122 (+25 −19) | 9 | 209 | 1953.1 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 3:16 | 28 | 17 | 8.0 s | 7.5 s | 14 | 158 | 8 | 64 | 123 (+18 −17) | 8 | 217 | 2099.2 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:20 | 33 | 20 | 7.5 s | 6.7 s | 14 | 169 | 8 | 75 | 121 (+15 −17) | 8 | 245 | 2252.0 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 2:57 | 28 | 20 | 7.5 s | 7.4 s | 14 | 187 | 8 | 93 | 122 (+18 −17) | 8 | 255 | 2390.2 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 2:50 | 25 | 10 | 7.9 s | 7.4 s | 14 | 206 | 8 | 112 | 119 (+4 −7) | 3 | 274 | 2513.3 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:06 | 26 | 18 | 9.3 s | 7.5 s | 14 | 218 | 8 | 123 | 146 (+32 −5) | 2 | 285 | 2653.1 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 2:58 | 28 | 15 | 7.9 s | 7.3 s | 14 | 234 | 8 | 139 | 177 (+36 −5) | 2 | 309 | 2785.7 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | extracted | 3:11 | 29 | 13 | 7.5 s | 7.1 s | 14 | 248 | 8 | 153 | 185 (+27 −19) | 9 | 333 | 2914.7 | 0 | held | - |
| 19 | 19 | Nia 'Oathless' | extracted | 3:10 | 31 | 16 | 7.5 s | 6.8 s | 14 | 263 | 8 | 168 | 181 (+13 −17) | 8 | 347 | 3053.4 | 0 | held | - |
| 20 | 20 | Nia 'Oathless' | extracted | 3:20 | 32 | 17 | 7.5 s | 6.6 s | 14 | 282 | 8 | 187 | 192 (+28 −17) | 8 | 359 | 3199.1 | 0 | held | - |

**Alerts standing at the last shift's end**: the Flood is up 3 tiles; 187 on the Heart's floor: no room in the grid; 1 newcomer waits at a grate: a free bed shelters them

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
