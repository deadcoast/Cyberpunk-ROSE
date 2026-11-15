# Reward cadence — bot runs against the budget

Written by `rose cadence --runs 20 --strata 1,2,3 --seed 1` on 2026-11-15. The bot (`run::bot`) opens what it sees, takes what fits, fights adjacent threats over 40 % HP, digs deposits it stands beside, extracts when the rig is nearly full, the shift is under 90 s or the body is under 30 %; it walks the lit route and never waits or reads. Player budget (Full Design, The cadence budget): pull gap 60–120 s median, landing gap 10–30 s median, longest quiet between losses under 120 s, death rate 10–30 %. The bot is taken as 3× faster than a cautious player (an assumption until a recorded human run calibrates it), so its bands are pull 20–40 s, landing 3–10 s, quiet under 40 s; the death band is the player's. The run is verdicted on pulls, pressure and deaths; the landing budget is the whole game's (run and colony together) and is judged on the combined log from Phase C, so the landing gap is reported and not verdicted.

## Aggregate

| stratum | runs | deaths | run length (median) | pull gap median | landing gap median | pulls / run | notables / run | kills / run | damage / run | longest quiet (median) | blind e / min | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| S1 | 20 | 6 (30 %) | 4:13 | 10 s | 10 s | 13.5 | 0.50 | 1.0 | 58 | 40 s | 10.8 | pull fast · loss ok · death ok |
| S2 | 20 | 3 (15 %) | 3:44 | 13 s | 13 s | 12.0 | 0.65 | 1.2 | 45 | 40 s | 30.2 | pull fast · loss ok · death ok |
| S3 | 20 | 4 (20 %) | 3:48 | 12 s | 11 s | 11.6 | 2.70 | 1.4 | 57 | 40 s | 28.5 | pull fast · loss ok · death ok |

## Every run

| stratum | seed | result | length | pulls | landings | notables | kills | damage | pull gap med | landing gap med | quiet max | blind e |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| S1 | 1 | extracted | 04:03.1 | 14 | 33 | 1 | 3 | 43 | 11 s | 8 s | 40 s | 47 |
| S1 | 2 | extracted | 04:26.1 | 14 | 29 | 1 | 0 | 56 | 11 s | 12 s | 40 s | 70 |
| S1 | 3 | extracted | 04:15.1 | 14 | 28 | 0 | 0 | 27 | 11 s | 12 s | 40 s | 55 |
| S1 | 4 | extracted | 03:16.6 | 12 | 28 | 1 | 1 | 49 | 8 s | 8 s | 40 s | 43 |
| S1 | 5 | died | 04:24.0 | 15 | 30 | 1 | 0 | 56 | 12 s | 12 s | 60 s | 49 |
| S1 | 6 | died | 05:28.0 | 16 | 34 | 1 | 1 | 122 | 8 s | 8 s | 80 s | 53 |
| S1 | 7 | extracted | 03:53.2 | 15 | 34 | 0 | 4 | 45 | 8 s | 10 s | 40 s | 49 |
| S1 | 8 | died | 03:31.9 | 9 | 19 | 0 | 1 | 67 | 12 s | 9 s | 40 s | 14 |
| S1 | 9 | extracted | 03:52.6 | 11 | 25 | 0 | 2 | 59 | 10 s | 10 s | 40 s | 43 |
| S1 | 10 | died | 04:15.9 | 12 | 25 | 1 | 0 | 73 | 11 s | 11 s | 40 s | 53 |
| S1 | 11 | extracted | 04:35.4 | 14 | 29 | 0 | 0 | 67 | 12 s | 12 s | 40 s | 28 |
| S1 | 12 | extracted | 04:34.4 | 17 | 36 | 1 | 0 | 57 | 10 s | 11 s | 40 s | 59 |
| S1 | 13 | died | 04:40.0 | 13 | 26 | 0 | 0 | 84 | 10 s | 10 s | 44 s | 28 |
| S1 | 14 | extracted | 04:36.7 | 16 | 35 | 0 | 1 | 55 | 11 s | 11 s | 40 s | 39 |
| S1 | 15 | died | 03:23.9 | 9 | 19 | 0 | 1 | 62 | 10 s | 10 s | 60 s | 18 |
| S1 | 16 | extracted | 03:19.8 | 16 | 33 | 1 | 0 | 0 | 8 s | 8 s | 40 s | 58 |
| S1 | 17 | extracted | 02:59.9 | 7 | 19 | 0 | 4 | 73 | 12 s | 12 s | 41 s | 14 |
| S1 | 18 | extracted | 04:01.9 | 14 | 32 | 1 | 1 | 46 | 8 s | 8 s | 40 s | 88 |
| S1 | 19 | extracted | 04:10.9 | 16 | 33 | 1 | 1 | 61 | 8 s | 8 s | 40 s | 42 |
| S1 | 20 | extracted | 04:31.8 | 16 | 32 | 0 | 0 | 58 | 8 s | 8 s | 40 s | 40 |
| S2 | 1 | died | 04:02.2 | 13 | 28 | 0 | 4 | 88 | 14 s | 12 s | 40 s | 43 |
| S2 | 2 | died | 03:39.9 | 8 | 18 | 1 | 1 | 85 | 14 s | 14 s | 56 s | 75 |
| S2 | 3 | extracted | 03:01.2 | 11 | 24 | 1 | 0 | 18 | 13 s | 14 s | 40 s | 116 |
| S2 | 4 | extracted | 02:53.7 | 11 | 25 | 0 | 2 | 32 | 10 s | 9 s | 41 s | 57 |
| S2 | 5 | extracted | 02:11.6 | 6 | 18 | 1 | 2 | 75 | 13 s | 8 s | 16 s | 325 |
| S2 | 6 | extracted | 03:30.5 | 10 | 22 | 1 | 0 | 67 | 14 s | 14 s | 40 s | 59 |
| S2 | 7 | extracted | 04:09.2 | 14 | 30 | 1 | 0 | 36 | 12 s | 13 s | 40 s | 100 |
| S2 | 8 | extracted | 03:23.5 | 12 | 25 | 0 | 0 | 0 | 14 s | 14 s | 40 s | 76 |
| S2 | 9 | extracted | 04:02.1 | 14 | 30 | 0 | 1 | 12 | 14 s | 13 s | 40 s | 72 |
| S2 | 10 | extracted | 03:48.8 | 12 | 27 | 2 | 0 | 42 | 13 s | 14 s | 40 s | 118 |
| S2 | 11 | extracted | 04:07.1 | 13 | 27 | 0 | 0 | 12 | 13 s | 14 s | 40 s | 65 |
| S2 | 12 | extracted | 04:34.0 | 15 | 33 | 2 | 0 | 6 | 14 s | 14 s | 40 s | 110 |
| S2 | 13 | extracted | 05:36.9 | 13 | 32 | 0 | 4 | 70 | 14 s | 10 s | 40 s | 67 |
| S2 | 14 | extracted | 03:52.7 | 13 | 28 | 0 | 1 | 55 | 12 s | 10 s | 40 s | 77 |
| S2 | 15 | extracted | 03:20.9 | 12 | 29 | 1 | 2 | 39 | 12 s | 11 s | 40 s | 318 |
| S2 | 16 | extracted | 03:15.0 | 12 | 26 | 1 | 1 | 68 | 10 s | 9 s | 25 s | 80 |
| S2 | 17 | died | 04:40.2 | 14 | 30 | 0 | 3 | 86 | 14 s | 13 s | 40 s | 50 |
| S2 | 18 | extracted | 04:17.3 | 13 | 31 | 1 | 2 | 36 | 14 s | 11 s | 40 s | 93 |
| S2 | 19 | extracted | 03:24.9 | 12 | 25 | 0 | 1 | 65 | 13 s | 14 s | 40 s | 50 |
| S2 | 20 | extracted | 03:22.4 | 12 | 26 | 1 | 0 | 0 | 12 s | 13 s | 40 s | 56 |
| S3 | 1 | extracted | 06:15.2 | 11 | 28 | 1 | 3 | 76 | 12 s | 10 s | 41 s | 68 |
| S3 | 2 | extracted | 03:08.4 | 11 | 28 | 3 | 1 | 29 | 12 s | 12 s | 41 s | 96 |
| S3 | 3 | extracted | 03:46.4 | 11 | 29 | 4 | 2 | 75 | 13 s | 12 s | 40 s | 95 |
| S3 | 4 | extracted | 03:58.4 | 12 | 33 | 5 | 3 | 64 | 14 s | 9 s | 40 s | 176 |
| S3 | 5 | extracted | 02:11.8 | 10 | 24 | 3 | 0 | 33 | 8 s | 9 s | 30 s | 55 |
| S3 | 6 | extracted | 05:29.2 | 14 | 34 | 2 | 2 | 71 | 18 s | 13 s | 40 s | 60 |
| S3 | 7 | died | 03:57.3 | 13 | 29 | 3 | 0 | 71 | 11 s | 11 s | 49 s | 95 |
| S3 | 8 | extracted | 04:29.8 | 13 | 29 | 2 | 1 | 68 | 11 s | 11 s | 40 s | 67 |
| S3 | 9 | extracted | 03:44.8 | 13 | 29 | 2 | 1 | 49 | 10 s | 10 s | 40 s | 167 |
| S3 | 10 | extracted | 03:54.5 | 12 | 31 | 2 | 2 | 62 | 13 s | 12 s | 40 s | 151 |
| S3 | 11 | died | 03:15.9 | 10 | 23 | 1 | 2 | 62 | 12 s | 11 s | 41 s | 42 |
| S3 | 12 | extracted | 02:46.2 | 10 | 25 | 4 | 1 | 55 | 11 s | 11 s | 21 s | 90 |
| S3 | 13 | extracted | 03:48.9 | 11 | 28 | 3 | 2 | 26 | 14 s | 12 s | 40 s | 80 |
| S3 | 14 | died | 04:03.2 | 10 | 23 | 1 | 2 | 111 | 13 s | 13 s | 41 s | 34 |
| S3 | 15 | extracted | 03:09.3 | 11 | 24 | 2 | 0 | 18 | 10 s | 11 s | 41 s | 70 |
| S3 | 16 | extracted | 06:10.2 | 19 | 43 | 5 | 1 | 61 | 12 s | 12 s | 86 s | 186 |
| S3 | 17 | extracted | 04:37.5 | 12 | 31 | 3 | 3 | 79 | 12 s | 12 s | 40 s | 265 |
| S3 | 18 | extracted | 02:16.8 | 8 | 21 | 4 | 1 | 28 | 11 s | 12 s | 40 s | 134 |
| S3 | 19 | died | 03:35.9 | 10 | 20 | 1 | 0 | 72 | 12 s | 12 s | 44 s | 72 |
| S3 | 20 | extracted | 03:08.4 | 11 | 27 | 3 | 1 | 23 | 12 s | 10 s | 38 s | 118 |
