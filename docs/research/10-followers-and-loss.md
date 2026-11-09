Research 10 — followers, corpse runs and what loss keeps
=========================================================

Read 2026-11-08 for PAC-024. Sources: [GameSpot — Republic Commando designer diary 1: building the squad](https://www.gamespot.com/articles/star-wars-republic-commando-designer-diary-1-building-the-squad/1100-6112330/); [Maxroll — Diablo II death rules](https://maxroll.gg/d2/resources/death); research 01 (Tarkov insurance, Hunt's loss design) in this folder.

What the genre does
-------------------

- **Squad orders are few and the squad fights without them.** Republic Commando's designers named the flaw they
  were fixing — "Too much work. Squad games had a lot of micromanagement and babysitting" — and shipped four
  stances (form up, secure area, search and destroy, cancel) plus one contextual button for everything else, so
  the squad "aren't waiting for the next waypoint to be placed or the next elaborate button combo to be pressed."
- **A corpse is a recoverable setback with a clock on it.** Diablo II leaves "the body and all equipped gear … where
  the Character died"; dying again makes another corpse; after sixteen unrecovered deaths the gear drops to the
  floor; leaving the game keeps only the corpse "with the highest sum value of Gold." Retrieval pays back 75 % of
  lost experience. The penalty is proportional and the recovery is rewarded, so players go back for their bodies.
- **Insurance is a priced promise** (research 01): Tarkov's insured gear returns only if nobody took it, after a
  delay; what the player loses is what someone else wanted.

Decisions this changed in the design
------------------------------------

Written into Full Design v3.2 (Expedition mode: Followers, Extraction, Death and recovery; Technical architecture:
save v3). The numbers are ours.

1. Followers take three orders — Follow (the default), Hold, Guard the tile they stand on — and otherwise fight on
   their own; a downed Runner is picked up by the nearest Follow-order follower within six tiles without being told.
2. A corpse keeps the whole rig at the tile of death, identified; it is a loot source on the next run of that route.
   Raiders roll once per colony day since the death, 10 % + 5 % per stratum; a raided corpse is stripped and the
   body stays. Several corpses may lie on one route.
3. Spored gear is the priced promise: it costs a Bench craft with Spore Sac tissue, regrows at the Heart on the
   Runner's death once, and is removed from the corpse so nobody recovers it twice.
4. Items dropped on Mat tiles come home as a separate Mat haul whether the Runner lives or dies: the Mat's reach is
   the one safe place to leave weight.
5. Followers within four tiles at the end of the channel extract; the rest are stranded at the extraction tile and
   recorded on the route card for the next run.
