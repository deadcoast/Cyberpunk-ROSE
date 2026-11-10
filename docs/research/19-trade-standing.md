Research 19 — trade, standing and stock
=======================================

Sources read 2026-11-10: [RimWorld wiki, Trading](https://www.rimworldwiki.com/wiki/Trading), [RimWorld wiki,
Goodwill](https://rimworldwiki.com/wiki/Goodwill), [Guild Order, Tarkov trader loyalty
system](https://guildorder.com/games/tarkov/wiki/trader-loyalty-system), with the design's own: Pass 1's
Economy and Trade tables (tier prices, the factions' buys, sells and levers, reputation −3 to +3 moving prices 30 % at
the ends, glut −5 % a unit and +20 % a shift), Pass 2's contract chains ("deliver 40 Downer at the north grate in 3
shifts"; a 5-link Corpo chain pays triple five singles) and the Full Design's Economy section.

What the genre does
-------------------

- RimWorld sells a colony's goods at 60 % of market value and buys at 140 % — the trader's spread is 2.3×. A
  negotiator's Social skill moves both (the Trade Price Improvement stat); a few traders charge double for certain
  goods. A faction base restocks every 30 days; a trader has limited silver, and what is handed over past it is given
  away.
- RimWorld's goodwill drifts back toward a faction's natural range: from below it rises 0.2 a day, from above it
  falls 0.4 a day; at −75 a neutral faction turns hostile and will not trade.
- Escape from Tarkov's traders have four loyalty levels; the next opens when three things are met together: the
  player's level, the standing with that trader, and the money traded with them — buying and selling both count.
  Standing comes mostly from that trader's quests, and some quests cost standing with a rival trader.

Decisions this changed in the design
------------------------------------

- The volume that opens a stock tier is eddies traded with the faction both ways, sold and bought (Tarkov), not
  sales alone: a player who buys their paste from the Nomads is doing business with the Nomads.
- Contract links cost the faction's rival favour (Tarkov's rival standing), the mechanism under Pass 1's "never
  selling them out to Badges" and Pass 2's alliance that drops "its rival" to −2. The Gangs and the Badges are each
  other's (Pass 1); the Corpo remnants and the Nomads are this design's pair.
- Vendors sell at twice an item's base value, before demand and reputation (this design's, near RimWorld's 2.3×
  spread): a haul is worth half as much bought back, so buying is for what the city cannot find.
- The market's staff haggle: the best Trade skill among the Seep Market's Haul slots adds 2 % a level to what the
  stall sells for and takes 2 % a level off what it buys (RimWorld's negotiator). Pass 1's Past bonus stays: a staff
  slimo of the faction's Past counts as one more reputation at that stall.
- No silver cap: the glut (−5 % a unit, floor 25 %) is the limit on what one stall will pay in a shift.
- Stock rotates daily (the Full Design's rule) instead of RimWorld's 30-day restock: a shift is the game's clock and a
  day is three of them.
- Below reputation 0 a faction comes round a favour a shift (RimWorld's drift from below), so −3 — the stalls closed —
  reopens in twenty shifts. `rose economy` found the need: a city whose only reached grate closed had nothing left
  that could earn favour there. Above 0 nothing decays; the levers are what hold a standing up.
