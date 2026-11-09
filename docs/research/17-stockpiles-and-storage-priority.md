Research 17 — stockpiles and storage priority
=============================================

Sources read 2026-11-09: [RimWorld wiki, Stockpile zone](https://rimworldwiki.com/wiki/Stockpile_zone) (priority,
filters, haulers), with the design's own: Full Design v3.11 (Zones, the Rack and the Cold Store rows, Cargo) and
Pass 1's steering tools (Pit 4 T1 and one tweaker-shift, Bait 1 T1, a Salt Line a sack per 8 tiles, a Severing one
tweaker-shift, the Relay Shrine 6 T2 and two tweaker-shifts).

What the genre does
-------------------

- RimWorld stores by zone: each stockpile has a filter (the items it allows, hit points, quality) and a priority
  (Critical, Important, Preferred, Normal, Low). "Items are brought to stockpiles with higher priority that have
  available space", and haulers "will even move items already in a stockpile that can be moved to a higher priority
  one". A Low stockpile takes overflow and stays empty when supply is short; a small high-priority one beside a
  workbench is kept full by haulers as the crafters draw it down.

Decisions this changed in the design
------------------------------------

1. **A stockpile has a priority, 1–4 on a job's scale (4 the most wanted)**, and the Heart's grid stands between 2
   and 3. At 3–4 a stockpile draws what it accepts out of the grid and out of stockpiles below it (RimWorld's
   move-to-better); at 1–2 it takes only what the grid cannot hold (RimWorld's Low as overflow). The ROSE twist is
   that the Heart's grid is the Stash the player works in, so the default Rack is an overflow store (2) and moves
   nothing the player did not overfill; a Cold Store defaults to 3 with an Organic filter, since keeping food from
   spoiling is why it is built. One scale for jobs and stores: on both, the higher number wins.
2. **The stockpiles are the Stash's rooms**, and every verb works where an item lies; a stockpile the player has to
   empty by hand before selling from it would be storage that costs a shift to use.
3. **Haulers make the painted zone count**: cargo reaches the cells on the Mat; the rest of the zone needs a hauler.
   Without one a stockpile holds its Mat cells and its own footprint, so a hauler is worth a slot only where the
   zone reaches off the Mat — Pass 2's "haulers carry what cords cannot".
4. **The steering tools take Pass 1's costs**, converted at the Pit's own rate (a tweaker-shift is the 8 build points
   a Pit costs): Sever is a cut of 8 points by the Build job, the Relay 6 Chrome and 16 points as a structure, Salt a
   bag per eight tiles, Bait one Common Chrome (PAC-043).
