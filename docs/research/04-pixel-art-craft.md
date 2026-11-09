Research 04 — pixel-art craft: outlines, ramps, clusters
=======================================================

Sources read: [Lospec, Pixel art outlines part 2](https://lospec.com/articles/pixel-art-outlines-part-2-using-color);
[Slynyrd, Pixelblog 1: colour palettes](https://www.slynyrd.com/blog/2018/1/10/pixelblog-1-color-palettes). Read 2026-11-07.

What the craft says
-------------------

- An outline exists to separate the object from what is behind it; it must be darker than both the object and the
  background, and any two touching colours must be tellable apart at 1×.
- Selective outlining varies the outline shade around the form (darker in shadow, lighter in light) and breaks the line
  where a form turns into shadow; black is the fallback when no single shade works on a dark scene.
- Internal lines use the shade one step darker than the area; lines thinner than one pixel are implied by broken
  segments near shadows.
- Ramps are hue-shifted (shadows toward a cool hue, lights toward a warm or emitter hue), few steps, and the palette
  is small enough to hold in the head.

Decisions this changed in the design
------------------------------------

1. **Void is the outline fallback, not the rule.** The art bible's "1 px Void selective outline" now reads: outline in
   the darkest step of the object's own ramp, stepping to Void where the background is dark (which in a sewer is most
   of the time), broken where a form turns into shadow. The validator checks that edge pixels are Void *or* the
   ramp's darkest step; pure Void everywhere was over-specified.
2. **Tiles still carry no outline**, which this reading supports: a tile is background, and an outline on the
   background separates nothing.
3. **Touching-colour test added to the validator plan** (Phase F): adjacent palette steps must differ in luminance
   by a threshold at 1× or the pass fails.
4. **Internal lines are one step darker than their area**, written into the per-module plan for structures and items.
5. **Ooze translucency is a dither of the tile beneath**, never a blend: a blend would introduce off-palette colour and
   fail the lock, and the dither reads as translucent at 3× scale.
