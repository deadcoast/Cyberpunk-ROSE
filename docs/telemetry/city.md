# The city, twenty shifts headless

Written by `rose city --seed 11 --shifts 20` on 2026-11-11 (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at 2 nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan (pod, lantern, rack, pod, bunkhouse, charter_board, salt_works) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace (3 city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.

**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: all hold.

**The landing cadence** (research 02's whole-game budget of something landing every 10–30 s, the bot's band 3.3–10.0 s at the bot factor 3, an assumption until the Phase F playtest): the run alone lands every 7.9 s (median of the shifts' medians), the run and the city merged every 7.5 s — **ok**. Landings a second apart count as one moment, as the run's report counts them. Deaths: 0 of 20 runs.

| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | Nia 'Oathless' | extracted | 3:31 | 29 | 6 | 8.0 s | 8.0 s | 7 | 28 | 0 | 0 | 0 (+0 −0) | 0 | 208 | 188.1 | 0 | held | Hab-Pod |
| 2 | 2 | Nia 'Oathless' | extracted | 3:17 | 28 | 15 | 7.7 s | 7.5 s | 7 | 41 | 0 | 0 | 82 (+83 −1) | 0 | 214 | 219.9 | 0 | held | Lantern Post |
| 3 | 3 | Nia 'Oathless' | extracted | 3:11 | 27 | 13 | 7.9 s | 7.5 s | 7 | 51 | 0 | 0 | 93 (+12 −1) | 0 | 216 | 251.3 | 0 | held | Stockpile Rack |
| 4 | 4 | Nia 'Oathless' | extracted | 3:14 | 27 | 16 | 7.9 s | 7.5 s | 7 | 56 | 0 | 0 | 130 (+38 −1) | 0 | 220 | 283.2 | 0 | held | Hab-Pod |
| 5 | 5 | Nia 'Oathless' | extracted | 3:25 | 31 | 17 | 7.7 s | 7.3 s | 7 | 66 | 0 | 0 | 142 (+13 −1) | 0 | 224 | 318.0 | 0 | held | - |
| 6 | 6 | Nia 'Oathless' | extracted | 2:07 | 17 | 16 | 10.6 s | 8.6 s | 7 | 71 | 0 | 0 | 156 (+15 −1) | 0 | 226 | 332.6 | 0 | held | Bunkhouse |
| 7 | 7 | Nia 'Oathless' | extracted | 3:47 | 33 | 23 | 7.9 s | 7.5 s | 7 | 77 | 0 | 0 | 174 (+19 −1) | 0 | 228 | 370.0 | 0 | held | Charter Board |
| 8 | 8 | Nia 'Oathless' | extracted | 2:34 | 22 | 22 | 8.2 s | 7.5 s | 9 | 77 | 4 | 0 | 190 (+17 −1) | 0 | 230 | 395.5 | 0 | held | Salt Works |
| 9 | 9 | Nia 'Oathless' | extracted | 2:58 | 26 | 26 | 7.9 s | 7.5 s | 9 | 93 | 7 | 3 | 214 (+25 −1) | 0 | 232 | 424.5 | 0 | held | - |
| 10 | 10 | Nia 'Oathless' | extracted | 3:01 | 25 | 13 | 8.0 s | 7.7 s | 9 | 105 | 8 | 12 | 228 (+15 −1) | 0 | 235 | 453.1 | 0 | held | - |
| 11 | 11 | Nia 'Oathless' | extracted | 3:03 | 28 | 19 | 7.9 s | 7.5 s | 9 | 115 | 7 | 23 | 256 (+29 −1) | 0 | 239 | 481.8 | 0 | held | - |
| 12 | 12 | Nia 'Oathless' | extracted | 2:50 | 23 | 20 | 8.1 s | 7.9 s | 9 | 129 | 8 | 33 | 291 (+36 −1) | 0 | 239 | 500.1 | 0 | held | - |
| 13 | 13 | Nia 'Oathless' | extracted | 3:20 | 31 | 13 | 7.7 s | 7.5 s | 9 | 145 | 8 | 45 | 303 (+13 −1) | 0 | 242 | 530.5 | 0 | held | - |
| 14 | 14 | Nia 'Oathless' | extracted | 2:49 | 25 | 17 | 7.9 s | 7.4 s | 9 | 162 | 8 | 60 | 331 (+29 −1) | 0 | 245 | 556.6 | 0 | held | - |
| 15 | 15 | Nia 'Oathless' | extracted | 3:44 | 31 | 15 | 8.0 s | 7.7 s | 9 | 177 | 8 | 75 | 336 (+6 −1) | 0 | 246 | 590.5 | 0 | held | - |
| 16 | 16 | Nia 'Oathless' | extracted | 3:24 | 27 | 16 | 7.7 s | 7.5 s | 9 | 187 | 8 | 85 | 347 (+12 −1) | 0 | 246 | 621.0 | 0 | held | - |
| 17 | 17 | Nia 'Oathless' | extracted | 3:23 | 31 | 17 | 7.9 s | 7.5 s | 9 | 205 | 8 | 103 | 386 (+40 −1) | 0 | 246 | 650.4 | 0 | held | - |
| 18 | 18 | Nia 'Oathless' | extracted | 3:45 | 35 | 14 | 7.9 s | 7.5 s | 9 | 215 | 8 | 113 | 394 (+9 −1) | 0 | 246 | 671.5 | 0 | held | - |
| 19 | 19 | Nia 'Oathless' | extracted | 2:56 | 24 | 21 | 8.0 s | 7.7 s | 9 | 226 | 8 | 124 | 412 (+19 −1) | 0 | 246 | 697.4 | 0 | held | - |
| 20 | 20 | Nia 'Oathless' | extracted | 3:35 | 31 | 15 | 7.7 s | 7.4 s | 9 | 244 | 8 | 142 | 420 (+9 −1) | 0 | 246 | 727.8 | 0 | held | - |

**Alerts standing at the last shift's end**: the Flood is up 2 tiles; 142 on the Heart's floor: no room in the grid

The merged log, every run and city event on one clock, is `out/city.log` (not committed).
