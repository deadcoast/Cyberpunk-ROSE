Research 18 — threat pacing
===========================

Sources read 2026-11-09: [RimWorld wiki, AI Storytellers](https://rimworldwiki.com/wiki/AI_Storytellers), with the
design's own: Pass 1's Threats (Attention per grate zone, Patrol 30 / Sweep 60 / Cleansing 90, decay 2 a shift, the
Cleansing's three phases), Pass 2's first session (a Patrol at minute seven) and reward ladder (a Patrol repelled at
5–10 minutes), and the Full Design's Risk vs reward table.

What the genre does
-------------------

- RimWorld's storytellers size a raid by "raid points" and read, before each event, the colony's wealth, its
  buildings' wealth, its colonists and animals, recent deaths and severe wounds, and the time since the last major
  event. Cassandra runs "a classic increasing curve of challenge and tension" — push, ease off, push again. Losses
  feed back through an adaptation score that grows with time and drops when the colony is hurt (growth 100 % and
  impact 90 % at Strive to Survive).
- The page gives no minimum gap between major threats; the time since the last one is an input, so the next threat
  waits on it rather than firing the moment the colony is rich enough.

Decisions this changed in the design
------------------------------------

- Pass 1 sets the lines (30, 60, 90) and the decay (2 a shift) but not what happens while a zone sits above a line.
  Read literally, a zone at 35 would take a Patrol every shift until decay brought it under 30. The design gives each
  stage a quiet spell instead — a day after a Patrol, two after a Sweep, five after a Cleansing — inside which the
  same zone does not see that stage or a lower one again; a higher stage can still come. This is the storytellers'
  "time since the last major event" in the gauge's own terms: Attention stays the size of the threat, the quiet
  spell its rhythm.
- Wealth already feeds the gauge (trade value, Blackline held, Stills, Lume), so no separate wealth term is added.
- Adaptation (easing after losses) is left out: the Cleansing is the lore's genocide and the design wants it to land
  whatever the city lost before it. Difficulty moves the decay instead (Sump 3, Undercroft 1, the Game shell).
