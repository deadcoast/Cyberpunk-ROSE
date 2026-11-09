Research 09 — threat awareness, alert phases and per-section bodies
====================================================================

Read 2026-11-08 for PAC-023. Sources: [Tom Leonard, Building an AI Sensory System (Thief)](https://www.gamedeveloper.com/programming/building-an-ai-sensory-system-examining-the-design-of-i-thief-the-dark-project-i-) and [AiGameDev's 14 tricks](http://aigamedev.com/open/review/thief-ai/) (both read for research 07); [Metal Gear wiki — Enemy status](https://metalgear.fandom.com/wiki/Enemy_status); [Gamepressure — Tarkov health and injuries](https://www.gamepressure.com/escape-from-tarkov/health-and-injuries/z1cf2f) and [Bamboo Gaming — Tarkov health](https://www.bamboogaming.net/tarkov/health).

What the genre does
-------------------

- **Awareness is one continuous value with discrete states and asymmetric motion.** Thief folds "lighting, object size, distance, and sound falloff into one floating-point awareness value," cuts it into low/medium/high, delays increases with "ramp-up delays [that] simulate reaction time" and jumps straight to the goal state, and decays "across some amount of time, passing through all the intermediate states" — a capacitor. Sounds arrive as awareness tied to a position; distant sounds carry less detail.
- **Phases are the player's clock.** Metal Gear's Noise → Alert → Evasion → Caution → Normal: a noise sends guards to "investigate the sound's direction, then return"; losing sight starts a counter during which noises no longer escalate; when "the counter reaches zero" patrols resume. The player always knows which phase they are in and what ends it.
- **Guards only see you once you could have seen them** (Thief's "selective vision"): no surprise attacks from outside the player's own information.
- **Bodies are sections with consequences, not a bar.** Tarkov: head and chest "Destroyed: causes immediate death"; a destroyed limb transfers its damage "to other body parts"; a fractured leg "cuts movement speed by 45%" and sprinting on it hurts; a fractured arm makes reloading and aiming "take 50% longer" (left) or "67% longer" (right) and item use slower; bloodloss is "40 health points every minute" until treated; a blinding flash lasts 10 s, a stun mutes sound 10 s.

Decisions this changed in the design
------------------------------------

Written into Full Design v3.1, Expedition mode (Threats, Combat, The Runner's body); the numbers are ours.

1. Every creature carries one awareness value 0–1 with Thief's shape: sight raises it by distance inside the
   Runner's notice range (research 07's number) and only inside the creature's field of view — which is the same
   symmetric shadowcast the Runner uses, so a creature can see the Runner only where the Runner could see it;
   sound sets a floor of 0.5 and a remembered position; decay is slow and passes through every state.
2. Four phases with visible tells: Calm (patrol or prowl), Suspicious ≥ 0.4 (walk to the last stimulus, look), Alert
   ≥ 0.8 (hunt the last known position, attack in reach), Search (Alert with no stimulus for 12 s: circle the last
   known position, then Calm). Ferals skip Suspicious: a sewerat that hears you is hunting you.
3. Scouts walk their chalk marks in pairs, map and leave: after the last mark or 90 s they walk off the route by the
   way they came. They attack only from Alert; they drop 1–2 Blackline and +15 Attention on death (design).
4. The Runner's body is four sections — head 20, torso 50, arms 30, legs 30 — each attack type lands on a section by
   its own odds; a section at 0 transfers further damage to the torso (Tarkov); head or torso at 0 is Down; arms at 0
   double dig and reload time and halve melee; legs at 0 halve speed and forbid sprint and double the extraction
   channel. Down lasts 60 s untreated, then Dead (design).
5. Statuses with durations: Bleeding 1 HP per 4 s to the torso for 60 s or until antiseptic; Blinded 4 s (sight
   radius 1); Soaked for 20 s after leaving water (lume halved, research 07); Chrome Shock from a Blackline weapon
   locks one quick slot until a Bench clears it; Fungal Fever from a Burster persists into the colony (design).
6. Stamina 100: sprint 12/s, swing 15, dodge 20, dimming veins 6/s; regen 10/s after a second without spending;
   overweight halves regen and sprint. Melee has a wind-up (pipe 0.3 s) then an active window on the tile ahead;
   a hit from behind the target's facing does ×1.5 (design: melee from behind). Dodge is 0.35 s of invulnerability
   across two tiles. Ranged is a line of sight up to 12 tiles, small magazines, a reload, and the loudest sound a
   Runner makes.
7. The run itself is one Rust simulation stepped at 20 Hz from a logged input word (design: Determinism), so the
   whole run — movement, senses, loot, creatures, combat — replays bit-exact and Godot draws it.
