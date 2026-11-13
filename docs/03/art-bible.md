Art bible — Sewer Noir 32
=========================

One style for every pixel on screen, placeholders included. Anything this document does not cover is decided by the
rules in it, and `rose-tools validate` enforces the parts a tool can check.

Grid and scales
---------------

    world base            640 x 360, integer-scaled (3x at 1080p, 6x at 4K); no sub-pixel motion anywhere
    tiles                 32 x 32
    slimos                32 x 48            Ivan, lamp carriers   48 x 64
    sewerats              32 x 32 (Maw 64 x 48)
    items                 32 x 32 icons on a 24 x 24 inner silhouette grid; inventory cells 32 px
    UI                    4 px rhythm, 8 px margins; 9-slice frames with 4 px margins
    panels and codex art  1280 x 720 — exactly 2x the world, so panel pixels are world pixels doubled

Line
----

Sprites, item icons and UI frames carry a one-pixel selective outline: the darkest step of the object's own ramp,
stepping to Void where the ground behind is dark (most of a sewer), broken where a form turns into shadow. Internal
lines are one step darker than the area they sit in; lines thinner than a pixel are implied by broken segments near
shadow. Tiles carry no outline; a tile is background and an outline on the background separates nothing.
(Source and reasoning: docs/research/04-pixel-art-craft.md.)

Value and colour
----------------

Three value steps per ramp, hue-shifted: shadows pull toward Drain Blue, lights pull toward the nearest emitter's hue.
Gradients are two-tone ordered dither at 50% and 25%; three-tone gradients never. Pixel clusters follow form: no
single-pixel noise, no banding. Saturated colour exists only where something emits light.

SEEPAGE v2 (game/assets/palette/SEEPAGE_v2.gpl)
------------------------------------------------

    Noir        07070C Void   0E1018 Sump   171A26 Culvert   232737 Concrete Night   343A4F Wet Concrete
    Industrial  4A4F5E Rust Gray   6B6F7B Rebar   8E9097 Dust   B5B4B0 Chalk
    Mud         2A1F19 Silt   4A3526 Sewer Brown   6E4E34 Rust   9A7049 Oxide
    Bile        2E3A1E Bile   4E5E27 Vomit Green   7A8A3A Pond Scum            (sewer water and mould only)
    Drain       122238 Drain Blue   1F3D63 Cistern   35679C Flood Light
    Flesh       5B3A4A Bruise   9A6070 Flesh Dim   D99AA6 Flesh
    Ooze        2A1140 Ooze Dark   5A1E8C Ooze   A44BE0 Ooze Lit   F0A6FF Ooze Glow   (emits: the Mat, D4 bodies, veins, Ivan)
    Neon        FF2E97 Hot Pink   19E6FF Cyan   F6FF2E Acid Yellow   FF6A1A Muzzle Orange   (emits: leaks, UI, Ivan's rim, fire)
    White       F2EFE4 Vestment White   D4A52A Reliquary Gold   (emits: Oathkeepers only)

Rules: (1) environment and bodies use Noir, Industrial, Mud, Bile, Drain, Flesh; (2) Ooze, Neon and White appear only
as emitters, light pools, UI and signage; (3) ambient is Sump and everything visible is lit by something the player
can point at, with hard-edged light pools; (4) the line rule above; (5) the dither rule above; (6) every horizontal
surface takes a one-pixel highlight in the nearest light's hue — that pixel is what makes the sewer wet; (7) Vestment
White is the only pure white in the game.

Per-module plans
----------------

- Tiles: 47-tile blob autotile per material (concrete, brick, pipe, water, grate, collapse), cut from 64 px panels;
  parallax planes behind (cavern wall, distant tunnel) and in front (pipes, cables).
- Mat: cords 8–12 px wide, Ooze Dark core, Ooze Lit edge; the signal pulse is a shader running Ooze Glow outward from
  the Heart at 6 tiles per second; translucency is a dither of the tile beneath, never a blend; states fed / lean /
  starving; organ and class decorations as node overlays.
- Slimos: five depth bases x skin x inclusion overlay x vein overlay x gear layers. D4 bodies use the Ooze ramp with
  Chalk inclusions. Cycles: walk 8, idle 4, work 6, carry 6, dig 6, swing 4, aim 2, down 1.
- Ivan: 48 x 64, Ooze ramp at full saturation, Hot Pink rim light, four Authority states.
- Oathkeepers: 48 x 64, Vestment White with Reliquary Gold trim, bulky Y2K hardware, lamps on poles.
- Items: 32 x 32 icons on the 24 x 24 inner grid; tier shown by a one-pixel corner mark; condition by rust overlay bands
  below 60 and 20.
- UI: 9-slice frames whose background ends exactly at the frame line; two fonts (5 x 7 monospace for numbers and
  terminal text, 7 px-cap condensed grotesk for labels and narration — both Phase F, through the ui.font.mono slot;
  the shell runs on the tool's 3 x 5 sheet until then); Cyan numbers, Acid Yellow alerts, Hot Pink confirmations,
  Vestment White for anything Oathkeeper; labels slam in with a two-frame glitch.
- VFX: sparks, ooze drips, bleach foam, flame, bullet trails, dig dust, extraction ring; all two-tone, all emitters.
- Panels: hard ink, high contrast, text boxes in the grotesk; Oathkeeper panels get a two-frame glitch.

Visual progression
------------------

Every upgradable thing has visible states (Full Design v3, Visual progression). A state adds or removes shapes; a
palette swap is how states are prototyped in placeholders, never how they ship.

Pipeline
--------

1. Three style anchors (a culvert tile panel, a D2 slimo, a Rare chrome item) are generated in PixelLab with
   `generate_image_pixflux`, curated by hand and committed to game/assets/anchors.
2. Every later slot is generated with `generate_image_bitforge` against its anchor at style strength 70 or higher,
   from the brief in docs/03/pixellab-prompts.md (generated from the manifest; never typed).
3. `rose-tools validate` checks exact size, the palette lock (at most 2% off-palette pixels) and the outline rule
   (edge pixels Void or the ramp's darkest step); a failing file never ships.
4. The engine resolves `game/assets/art/<id>.png` when present, else the placeholder; placeholders retire slot by slot.

Placeholders
------------

One language, generated by `rose-tools placeholders` from the manifest: exact size, a flat fill in the slot's ramp,
a one-pixel dashed Void border, the slot's three-letter tag in the tool's pixel font, a silhouette hint (capsule for a
body, rectangle for an item, hatch for a tile, disc for an organ, strip for a cord or gauge, pile for rubble, frame
for a panel). World tiles carry the tag on their first variant only so a painted stratum stays readable. UI frames and
buttons are clean nine-patches with no tag, because anything in their centre is stretched. A placeholder obeys the grid
and the ramp rules, so a screen of placeholders already reads as one style; it is obviously unfinished because of the
dashes and the tag, never because it breaks the grid.
