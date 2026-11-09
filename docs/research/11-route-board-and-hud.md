Research 11 — the embark screen and the run HUD
===============================================

Read 2026-11-08 for PAC-025. Sources: [EFT-Ammo — Tarkov extraction points](https://www.eft-ammo.com/wiki/tarkov-extraction-points);
[PC Gamer — Hunt: Showdown 1896's UI complaints](https://www.pcgamer.com/games/fps/hunt-showdown-enjoys-highest-ever-player-count-after-1896-update-but-player-complaints-about-ui-are-pushing-down-its-steam-review-score-ive-never-seen-such-a-bizarre-reaction-to-so-much-amazing-content);
[LP Archive — Darkest Dungeon, team composition and preparation](https://lparchive.org/Darkest-Dungeon/Update%2004);
research 07 (Thief's light gem) in this folder.

What the genre does
-------------------

- **The raid has a clock and the clock is on the map.** Tarkov's timer "generally lasts between 30–60 minutes" and
  "varies based on map"; the map key shows "the raid timer and your available extractions"; running out means
  losing everything but the secure container. The exits split into guaranteed ones and "potential extracts" that
  "may or may not be active" — where an exit is can be learned, whether it is open today cannot. That split (the
  place is knowable, the contents are not) is what makes a map worth learning without making a run solved.
- **Fewer clicks, more on one screen.** Hunt's 1896 menu drew the most-reviewed complaint of the game's best year:
  "3 or so clicks per action more than the old UI" and "way less information on one screen without switching
  between various submenus." The in-match HUD drew no such complaint. An embark screen is read in the five seconds
  before a run; everything the decision needs has to be on it at once.
- **Length tells you how to pack.** Darkest Dungeon's quests come in short, medium and long, and provisioning
  scales with length ("Food runs from 8–12 on short missions to a full store purchase on long ones"). The card's
  length line is the input to the rig, so it has to be on the card, in the units the pack decision uses.
- **Show the rule on the body, not in a corner** (research 07): Thief's light gem puts the one stealth number
  beside the player's own hands, where the eye already is.

Decisions this changed in the design
------------------------------------

Written into Full Design v3.3 (Expedition mode: The Route Board, The Run HUD; Stash, inventory and the Bench:
quick-slot sizes; Technical architecture: `run::board`, `RunSim` shift clock). The numbers are ours.

1. The route card states what can be known before a run and nothing that cannot: length in chunks and shift
   minutes, danger from the authored spawns, the fraction of the route remembered, the containers seen on earlier
   runs (never the total), the water share, our corpses and stranded followers. Contents stay unknown.
2. The shift clock is a real number on the HUD and on the card: 8 min + 70 s per chunk beyond eight, 8:00–15:00
   for 8–14 chunks. Past zero the clock counts overtime and every minute of it spawns another pack at a burrow and,
   from S2, rolls a patrol onto an empty chalk route; the Flood joins it in Phase D.
3. One screen embarks: three cards, the memory strip of the selected route, the crew and the rig, one DEPART.
   The rig is packed from the Stash itself — the Stash is one persistent grid at the Heart, not a screen's own copy.
4. Quick slots take items of one or two cells; the pry bar (1 × 3) rides in the backpack and is found there.
5. The HUD puts the senses on the Runner (the exposure arc and the noise arc, the notice ring at its true radius,
   the channel ring during extraction and digging) and keeps the corners for the body silhouette, stamina, the
   quick slots and the clock; the loot panel is grid to grid.
