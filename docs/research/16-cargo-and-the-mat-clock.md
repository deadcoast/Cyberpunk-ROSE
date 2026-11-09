Research 16 — cargo on the Mat, and what feeds it
=================================================

Sources read 2026-11-09: the design's own sources, `docs/02/Pass 2 (pillars).md` (Pillar 3: cargo routing, quotas,
the risk row "Trunk capacity", the views table, the backend list) and `docs/01/Pass 1 (superseded).md` (the CA's
upkeep and its tuning shift, the tiers' feed values, "population costs nutrient"); Factorio's logistic requests
([forum: do robots count the items in flight](https://forums.factorio.com/viewtopic.php?p=346222)); and a
measurement of this repository's Mat (`logistics::scale_scenario`, seed 11, the founding growth of 500 steps).

What the genre does
-------------------

- Factorio's requester chest asks the network to "keep N" and subtracts what is already in transit from the
  request: "the network realises that 4 of those items are in transit so it needs to provide 6 more". Without it
  every bot dispatched before the first arrives over-fills the chest.

What the measurement says
-------------------------

The game's colony ran Phase A's scale proof as its city: three thousand slimos walking market → nearest Pit and
feeding 0.3 on arrival, plus every market landing 12–30 synthetic units every 15 ticks. None of it came from the
Stash, so the Mat was fed for free. Taken away, the founding Mat (1 506 cells) at the 2 Hz clock it ran on:

| Mat steps | Fed by the phantoms | Fed by nothing |
| --- | --- | --- |
| 600 (5 min at 2 Hz) | 1 780 cells | 1 169 |
| 2 400 (20 min) | 1 561 | 518 |

A shift is a run, 8–15 minutes: 960–1 800 Mat steps at 2 Hz. Pass 1 tuned the CA on a shift of 125 steps
("0.004 N per tick, 0.5 N per shift"). At 125 Mat steps a shift, with nutrient into the Heart's Pit once a shift:

| Fed a shift | Cells after 4 / 8 / 12 / 16 shifts | Signalled tissue (σ ≥ 0.3) within 12 of the Heart |
| --- | --- | --- |
| 0 | 1 254 / 944 / 726 / 583 | 9 / 7 / 4 / 4 |
| 4 nutrient (≈ 20 Common Chrome) | 1 287 / 992 / 774 / 630 | 34 throughout |
| 12 nutrient | 1 280 / 1 012 / 823 / 686 | 137 / 127 / 121 / 116 |

A run's rig carries 10–25 items, about half of them Chrome at 0.18–0.30: 2–4 nutrient if all of it is fed. Pass 1's
own numbers agree with the measurement: 0.6 a tick sustains about a thousand cells, which is 75 a shift, and Pass 2's
chapter 1 asks for 60 cords at ten minutes — the Mat one haul a shift can keep.

Decisions this changed in the design
------------------------------------

1. **The Mat steps once every 12 city ticks** (125 steps in a 12.5-minute shift), the shift its rates were tuned
   on, while cargo and slimos keep the 2 Hz tick. At 2 Hz the CA ate eleven Pass 1 shifts in one run.
2. **The Mat eats only what the city feeds it**: its Pits (feeds, quotas, a dissolved D5) and the Deposits under it.
   The phantom crowd and hauls stay in `rose scale` as the load test of the arrays; the game's movers are the
   roster and its cargo is the Stash's.
3. **The founding stays** (the 500-step stratum, its Heart, seven Relays and their Pits) and is named for what it is:
   more Mat than one Runner can feed. The core holds on about four nutrient a shift; unfed the outer Mat recedes
   about five per cent a shift to what its Deposits carry. Chapter 1 (Phase E) replaces it with Pass 2's opening.
4. **A quota counts what is riding to it** (Factorio's in-transit rule), so one shift's pull never over-fills a Pit.
5. **Hauls arrive at the Heart**, where the Stash is, not at a market: the split to quotas happens there, by Pass 2's
   routing rule (demand per tile of distance, ties toward the Heart), so the design keeps one ordering rule where
   Pass 2 had two (the routing and "the priority order set on the market").
