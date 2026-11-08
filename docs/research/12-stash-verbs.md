Research 12 — selling, feeding and bulk management of a hoard
===============================================================

Read 2026-11-08 for PAC-026. Sources: [Escape from Tarkov wiki — Flea market](https://escapefromtarkov.gamepedia.com/Flea_market);
[GGG — The Future of Stash Tabs (staff posts)](https://www.pathofexile.com/forum/view-thread/2882324/filter-account-type/staff);
research 01 (extraction loot economies) and 08 (loot tables) in this folder.

What the genre does
-------------------

- **Selling is a priced, delayed promise, not a vending machine.** Tarkov's traders each buy at a fixed fraction of
  base (Prapor 0.5, Therapist 0.63, Fence 0.4 … Ragman 0.62); the flea market pays more but charges a fee that
  grows with the log of how far the ask sits above the item's value (0.03 × value × 4^log10(ask/value)), limits
  open offers by reputation (1 slot below 0.20, 10 at 1000+), and only lists items that were **found in raid**.
  Proceeds arrive later, through a trader, and expire if uncollected. Every one of those rules makes "sell or
  keep" a decision with a shape instead of a click.
- **Bulk handling is for the unambiguous.** GGG's stash affinities send Ctrl-clicked items to their tab, but
  "intentionally would not work with other items (such as rares you have picked up)" because those "require
  attention to identify and consider them." Bulk actions belong to the categories a player has already decided
  about; anything the player has marked as worth attention stays out of them.
- **Glut is the brake on dumping** (research 01): selling the same thing at the same place drives its price down
  within the shift and it recovers over the next, so a hoard pays more sold slowly across markets than all at once.

Decisions this changed in the design
------------------------------------

Written into Full Design v3.4 (Stash, inventory and the Bench: The four verbs, Bulk actions, Shift end; Technical
architecture: `inventory::ledger`, save v4). The numbers are ours.

1. Sell queues an item to a market the Mat has reached (a grate); with none reached the verb is closed and says so.
   The queue resolves at shift end at base × condition × demand × glut × blind, glut −5 % per unit already sold at
   that market this shift (floor 25 %) recovering 20 % per shift; an unidentified item sells blind at 40 %. A queued
   item can be pulled back until the shift ends.
2. Feed takes Chrome and Relics (the items with a feed value) to a Pit of choice; at shift end the Pit's stock rises
   by the nutrient (halved below condition 20) and the genome drifts by the item's bias × the tier's drift. The card's
   "feed now" is cells of growth: nutrient / the 0.16 a sprout costs its parent cell, about six per unit.
3. Craft opens the Bench with the item in the first slot that takes one of its craft tags; Keep pins, and a pinned
   item is outside every bulk action.
4. Bulk actions apply one verb to every unpinned item the current filters select (category, tier, condition below
   N, unidentified); the count and the eddies or nutrient they total show before the click.
5. The shift ends when a run ends, because the run clock and the city clock are one clock; the Stash shows the
   shift report (sold, earned, fed, drifted) on the way back from the run.
