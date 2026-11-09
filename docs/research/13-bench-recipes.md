Research 13 — recipes by component and stats from parts
=========================================================

Read 2026-11-09 for PAC-027. Sources: [PC Gamer — Fallout 4 crafting guide](https://www.pcgamer.com/fallout-4-crafting-guide/);
[Zelda Dungeon — Fuse Attack Power](https://zeldadungeon.net/wiki/Fuse_Attack_Power); research 12 (bulk only for the
unambiguous) in this folder.

What the genre does
-------------------

- **Recipes name components, not items.** Fallout 4's recipes "call for components" — a recipe that needs rubber
  takes a basketball once it breaks down — and the workbench "strips the matching junk automatically" when a
  recipe needs a component it holds. The player's job becomes knowing which junk carries which component, and the
  game helps with "tag for search": a tagged component shows a magnifying glass on every world item that holds it.
  That is the whole reason junk is worth picking up.
- **The part's number adds to the base.** TotK's Fuse is the cleanest statement of stats-from-parts: a Bokoblin
  Horn (fuse attack 4) on a decayed Traveler's Sword (5) is a Boko Reaper at 9, and a fused weapon contributes
  "its base attack power, without its weapon bonus" when it is itself the material. The part is legible because
  its contribution is one visible number, and the same material does different things on different bases
  (teeth and claws double on arrows, halved and more on spears). The rule is simple; the outcomes are many.

Decisions this changed in the design
------------------------------------

Written into Full Design v3.5 (Stash, inventory and the Bench: The Bench; Technical architecture: the `bench`
crate, save v5). The numbers are ours.

1. A recipe is a list of slots, each a craft tag and the stat it drives; any item carrying the tag fills the slot.
   Sixty recipes in `data/recipes.toml` over four tiers (15 / 17 / 16 / 12); tier 1 is open at launch, tiers 2–4
   open with Phase E's blueprints.
2. The part's tier moves the stat it drives: ±15 % per tier above or below the recipe's tier (clamped 0.7–1.6×)
   for damage, armour, light and magazine; −1 noise per tier above, +1 per tier below (floor 0). Product condition
   is the mean of the parts' condition, the housing slot counted twice. The outcome preview shows every number
   against the base before the click.
3. Crafted gear carries its provenance: who crafted it, the day, and each part by name and tier; the Run's rules
   read the instance's stats, not the def's, so a good blade makes a better sword in the sewer, not only on a card.
4. Four product kinds besides items: repair (+40 condition), identify (one shift with an Optic part), Spored (a
   Spore Tissue or Spore Cap on a piece of gear), and a batch count for consumables (salt, cells).
5. The Bench has a queue; a job takes its tier in shifts (1, 1, 2, 3) and finishes at shift end into the Stash,
   because the shift is the one clock. Parts leave the Stash when the job is queued; cancelling returns them.
