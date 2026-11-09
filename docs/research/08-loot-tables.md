Research 08 — loot tables, streak control and tools that buy certainty
=====================================================================

Read 2026-11-08 for PAC-022. Sources:

- [PureDiablo wiki — Treasure Class](https://www.purediablo.com/d2wiki/Treasure_class)
- [Liquipedia — Dota 2 Random Distribution](https://www.liquipedia.net/dota2/Random_Distribution)
- [Risk of Rain 2 wiki — Chest](https://riskofrain2.wiki.gg/wiki/Large_Chest)
- [Gfinity — Escape From Tarkov developer explains 12.6 looting changes](https://www.gfinityesports.com/article/escape-from-tarkov-12-6-loot-changes-pc-looting)
- [Darkest Dungeon wiki — Curio](https://darkestdungeon.gamepedia.com/Curio?oldid=4951)
- Research 01 (extraction loss design) and 02 (survivors cadence: scripted generous first chests) in this folder.

What the genre does
-------------------

**Tables are nested pools gated by level, not flat item lists.** Diablo 2 groups base items into Treasure Classes
"with similar levels," a monster "has a range of assigned TCs it may drop from," the TC is chosen first and the item
second, and quality is rolled last in a fixed order ("Unique > Set > Rare > Magical > High Quality > Normal").
Level gates the top: "A monster must exceed the Qlvl of the Item Set, or the Unique, to drop that item," and
"Monsters are always very unlikely to drop from their highest possible TCs" (Pindleskin's best armour class at 1,233
to 1). Magic find moves quality, never the pool.

**Rarity bands are steep.** A Risk of Rain 2 small chest is 79.2 % common, 19.8 % uncommon, 0.99 % legendary. The
legendary is in the table at all so every chest carries the possibility; it is this rare so it stays an event.

**Streaks are controlled, not left to the dice.** Dota 2's pseudo-random distribution raises the chance by a
constant C on every miss, P(N) = C·N, and resets on a hit; C is set below the nominal rate (a 25 % proc starts at
8.5 %) so the average still lands on the listed number. The stated goal is "to limit the influence of huge
streaks": at 25 %, the standard deviation of attempts falls from 3.49 to 1.90, and the proc is guaranteed by the
12th attempt. Bad luck gets shorter; so does good luck.

**Memorised loot routes are the enemy of an extraction game.** After Tarkov's 12.6 patch the lead developer
confirmed "loot spawn chances were randomized more," from a 25–30 % chance per spawn to 10–60 %: where things are
is learnable, whether they are there this run is not.

**A tool trades a consumable or a slot for certainty.** Darkest Dungeon's curios have outcome tables (a Locked
Strongbox: 50 % loot, 25 % blight) that a supply item replaces: a key makes it "100 % Loot (3x Any)," a shovel 66.6 %
loot with the blight removed, bandages turn 40 % loot into 100 %. The player decides before the run what certainty
to carry.

**The first haul is scripted** (research 02): Vampire Survivors' first chests are generous before the real odds apply.

Decisions this changed in the design
------------------------------------

The Full Design v3 fixes: lockers and crates tiered by chunk; deposits dug 4–10 s and loud; corpses with inventory;
Oathkeeper scouts drop 1–2 Blackline; S2 loot tops out at Uncommon; each stratum up roughly doubles tier value;
found items are unidentified until appraised; first three containers of a route's first run roll one tier up.
Everything marked **(ours)** is PAC-022's addition and lives in `data/loot.toml` so balancing never recompiles.

1. **Tables are data, resolved D2-style**: a source (locker, crate, deposit, newcomer corpse, scout drop, carrier
   drop, cache) names its pick count, a NoDrop chance per pick, a category pool with weights, a condition band and
   one tier-weight row per source tier 1–6. A pick rolls NoDrop, then category, then tier from the row, then an item
   uniformly from that category and tier, stepping down a tier when the table has none. Tier gates the top the way
   level does in D2: the S1 row has no Rare at all, the S2 row none either (so S2 tops out at Uncommon), Rare enters
   at tier 3, Blackline at 4, Relic only in the cache table **(ours)**.
2. **Rows are steep** **(ours)**: tier 1 is 80/20, tier 3 is 50/38/12, tier 6 is 10/30/35/25 — the Risk of Rain shape,
   widened per stratum so that mean container value roughly doubles per stratum (the design's rule); the sweep in
   `rose route --loot` prints the measured ratio.
3. **Notable picks run on PRD** **(ours)**: the "notable" roll (Rare or better, where the row allows it) uses Dota's
   constant-ramp: the chance is C·N for N picks since the last notable, C solved from the row's nominal notable
   share, reset on a hit. Dry streaks end; hot streaks end too. The counter is per route per run.
4. **First-run generosity** (research 02): the first three container opens of a route's first run roll on the row one
   tier up. The route memory in the save carries the run count.
5. **Where is learned, what is not** (Tarkov): containers are authored into chunks so the route card can show them,
   but contents roll per run from (route seed, run nonce, container index), so re-running a route never farms the
   same locker.
6. **Tools buy certainty** (Darkest Dungeon) **(ours)**: crates are sealed corporate shipping; prying one with a Pry
   Bar is quiet (noise 3) and adds a pick, forcing it by hand is loud (noise 6) with no bonus. Lockers are 30 % locked
   at tier 1 rising 5 % per tier; a Pry Bar opens one at noise 4, forcing by hand is noise 8 and half the contents
   lose 30 condition. The bar costs a 1×3 slot and 2 kg carried in, before anyone knows if it will be needed.
7. **Deposits** dig into a pile on the tile: Parts and Chrome, 1–2 picks, NoDrop 15 %, condition 40–80 **(ours)**;
   the pile is a source like any other.
8. **Newcomer corpses** carry what a person carries down here: organics, a curio, a part, a piece of worn gear at
   condition 20–60, 2–4 picks **(ours)**; the body itself is PAC-024's recruit.
9. **Oathkeeper drops** are fixed-tier tables: scouts 1–2 Blackline from chrome, gear and tech plus a 50 % vestment
   cloth; lamp carriers add the lamp pole **(ours, from the design's 1–2 Blackline)**.
10. **Found items are Unknown**: the manifest shows the tier colour, category and silhouette, never the name, and the
    item sells blind at 40 % until appraised (design). Taking is into the Rig by auto-place; what does not fit stays in
    the container, so weight and grid are the real limits, not a count.
11. **Measured, not asserted**: `rose route --loot` sweeps seeds per stratum and prints per-container value
    mean/p50/p90, tier mix, notable rate and the stratum-to-stratum value ratio; tests pin the S1 band, the
    S2-tops-at-Uncommon rule, the per-stratum climb, and that PRD keeps the longest dry streak under the guaranteed
    bound.
