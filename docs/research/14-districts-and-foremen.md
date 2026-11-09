Research 14 — elected local leaders in colony sims (districts, foremen, petitions)
===================================================================================

Source read: [Dwarf Fortress wiki, Mayor](https://dwarffortresswiki.org/index.php/Mayor). Read 2026-11-09.

What the genre does
-------------------

- The mayor appears at a population threshold (50, children included) and is elected at a fixed calendar moment
  (summer, day 17); the dwarf with the highest social / conversational skill usually wins; absent dwarves cannot be
  elected; the term is a year and re-election is allowed.
- The office has teeth in both directions: the mayor issues **mandates** (make or do not export this good, one at a
  time) and **demands** (furniture for their rooms, up to two); meeting a mandate pleases them, missing it makes them
  unhappy and they order a punishment. The office is a pressure the player answers with production.
- The rooms the office requires (office, quarters, dining room of a quality) are a standing material ask that grows
  with rank.

Decisions this changed in the design
------------------------------------

1. **A district elects at a threshold and on a clock**, as the mayor does: a hub Node with three built structures
   forms the district; the election runs the shift it forms and then every ten shifts, and at once when the foreman
   dies or leaves the district. The social skill as the deciding stat becomes **Voice** (the stat the design already
   gives "foreman ability"), plus a Mood term so a miserable Voice does not keep the office: weight = Voice + Mood / 20,
   Orator ×2 (the trait already says "election weight ×2"); a slimo out on a run or stranded cannot be elected.
2. **Petitions are mandates**: one open petition per district, with a named ask (a structure the district lacks for
   the need that is failing) and a deadline of five shifts; met, the district's Mood target rises for three shifts;
   missed, the target falls until it is met. The foreman's priorities override a member's when higher, which is the
   design's existing rule and the office's second tooth.
3. **No punishments**: the genre's "dwarven justice" is replaced by the unrest lines the Charter reads (strike,
   defection, petition), so an unanswered foreman costs output and people, never a scripted beating.
4. **The office asks for nothing for itself**: the foreman's rooms and furniture are dropped; the district's needs
   are the office's demands. A Charter Board in the district is the one standing material ask, and only to pass laws
   there (PAC-042).
