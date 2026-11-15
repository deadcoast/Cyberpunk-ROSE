# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-15 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.6 s (median of the shifts' medians), the run and the city merged every 6.3 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 2 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 | Nia 'Oathless' | extracted | 2:50 | 23 | 9 | 11.0 s | 8.5 s | 7 | 43 | 0 | 0 | 54 (+54 −0) | 0 | 250 | 402.5 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | died | 3:49 | 23 | 14 | 8.1 s | 7.5 s | 6 | 25 | 0 | 0 | 72 (+31 −13) | 6 | 304 | 607.5 | 0 | held | Lantern Post |
| 3 | 3 | Moss 'Bleach' | died | 1:54 | 6 | 3 | 8.9 s | 8.9 s | 6 | 20 | 0 | 0 | 71 (+0 −1) | 0 | 308 | 708.9 | 0 | held | Stockpile Rack |
| 4 | 4 | Kerr | extracted | 2:42 | 24 | 5 | 11.4 s | 8.1 s | 8 | 38 | 0 | 0 | 54 (+0 −17) | 8 | 344 | 849.1 | 0 | held | Hab-Pod |
| 5 | 5 | Kerr | extracted | 5:06 | 35 | 14 | 11.3 s | 7.4 s | 14 | 45 | 0 | 0 | 35 (+10 −29) | 14 | 374 | 1113.2 | 0 | held | Bunkhouse |
| 6 | 6 | Kerr | extracted | 3:49 | 34 | 15 | 7.9 s | 7.5 s | 14 | 55 | 0 | 0 | 153 (+145 −27) | 13 | 431 | 1300.9 | 0 | held | Charter Board |
| 7 | 7 | Kerr | extracted | 3:27 | 30 | 15 | 7.7 s | 6.3 s | 14 | 58 | 0 | 0 | 139 (+5 −19) | 9 | 481 | 1479.6 | 0 | held | Salt Works |
| 8 | 8 | Kerr | extracted | 3:52 | 36 | 14 | 8.0 s | 7.5 s | 14 | 77 | 0 | 0 | 167 (+29 −1) | 0 | 548 | 1676.2 | 0 | held | - |
| 9 | 9 | Kerr | extracted | 3:14 | 35 | 16 | 7.5 s | 5.7 s | 14 | 85 | 0 | 6 | 163 (+5 −9) | 4 | 628 | 1839.2 | 0 | held | - |
| 10 | 10 | Kerr | extracted | 3:24 | 36 | 30 | 7.9 s | 5.8 s | 14 | 104 | 5 | 18 | 195 (+33 −1) | 0 | 678 | 2007.4 | 0 | held | - |
| 11 | 11 | Kerr | extracted | 3:17 | 30 | 18 | 7.5 s | 6.2 s | 14 | 121 | 7 | 34 | 226 (+32 −1) | 0 | 716 | 2161.7 | 0 | held | - |
| 12 | 12 | Kerr | extracted | 2:35 | 27 | 22 | 7.0 s | 6.2 s | 14 | 130 | 8 | 42 | 253 (+28 −1) | 0 | 754 | 2276.1 | 0 | held | - |
| 13 | 13 | Kerr | extracted | 3:46 | 36 | 14 | 7.5 s | 7.4 s | 14 | 152 | 8 | 62 | 263 (+11 −1) | 0 | 790 | 2449.3 | 0 | held | - |
| 14 | 14 | Kerr | extracted | 3:36 | 36 | 25 | 7.4 s | 6.2 s | 14 | 162 | 8 | 73 | 281 (+21 −3) | 1 | 857 | 2618.6 | 0 | held | - |
| 15 | 15 | Kerr | extracted | 3:21 | 34 | 17 | 7.4 s | 7.2 s | 14 | 179 | 8 | 89 | 317 (+37 −1) | 0 | 900 | 2768.6 | 0 | held | - |
| 16 | 16 | Kerr | extracted | 3:30 | 34 | 18 | 7.5 s | 6.3 s | 14 | 195 | 8 | 107 | 331 (+15 −1) | 0 | 960 | 2923.7 | 0 | held | - |
| 17 | 17 | Kerr | extracted | 2:31 | 24 | 21 | 7.4 s | 5.7 s | 14 | 199 | 8 | 110 | 360 (+30 −1) | 0 | 1031 | 3036.1 | 0 | held | - |
| 18 | 18 | Kerr | extracted | 2:56 | 29 | 13 | 7.4 s | 6.1 s | 14 | 219 | 8 | 129 | 356 (+13 −17) | 8 | 1072 | 3161.9 | 0 | held | - |
| 19 | 19 | Kerr | extracted | 2:46 | 29 | 14 | 7.9 s | 6.0 s | 14 | 225 | 8 | 134 | 368 (+13 −1) | 0 | 1151 | 3285.1 | 0 | held | - |
| 20 | 20 | Kerr | extracted | 2:06 | 25 | 13 | 7.0 s | 5.8 s | 14 | 239 | 8 | 149 | 386 (+23 −5) | 2 | 1204 | 3374.5 | 0 | held | - |

**Alerts standing at the last shift's end**: 149 on the Heart's floor: no room in the grid

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
