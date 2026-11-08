Research 07 — light, noise and fog as stealth systems
=====================================================

Read 2026-11-08 for PAC-021. Sources, in the order they are used below:

- [Game Developer — Building the original Thief's revolutionary stealth system](https://www.gamedeveloper.com/design/building-the-original-i-thief-s-i-revolutionary-stealth-system) (Paul Neurath on the light gem)
- [Tom Leonard — Building an AI Sensory System: Examining the Design of Thief: The Dark Project](https://www.gamedeveloper.com/programming/building-an-ai-sensory-system-examining-the-design-of-i-thief-the-dark-project-i-)
- [AiGameDev — Sneaking Behind Thief's AI: 14 Tricks](http://aigamedev.com/open/review/thief-ai/)
- [Brogue wiki — Stealth](https://brogue.wiki/wiki/Stealth)
- [anderoonies — Brogue generation, part 3 (lighting)](https://anderoonies.github.io/2020/11/14/brogue-generation-3.html)
- [dktapps — Minecraft lighting algorithm spec](https://github.com/dktapps/lighting-algorithm-spec) and [Grey's Minecraft coder — Lighting](https://greyminecraftcoder.blogspot.com/2020/04/lighting-1144.html)
- [Albert Ford — Symmetric Shadowcasting](https://www.albertford.com/shadowcasting/)
- [Game Developer — Reinventing stealth in 2D with Mark of the Ninja](https://gamedeveloper.com/design/reinventing-stealth-in-2d-with-i-mark-of-the-ninja-i-) and [Designing Sound — The dynamics of Mark of the Ninja](https://designingsound.org/2013/06/the-dynamics-of-mark-of-the-ninja)
- [PC Gamer — Let's hear it for Hunt: Showdown's incredible sound design](https://www.pcgamer.com/lets-hear-it-for-hunt-showdowns-incredible-sound-design)
- [PZwiki — Sneaking](https://pzwiki.net/wiki/Sneaking)
- [Guild Order — Tarkov audio fluency and sound triangulation](https://guildorder.com/hi/games/tarkov/guides/audio-fluency-and-sound-triangulation)
- [Game Developer — How Darkwood's visibility mechanics create a new kind of horror](https://gamedeveloper.com/design/how-i-darkwood-i-s-visibility-mechanics-create-a-new-kind-of-horror)

What the genre does
-------------------

**Exposure must be shown, as one number.** Thief's team found that without feedback "you got confused signals" and
guards looked arbitrary; the fix was the light gem, which "gets brighter or dimmer depending on how visibly hidden
you are." Leonard's sensory paper defines the player's visibility as "the lighting, movement, and exposure (size,
separation from other objects) of the entity," stored as 0–1 analog values and aggregated, and biases the lighting
sample toward the floor under the player "so the player can anticipate their own safety." Mark of the Ninja takes the
same idea to its limit: "a very binary visual contrast between being hidden and visible."

**Darkness changes the detection range, not a probability.** Brogue's stealth range is 7 in normal light, 3 in the
dark, 14 when lit (Stealth page); monsters inside the range have a flat 25 % chance per turn to notice; resting
halves the range; armour adds to it. Dark halves, lit doubles, everything else nudges. As you descend "their vision
range shrinks, so monsters may notice you before you notice them" — the miner's light is the player's own radius in
the dark, and it is the thing that gets worse with depth.

**Light is a per-cell integer field propagated from sources.** Minecraft stores 0–15 per block; each block's opacity
is the decrement from its brightest neighbour ("a low opacity (eg 1) means the light level decreases by 1"), water
costs 3 per block instead of 1, solid stone is opaque; emitters set their own level if higher than what arrives.
Brogue's lights are a halo of radius and fade "masked against a FOV restricted by objects which block the line of
sight," overlapping lights add, and light lands on walls, floors and water alike. Both are cheap, deterministic and
look right at tile scale.

**Field of view should be symmetric.** Ford's symmetric shadowcasting guarantees "if any floor tile A is in the field
of view of a floor tile B, then B will always be in the field of view of A," so "applications of line of sight like
ranged combat will match field of view" and cutting a corner never lands you beside a tile you could not see. The
slope-rounding rule (ties up for the start, down for the end) is what makes it exact.

**Noise is a shape in the world, and the player should see it.** Mark of the Ninja draws "a spreading blue ring that
says exactly how far that noise will reach"; Nels Anderson's reason is that stealth systems are "pretty invisible"
and players otherwise learn them "through failure and risk-taking." Thief's sounds are "tagged with semantic data
and propagated through the world's 3D geometry," arrive "from the directions they would in the real world, carrying
attenuated awareness values," and distant sounds give less detail than near ones. Awareness goes up through a
time-based filter and jumps to the goal state; it comes down "across some amount of time, passing through all the
intermediate states" — a capacitor.

**Surfaces and routes carry sound, not straight lines.** Tarkov players read that metal grating, tile and concrete
"carry far and read sharp," grass and mud are "softer and shorter," sound "bends around corners, leaks through some
surfaces but not others, and funnels down corridors," and a loud-but-dull sound means near but obstructed. Hunt:
Showdown builds its tension out of authored sound traps — ravens, chains, twigs, screaming carcasses — and the fact
that "every shot echoes across the map"; players use noise tactically (ring a bell to pull a monster off a clue).
Project Zomboid quantifies the same trade: sneaking is a 20 % footstep radius multiplier and 40 % detection-chance
multiplier before skill, with skill levels sliding the radius from 24 % to 8 %; a house alarm is a "county-scale
dinner bell" at about 600 tiles.

**Not seeing is the horror.** Darkwood restricts sight to "what was directly in front of them with perfect clarity,"
lets the player "see the movement of objects in the distance without displaying what it was that moved them," and
treats "a lack of clarity as a system of its own"; its lighting "was intended to conceal things at least as much as it
was to illuminate them."

Decisions this changed in the design
------------------------------------

Everything below is ours; the sources argue for the shapes, the numbers are ROSE's. The Full Design v3 fixes only:
lume = vein glow + lamp + the Lume gene of Mat you stand on; lume sets the radius you see and are seen from; veins
can be dimmed at a stamina cost; noise travels along open tiles and pipes, water halves it, brick carries it; light
is authored per chunk, dark by default; fog of war with per-route memory. Values marked **(ours)** are PAC-021's
additions and are the balancing knobs.

1. **Light is a 0–15 integer field** (Minecraft's scale, Brogue's halo). Static sources: fixtures emit 6 **(ours)**,
   grates emit 9 (Night City leaking down), Mat tiles emit 2 + Lume gene × 6. Dynamic sources: the Runner's lume,
   muzzle flashes (10 for one tick), lamp carriers (10, white). Propagation is an 8-connected Dijkstra with cost 2
   orthogonal / 3 diagonal, water ×3 (the Minecraft water rule), solids stop it but take the arriving level so lit
   wall faces read; level = emit − ⌈cost/2⌉; overlapping sources take the max. Three named bands: **dark ≤ 2, dim
   3–6, lit ≥ 7** (Brogue's three stealth states).
2. **The Runner's lume** = veins (Mod-Depth 1–5) + gear light (lantern 6, lamp pole 8, visor −1, already in
   `data/items.toml`) + Mat bonus under the feet, −3 while dimming veins (stamina drain), halved while Soaked;
   clamped 0–15. Dimming is a hold, not a toggle, so the cost is felt.
3. **Sight = symmetric shadowcasting ∩ (lit or near).** A tile is visible when it is in the Runner's symmetric FOV
   and either its light is ≥ dim or it lies within the see-radius `2 + lume + Eye/2` **(ours)** — Brogue's miner's
   light. Dark rooms are seen only as far as your own glow; a lit pool is seen from across the chunk. Symmetry means
   anything that can see the Runner can be seen by a Runner looking that way.
4. **Exposure = max(light at your tile, your lume), one number on the HUD** (the light gem), and it sets the range at
   which anything can notice you: **notice range = 2 + exposure** tiles **(ours)**, so dark/dim/lit land at roughly
   Brogue's 3/7/14. Movement nudges it as Thief and Brogue do: standing still −1, sprinting +2, digging +1.
   Detection itself (chance per tick, awareness capacitor) is PAC-023's; the range is the contract.
5. **Noise is a 0–15 footprint, not a circle.** Each event stamps `loudness − ⌈cost/2⌉` over the tiles it reaches by
   Dijkstra with per-tile costs **(ours)**: open 2 / diagonal 3, water 4 (halved reach), pipe 1 (carries far down a
   run), brick 3 (passes through a one-tile wall into the next room), rock and concrete block. The field decays 2
   per tick (20 Hz) so a footstep is gone in a tenth of a second and a gunshot lingers under a second; events also
   go into a ring buffer with tick and position for the AI's memory (Thief's awareness relation tied to a place).
6. **Loudness table (ours):** sneak step 2, walk step 4, sprint step 7, dig 8 per tick, container open 3, body pickup
   4, dropped item 5, extraction channel 9, melee swing 3 + gear noise, gunshot 8 + gear noise (2–6 in the table).
   Surface adds: water +1 (splash), grate +2 (metal). Hearing thresholds for PAC-023: sewerat 2, scout 4, so a
   sprinting Runner on a grate is heard across most of a chunk and a sneaking one is not heard past the room.
7. **The footprint is drawn** — the Mark of the Ninja ring, but honest about pipes: tiles the noise reached flash a
   1-bit cyan dither that fades with the field. Nobody learns the propagation rules through failure.
8. **Fog has three states per tile: unseen, remembered, visible**, drawn as a Void ordered dither at 100 % / 75 % /
   by light level over the scene so the dither grid is the art's pixel grid. Remembered tiles are what the route
   card shows and what persists: a per-route bitset in the save (`route_memory`, save version 2), so a route is
   learned across runs and a death still pays in map.
9. **What moves in the dark is shown as movement, not as a thing** (Darkwood): a creature outside visibility but
   inside hearing range is drawn as a one-pixel stir in its tile's dither for the tick it moved — PAC-023 draws it,
   the fog field exposes the state now.
10. **Authored sound traps are a chunk glyph**, not a system: hanging chains, loose grates and gas-pocket hiss become
    chunk props in a later authoring pass (Hunt's ravens and chains). PAC-021 ships the propagation they will use.
