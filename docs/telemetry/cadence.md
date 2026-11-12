# Reward cadence — bot runs against the budget

Written by `rose cadence --runs 20 --strata 1,2,3 --seed 1` on 2026-11-12. The bot (`run::bot`) opens what it sees, takes what fits, fights adjacent threats over 40 % HP, digs deposits it stands beside, extracts when the rig is nearly full, the shift is under 90 s or the body is under 30 %; it walks the lit route and never waits or reads. Player budget (Full Design, The cadence budget): pull gap 60–120 s median, landing gap 10–30 s median, longest quiet between losses under 120 s, death rate 10–30 %. The bot is taken as 3× faster than a cautious player (an assumption until a recorded human run calibrates it), so its bands are pull 20–40 s, landing 3–10 s, quiet under 40 s; the death band is the player's. The run is verdicted on pulls, pressure and deaths; the landing budget is the whole game's (run and colony together) and is judged on the combined log from Phase C, so the landing gap is reported and not verdicted.

## Aggregate

| stratum | runs | deaths | run length (median) | pull gap median | landing gap median | pulls / run | notables / run | kills / run | damage / run | longest quiet (median) | blind e / min | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| S1 | 20 | 3 (15 %) | 4:19 | 10 s | 10 s | 15.2 | 0.00 | 0.8 | 40 | 46 s | 9.5 | pull fast · loss lapses · death ok |
| S2 | 20 | 4 (20 %) | 3:44 | 14 s | 12 s | 12.7 | 0.00 | 1.1 | 43 | 8 s | 30.3 | pull fast · loss ok · death ok |
| S3 | 20 | 5 (25 %) | 3:49 | 12 s | 11 s | 12.6 | 2.80 | 1.6 | 49 | 62 s | 28.7 | pull fast · loss lapses · death ok |

## Every run

| stratum | seed | result | length | pulls | landings | notables | kills | damage | pull gap med | landing gap med | quiet max | blind e |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| S1 | 1 | extracted | 04:17.2 | 16 | 36 | 0 | 3 | 30 | 11 s | 8 s | 39 s | 37 |
| S1 | 2 | extracted | 04:38.8 | 15 | 31 | 0 | 0 | 18 | 10 s | 11 s | 37 s | 57 |
| S1 | 3 | extracted | 04:39.3 | 16 | 32 | 0 | 0 | 12 | 11 s | 11 s | 0 s | 52 |
| S1 | 4 | died | 03:55.5 | 13 | 29 | 0 | 2 | 26 | 8 s | 9 s | 60 s | 28 |
| S1 | 5 | died | 04:15.5 | 15 | 29 | 0 | 0 | 65 | 12 s | 12 s | 52 s | 32 |
| S1 | 6 | died | 04:47.4 | 16 | 33 | 0 | 1 | 111 | 8 s | 8 s | 24 s | 41 |
| S1 | 7 | extracted | 03:50.9 | 15 | 30 | 0 | 0 | 18 | 8 s | 8 s | 0 s | 43 |
| S1 | 8 | extracted | 04:24.2 | 15 | 32 | 0 | 1 | 69 | 12 s | 11 s | 28 s | 38 |
| S1 | 9 | extracted | 04:00.4 | 12 | 27 | 0 | 2 | 32 | 12 s | 12 s | 94 s | 36 |
| S1 | 10 | extracted | 05:03.7 | 18 | 37 | 0 | 1 | 57 | 11 s | 11 s | 78 s | 44 |
| S1 | 11 | extracted | 04:21.9 | 14 | 29 | 0 | 0 | 40 | 12 s | 12 s | 25 s | 23 |
| S1 | 12 | extracted | 04:35.5 | 16 | 33 | 0 | 0 | 45 | 11 s | 12 s | 45 s | 34 |
| S1 | 13 | extracted | 03:58.6 | 13 | 27 | 0 | 0 | 36 | 10 s | 11 s | 47 s | 36 |
| S1 | 14 | extracted | 03:52.6 | 15 | 33 | 0 | 1 | 16 | 9 s | 9 s | 3 s | 39 |
| S1 | 15 | extracted | 03:32.4 | 14 | 29 | 0 | 1 | 49 | 8 s | 8 s | 56 s | 43 |
| S1 | 16 | extracted | 03:03.6 | 15 | 29 | 0 | 0 | 0 | 8 s | 8 s | 0 s | 45 |
| S1 | 17 | extracted | 06:19.5 | 18 | 37 | 0 | 2 | 71 | 11 s | 11 s | 72 s | 45 |
| S1 | 18 | extracted | 04:14.6 | 15 | 33 | 0 | 1 | 50 | 8 s | 8 s | 77 s | 44 |
| S1 | 19 | extracted | 04:19.9 | 17 | 33 | 0 | 1 | 28 | 8 s | 8 s | 134 s | 55 |
| S1 | 20 | extracted | 04:20.4 | 16 | 33 | 0 | 0 | 24 | 8 s | 8 s | 71 s | 36 |
| S2 | 1 | died | 04:13.4 | 14 | 31 | 0 | 4 | 127 | 14 s | 9 s | 30 s | 64 |
| S2 | 2 | died | 03:47.2 | 12 | 26 | 0 | 1 | 127 | 13 s | 12 s | 44 s | 90 |
| S2 | 3 | extracted | 03:20.9 | 12 | 25 | 0 | 0 | 12 | 12 s | 13 s | 2 s | 88 |
| S2 | 4 | extracted | 02:14.9 | 9 | 22 | 0 | 2 | 20 | 10 s | 9 s | 1 s | 82 |
| S2 | 5 | extracted | 02:10.9 | 7 | 19 | 0 | 2 | 58 | 14 s | 9 s | 15 s | 315 |
| S2 | 6 | extracted | 05:54.5 | 18 | 36 | 0 | 0 | 12 | 14 s | 14 s | 3 s | 84 |
| S2 | 7 | died | 03:09.2 | 11 | 23 | 0 | 1 | 80 | 11 s | 12 s | 60 s | 63 |
| S2 | 8 | extracted | 04:04.1 | 13 | 29 | 0 | 1 | 16 | 14 s | 14 s | 0 s | 70 |
| S2 | 9 | extracted | 02:59.5 | 10 | 21 | 0 | 0 | 6 | 14 s | 14 s | 0 s | 73 |
| S2 | 10 | extracted | 05:17.8 | 15 | 31 | 0 | 0 | 27 | 15 s | 15 s | 4 s | 97 |
| S2 | 11 | extracted | 05:31.0 | 15 | 31 | 0 | 0 | 43 | 14 s | 14 s | 33 s | 76 |
| S2 | 12 | extracted | 05:26.9 | 17 | 35 | 0 | 0 | 18 | 14 s | 14 s | 6 s | 48 |
| S2 | 13 | extracted | 03:06.9 | 12 | 29 | 0 | 3 | 42 | 12 s | 9 s | 5 s | 59 |
| S2 | 14 | extracted | 02:47.2 | 10 | 22 | 0 | 1 | 17 | 14 s | 9 s | 1 s | 79 |
| S2 | 15 | extracted | 02:49.6 | 10 | 24 | 0 | 2 | 33 | 12 s | 11 s | 11 s | 292 |
| S2 | 16 | extracted | 03:27.6 | 13 | 27 | 0 | 1 | 62 | 10 s | 9 s | 29 s | 88 |
| S2 | 17 | died | 04:52.2 | 15 | 34 | 0 | 3 | 76 | 14 s | 12 s | 113 s | 48 |
| S2 | 18 | extracted | 03:40.1 | 12 | 27 | 0 | 1 | 14 | 14 s | 13 s | 17 s | 79 |
| S2 | 19 | extracted | 04:08.6 | 14 | 27 | 0 | 0 | 60 | 13 s | 13 s | 4 s | 101 |
| S2 | 20 | extracted | 04:35.7 | 15 | 33 | 0 | 1 | 16 | 13 s | 13 s | 14 s | 56 |
| S3 | 1 | died | 05:28.0 | 12 | 29 | 1 | 4 | 93 | 12 s | 10 s | 76 s | 54 |
| S3 | 2 | extracted | 03:06.4 | 11 | 28 | 3 | 1 | 28 | 12 s | 11 s | 35 s | 96 |
| S3 | 3 | extracted | 03:44.3 | 12 | 31 | 4 | 2 | 47 | 12 s | 12 s | 36 s | 96 |
| S3 | 4 | extracted | 03:53.9 | 12 | 33 | 5 | 3 | 36 | 14 s | 12 s | 74 s | 182 |
| S3 | 5 | extracted | 02:08.9 | 10 | 24 | 3 | 0 | 6 | 8 s | 9 s | 0 s | 55 |
| S3 | 6 | died | 05:28.0 | 16 | 38 | 2 | 3 | 87 | 12 s | 12 s | 132 s | 64 |
| S3 | 7 | extracted | 04:23.8 | 14 | 33 | 3 | 1 | 75 | 11 s | 11 s | 36 s | 96 |
| S3 | 8 | extracted | 04:35.5 | 14 | 31 | 2 | 1 | 23 | 12 s | 12 s | 141 s | 73 |
| S3 | 9 | extracted | 03:44.5 | 13 | 30 | 2 | 2 | 27 | 10 s | 10 s | 68 s | 167 |
| S3 | 10 | extracted | 03:53.4 | 12 | 31 | 2 | 2 | 56 | 12 s | 12 s | 66 s | 151 |
| S3 | 11 | extracted | 03:52.8 | 15 | 35 | 1 | 3 | 52 | 11 s | 11 s | 35 s | 89 |
| S3 | 12 | extracted | 03:41.9 | 13 | 31 | 4 | 1 | 68 | 11 s | 12 s | 21 s | 96 |
| S3 | 13 | extracted | 03:45.9 | 11 | 28 | 3 | 2 | 20 | 14 s | 14 s | 74 s | 80 |
| S3 | 14 | died | 03:36.6 | 10 | 23 | 1 | 2 | 80 | 13 s | 12 s | 60 s | 34 |
| S3 | 15 | extracted | 03:10.0 | 11 | 24 | 2 | 0 | 12 | 10 s | 11 s | 1 s | 70 |
| S3 | 16 | extracted | 06:07.0 | 19 | 43 | 5 | 1 | 49 | 12 s | 12 s | 64 s | 186 |
| S3 | 17 | died | 05:08.0 | 12 | 30 | 3 | 3 | 95 | 14 s | 10 s | 84 s | 265 |
| S3 | 18 | extracted | 02:14.4 | 8 | 21 | 4 | 1 | 16 | 12 s | 11 s | 76 s | 134 |
| S3 | 19 | died | 04:17.6 | 15 | 32 | 3 | 0 | 82 | 12 s | 11 s | 60 s | 92 |
| S3 | 20 | extracted | 03:05.4 | 11 | 27 | 3 | 1 | 22 | 12 s | 10 s | 34 s | 118 |
