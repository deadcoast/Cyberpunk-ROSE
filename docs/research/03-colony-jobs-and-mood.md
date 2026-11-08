Research 03 — colony-sim job and mood systems
=============================================

Source read: [RimWorld wiki, Mood](https://rimworldwiki.com/wiki/Mood). Read 2026-11-07.

What the genre does
-------------------

- Mood is 0–100; a target is computed from a difficulty base plus the net of positive and negative moodlets; actual
  mood moves toward the target at bounded rates (about +12 and −8 per in-game hour), so swings are visible, not instant.
- Three break lines at 35 / 20 / 5 with escalating loss of control; sleeping pawns cannot break; prolonged low mood
  can make a colonist leave.
- Passion for a job gives a large mood boost and faster learning, so assignment is a mood lever as well as an output lever.

Decisions this changed in the design
------------------------------------

1. **Mood moves toward a target at bounded rates** (+4 / −3 per shift) instead of jumping; the Charter's signal-wave
   propagation gives the far-district delay on top. The player sees a trend, which is what makes a law legible.
2. **Three unrest lines** at 30 / 20 / 10 map to strike / defection / petition, as already drafted; the bounded rate
   means a player has about two shifts of warning between lines.
3. **Sleep pauses unrest**: a slimo in a Pod during Dark cannot defect, so Curfew is a real unrest tool with a real
   output cost, not a flavour law.
4. **Job passion becomes Traits**: Hoarder loves Scav, Teacher loves Tweak around others; assigning against a trait
   costs Mood, assigning with it earns it. This is the mood lever the genre puts on assignment.
5. **Expectations scale with city tier** (Hamlet to Empire): the same Pod that satisfied a Hamlet slimo reads as a
   slum to an Empire one, which is why visual tiers of structures are a need as well as a reward.
