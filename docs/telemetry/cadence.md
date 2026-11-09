# Reward cadence — bot runs against the budget

Written by `rose cadence --runs 20 --strata 1,2,3 --seed 1` on 2026-11-09. The bot (`run::bot`) opens what it sees, takes what fits, fights adjacent threats over 40 % HP, digs deposits it stands beside, extracts when the rig is nearly full, the shift is under 90 s or the body is under 30 %; it walks the lit route and never waits or reads. Player budget (Full Design, The cadence budget): pull gap 60–120 s median, landing gap 10–30 s median, longest quiet between losses under 120 s, death rate 10–30 %. The bot is taken as 3× faster than a cautious player (an assumption until a recorded human run calibrates it), so its bands are pull 20–40 s, landing 3–10 s, quiet under 40 s; the death band is the player's. The run is verdicted on pulls, pressure and deaths; the landing budget is the whole game's (run and colony together) and is judged on the combined log from Phase C, so the landing gap is reported and not verdicted.

## Aggregate

| stratum | runs | deaths | run length (median) | pull gap median | landing gap median | pulls / run | notables / run | kills / run | damage / run | longest quiet (median) | blind e / min | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| S1 | 20 | 6 (30 %) | 4:58 | 11 s | 11 s | 14.7 | 0.00 | 0.8 | 33 | 70 s | 8.3 | pull fast · loss lapses · death ok |
| S2 | 20 | 6 (30 %) | 3:37 | 12 s | 12 s | 11.4 | 0.00 | 1.3 | 37 | 42 s | 17.0 | pull fast · loss lapses · death ok |
| S3 | 20 | 4 (20 %) | 4:06 | 12 s | 11 s | 12.4 | 2.95 | 1.5 | 46 | 60 s | 26.3 | pull fast · loss lapses · death ok |

## Every run

| stratum | seed | result | length | pulls | landings | notables | kills | damage | pull gap med | landing gap med | quiet max | blind e |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| S1 | 1 | died | 40:00.0 | 14 | 30 | 0 | 3 | 30 | 11 s | 8 s | 557 s | 27 |
| S1 | 2 | extracted | 04:48.5 | 16 | 33 | 0 | 0 | 18 | 11 s | 12 s | 37 s | 46 |
| S1 | 3 | extracted | 03:59.9 | 17 | 35 | 0 | 0 | 18 | 8 s | 8 s | 3 s | 55 |
| S1 | 4 | died | 40:00.0 | 13 | 28 | 0 | 1 | 16 | 8 s | 8 s | 792 s | 32 |
| S1 | 5 | extracted | 05:09.8 | 17 | 34 | 0 | 0 | 45 | 12 s | 12 s | 79 s | 34 |
| S1 | 6 | extracted | 04:22.8 | 16 | 33 | 0 | 0 | 12 | 8 s | 10 s | 3 s | 30 |
| S1 | 7 | extracted | 05:50.5 | 17 | 35 | 0 | 1 | 49 | 12 s | 12 s | 78 s | 48 |
| S1 | 8 | died | 03:35.9 | 8 | 20 | 0 | 3 | 79 | 12 s | 12 s | 41 s | 16 |
| S1 | 9 | extracted | 04:16.5 | 12 | 27 | 0 | 2 | 39 | 11 s | 8 s | 88 s | 29 |
| S1 | 10 | extracted | 04:39.5 | 16 | 33 | 0 | 0 | 24 | 11 s | 11 s | 86 s | 44 |
| S1 | 11 | died | 40:00.0 | 18 | 36 | 0 | 0 | 27 | 9 s | 9 s | 472 s | 33 |
| S1 | 12 | extracted | 04:01.0 | 14 | 29 | 0 | 0 | 30 | 11 s | 11 s | 38 s | 44 |
| S1 | 13 | extracted | 05:39.1 | 15 | 31 | 0 | 0 | 51 | 13 s | 14 s | 92 s | 54 |
| S1 | 14 | extracted | 02:04.7 | 10 | 21 | 0 | 0 | 0 | 8 s | 8 s | 0 s | 52 |
| S1 | 15 | extracted | 05:08.3 | 14 | 31 | 0 | 1 | 55 | 14 s | 14 s | 51 s | 48 |
| S1 | 16 | died | 05:24.4 | 15 | 30 | 0 | 1 | 77 | 14 s | 11 s | 110 s | 32 |
| S1 | 17 | died | 40:00.0 | 16 | 34 | 0 | 2 | 28 | 8 s | 8 s | 615 s | 37 |
| S1 | 18 | extracted | 04:24.8 | 16 | 34 | 0 | 1 | 16 | 11 s | 11 s | 2 s | 51 |
| S1 | 19 | extracted | 02:48.3 | 13 | 26 | 0 | 0 | 0 | 8 s | 8 s | 0 s | 55 |
| S1 | 20 | extracted | 06:56.3 | 17 | 34 | 0 | 0 | 51 | 12 s | 12 s | 61 s | 40 |
| S2 | 1 | extracted | 04:58.8 | 16 | 35 | 0 | 3 | 30 | 14 s | 12 s | 106 s | 85 |
| S2 | 2 | extracted | 04:28.9 | 15 | 33 | 0 | 1 | 17 | 13 s | 13 s | 0 s | 80 |
| S2 | 3 | extracted | 02:58.6 | 11 | 23 | 0 | 0 | 12 | 12 s | 12 s | 0 s | 96 |
| S2 | 4 | died | 01:48.7 | 4 | 11 | 0 | 3 | 37 | 9 s | 11 s | 61 s | 33 |
| S2 | 5 | extracted | 03:49.4 | 13 | 27 | 0 | 0 | 18 | 12 s | 13 s | 44 s | 75 |
| S2 | 6 | died | 16:42.2 | 15 | 31 | 0 | 1 | 104 | 13 s | 13 s | 571 s | 75 |
| S2 | 7 | died | 01:56.2 | 5 | 10 | 0 | 0 | 32 | 11 s | 11 s | 60 s | 31 |
| S2 | 8 | extracted | 02:58.6 | 10 | 24 | 0 | 2 | 64 | 12 s | 9 s | 22 s | 81 |
| S2 | 9 | extracted | 03:15.4 | 12 | 26 | 0 | 1 | 22 | 12 s | 12 s | 73 s | 74 |
| S2 | 10 | extracted | 11:14.8 | 14 | 29 | 0 | 0 | 46 | 14 s | 14 s | 330 s | 69 |
| S2 | 11 | extracted | 03:56.1 | 13 | 34 | 0 | 5 | 68 | 13 s | 9 s | 77 s | 60 |
| S2 | 12 | died | 02:43.7 | 8 | 16 | 0 | 0 | 92 | 13 s | 13 s | 60 s | 24 |
| S2 | 13 | extracted | 03:37.1 | 12 | 26 | 0 | 1 | 10 | 12 s | 12 s | 0 s | 82 |
| S2 | 14 | extracted | 03:01.6 | 11 | 24 | 0 | 1 | 16 | 12 s | 12 s | 3 s | 71 |
| S2 | 15 | extracted | 04:19.0 | 14 | 28 | 0 | 0 | 12 | 13 s | 13 s | 2 s | 77 |
| S2 | 16 | extracted | 03:38.9 | 13 | 27 | 0 | 1 | 28 | 11 s | 11 s | 21 s | 77 |
| S2 | 17 | extracted | 03:15.2 | 12 | 27 | 0 | 2 | 26 | 12 s | 11 s | 2 s | 61 |
| S2 | 18 | extracted | 03:36.1 | 12 | 27 | 0 | 2 | 26 | 12 s | 12 s | 40 s | 81 |
| S2 | 19 | died | 02:50.2 | 8 | 15 | 0 | 1 | 75 | 13 s | 11 s | 10 s | 21 |
| S2 | 20 | died | 40:00.0 | 10 | 23 | 0 | 2 | 10 | 12 s | 9 s | 681 s | 38 |
| S3 | 1 | extracted | 12:44.2 | 13 | 34 | 3 | 4 | 46 | 11 s | 10 s | 557 s | 79 |
| S3 | 2 | extracted | 03:06.4 | 11 | 28 | 3 | 1 | 28 | 12 s | 11 s | 35 s | 96 |
| S3 | 3 | extracted | 03:44.3 | 12 | 31 | 4 | 2 | 47 | 12 s | 12 s | 36 s | 96 |
| S3 | 4 | extracted | 03:53.9 | 12 | 33 | 5 | 3 | 36 | 14 s | 12 s | 74 s | 182 |
| S3 | 5 | extracted | 02:08.9 | 10 | 24 | 3 | 0 | 6 | 8 s | 9 s | 0 s | 55 |
| S3 | 6 | died | 05:28.0 | 16 | 38 | 2 | 3 | 87 | 12 s | 12 s | 132 s | 64 |
| S3 | 7 | extracted | 04:23.8 | 14 | 33 | 3 | 1 | 75 | 11 s | 11 s | 36 s | 96 |
| S3 | 8 | extracted | 04:35.5 | 14 | 31 | 2 | 1 | 23 | 12 s | 12 s | 141 s | 73 |
| S3 | 9 | extracted | 10:52.6 | 9 | 20 | 2 | 0 | 6 | 10 s | 11 s | 0 s | 151 |
| S3 | 10 | extracted | 03:53.4 | 12 | 31 | 2 | 2 | 56 | 12 s | 12 s | 66 s | 151 |
| S3 | 11 | extracted | 04:18.6 | 17 | 38 | 2 | 2 | 38 | 11 s | 11 s | 61 s | 98 |
| S3 | 12 | extracted | 03:41.9 | 13 | 31 | 4 | 1 | 68 | 11 s | 12 s | 21 s | 96 |
| S3 | 13 | extracted | 12:21.8 | 10 | 24 | 3 | 1 | 58 | 12 s | 14 s | 498 s | 77 |
| S3 | 14 | died | 03:36.6 | 10 | 23 | 1 | 2 | 80 | 13 s | 12 s | 60 s | 34 |
| S3 | 15 | extracted | 03:10.0 | 11 | 24 | 2 | 0 | 12 | 10 s | 11 s | 1 s | 70 |
| S3 | 16 | extracted | 06:07.0 | 19 | 43 | 5 | 1 | 49 | 12 s | 12 s | 64 s | 186 |
| S3 | 17 | died | 05:08.0 | 12 | 30 | 3 | 3 | 95 | 14 s | 10 s | 84 s | 265 |
| S3 | 18 | extracted | 02:14.4 | 8 | 21 | 4 | 1 | 16 | 12 s | 11 s | 3 s | 134 |
| S3 | 19 | died | 04:17.6 | 15 | 32 | 3 | 0 | 82 | 12 s | 11 s | 60 s | 92 |
| S3 | 20 | extracted | 03:05.4 | 11 | 27 | 3 | 1 | 22 | 12 s | 10 s | 34 s | 118 |
