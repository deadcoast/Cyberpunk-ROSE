Research 22 — the campaign's last storm, and barks that never repeat
==================================================================

Read 2026-11-11, for PAC-081 (chapters, barks, endings). Two questions where a decision turned on how other games do
it: how a scripted finale is set up so it is a test the player prepares for rather than an ambush, and how a narrator
picks one line from hundreds without the same line twice.

Sources
-------

- [Emily Short on Elan Ruskin's GDC 2012 talk, "AI-driven Dynamic Dialog through Fuzzy Pattern Matching"](https://emshort.blog/2012/03/16/gdc-2012-talk-on-dynamic-dialogue/)
  (Valve's Left 4 Dead and Team Fortress 2 system; [the session](https://gdcvault.com/play/1015317/AI-driven-Dynamic-Dialog-through)):
  the system "prioritizes rules and applies the most specific one it can find", with "less-specific ones as
  fall-backs", so writers add a special quip for a narrow situation without rewriting the general pool.
- [Prima Games, "Frostpunk – How to Survive the Storm"](https://primagames.com/tips/frostpunk-how-survive-storm):
  the scenario's finale is learned in advance — the city hears of the storm's "impending arrival" and then has tasks
  to complete before it comes — and the storm is then waited out over several days; the city that prepared comes out
  of it with few or no casualties.

What they change here
---------------------

1. **A bark is chosen most-specific first.** Ivan's lines are tagged by event and, optionally, by his Authority state,
   the strain's dominant class and the Dissolution Rite. The pool for a moment is every line of the event whose tags
   all hold, and the lines that carry the most tags that hold come first: a Hive city's Runner-dead line beats the
   general one, the general one is the fall-back. Writers add a narrow line without touching the rest (Valve's rule).
2. **A line is not said twice until its pool is spent** (this design's, from the same rule's purpose): each pool is a
   shuffled bag, saved with the run; Ivan speaks at most once in 20 seconds of the city's clock, a death or a chapter
   beat pre-empting a queued line. Without the bag 12-line pools repeat inside an hour at the bark rate measured in
   the twenty-shift city.
3. **The Great Cleansing is announced, counted down and prepared for** (Frostpunk's storm): chapter 7's card names the
   shift it comes, the HUD counts it, and the chapter's goal is the warchest the city brings to it — never a surprise
   wave. It is the one threat that does not answer Attention.
4. **The accord is the only scripted choice, and both answers bring the Cleansing** (Pass 2's one-shot; nothing here
   overrides it): the research only confirms the shape — a known, dated finale; one earlier dated betrayal.
