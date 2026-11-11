Research 20 — repair
====================

Sources read 2026-11-11: [RimWorld wiki, Structure](https://rimworldwiki.com/wiki/Structure), [RimWorld wiki, Repair
Success Chance](https://rimworldwiki.com/wiki/Repair_Success_Chance), [0 A.D., Repairable.js](https://docs.wildfiregames.com/javascript/simulation/Repairable.js.html),
with the design's own: Pass 2's Charter (the Foreman's Cut: "Districts self-repair faster; your quotas fill slower"),
the Full Design's Building paragraph ("a Build slimo mends 10 a shift"), the Scrap Forge's row and the Foreman's Cut
paragraph ("the stock pays the district's self-repair when structures take damage").

What the genre does
-------------------

- RimWorld repairs damage as Construct work: the pawns assigned to Construct mend a damaged building back to full.
  A breakdown is another thing — it needs a component and can fail by Construction skill; damage repair does not.
- 0 A.D. repairs as building: a builder's rate restores hit points so that a full repair takes a ratio of the
  building's build time, extra builders add less each (ten work at about five), and the code charges no resources
  ("TODO: should we have resource costs?").

Decisions this changed in the design
------------------------------------

- Mending is Build work and costs no Parts (both games): what a damage costs is the builders' shifts, which the
  ghosts and upgrades wanted.
- The rate is the Full Design's own, tied to the builder's Hands as building is: 2 × Hands HP a shift, so 10 at
  Hands 5. The 0 A.D. rule (a full mend for the build points) would mend a Sphincter Gate's 400 HP for its 8 points,
  a Cleansing's burn undone by two builders in a shift.
- Two builders to a damaged structure, as to a ghost: the slot cap does what 0 A.D.'s falling returns do.
- The Foreman's Cut's "self-repair" is the stock it already takes: in its districts each packet of the foreman's
  stock mends 10 HP at the shift's end with no builder, the Full Design's line that the stock pays for it.
