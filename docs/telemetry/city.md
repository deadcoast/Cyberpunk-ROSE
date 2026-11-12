# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-12 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 8.0 s (median of the shifts' medians), the run and the city merged every 7.5 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 1 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:16 | 27 | 6 | 9.7 s | 9.7 s | 7 | 27 | 0 | 0 | 0 (+0 −0) | 0 | 101 | 348.1 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 2:44 | 24 | 13 | 8.0 s | 7.7 s | 7 | 41 | 0 | 0 | 82 (+83 −1) | 0 | 113 | 496.0 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:32 | 31 | 11 | 7.9 s | 7.5 s | 7 | 49 | 0 | 0 | 100 (+19 −1) | 0 | 126 | 685.7 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:10 | 27 | 17 | 7.9 s | 7.5 s | 7 | 55 | 0 | 0 | 123 (+24 −1) | 0 | 126 | 856.7 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 2:57 | 25 | 20 | 8.8 s | 7.5 s | 7 | 58 | 0 | 0 | 137 (+15 −1) | 0 | 127 | 1011.4 | 0 | held | - |
| 6 | 6 | Nia 'Oathless' | extracted | 2:56 | 27 | 17 | 7.7 s | 7.5 s | 7 | 71 | 0 | 0 | 151 (+15 −1) | 0 | 137 | 1155.7 | 0 | held | Bunkhouse |
| 7 | 7 | Nia 'Oathless' | extracted | 3:19 | 29 | 17 | 7.7 s | 7.5 s | 7 | 76 | 0 | 0 | 173 (+23 −1) | 0 | 140 | 1323.3 | 0 | held | Charter Board |
| 8 | 8 | Nia 'Oathless' | extracted | 3:10 | 28 | 23 | 8.0 s | 7.5 s | 9 | 84 | 3 | 0 | 197 (+25 −1) | 0 | 145 | 1482.6 | 0 | held | Salt Works |
| 9 | 9 | Nia 'Oathless' | extracted | 2:34 | 20 | 16 | 9.7 s | 7.5 s | 9 | 92 | 5 | 5 | 206 (+10 −1) | 0 | 145 | 1612.3 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 3:39 | 34 | 17 | 7.9 s | 7.5 s | 9 | 113 | 7 | 21 | 228 (+23 −1) | 0 | 152 | 1792.3 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 3:26 | 31 | 18 | 7.7 s | 7.5 s | 9 | 132 | 7 | 40 | 232 (+5 −1) | 0 | 153 | 1958.6 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 2:59 | 25 | 17 | 8.0 s | 7.5 s | 9 | 148 | 8 | 54 | 255 (+24 −1) | 0 | 153 | 2091.7 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:11 | 27 | 18 | 7.9 s | 7.5 s | 9 | 162 | 8 | 68 | 269 (+15 −1) | 0 | 153 | 2241.1 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 2:46 | 22 | 15 | 8.1 s | 7.9 s | 9 | 167 | 8 | 74 | 288 (+20 −1) | 0 | 153 | 2371.8 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 2:32 | 21 | 22 | 9.8 s | 7.8 s | 9 | 183 | 8 | 90 | 329 (+42 −1) | 0 | 154 | 2482.7 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 2:09 | 19 | 17 | 8.1 s | 7.5 s | 9 | 194 | 8 | 99 | 347 (+19 −1) | 0 | 154 | 2582.5 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 3:35 | 33 | 11 | 7.8 s | 7.5 s | 9 | 209 | 8 | 112 | 385 (+39 −1) | 0 | 154 | 2743.8 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | died | 6:43 | 46 | 21 | 8.1 s | 7.9 s | 8 | 199 | 8 | 103 | 426 (+42 −1) | 0 | 158 | 3019.9 | 0 | held | - |
| 19 | 19 | Moss 'Bleach' | extracted | 3:12 | 27 | 11 | 7.9 s | 7.5 s | 9 | 216 | 8 | 120 | 453 (+29 −2) | 0 | 158 | 3159.5 | 0 | held | Lantern Post |
| 20 | 20 | Moss 'Bleach' | extracted | 3:28 | 29 | 8 | 9.0 s | 7.7 s | 9 | 229 | 8 | 133 | 451 (+0 −2) | 0 | 161 | 3311.0 | 0 | held | - |

**Alerts standing at the last shift's end**: the Flood is up 3 tiles; 133 on the Heart's floor: no room in the grid; Sump Row petitions: lit; grate 5 is loud: a Patrol is coming

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
