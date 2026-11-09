Research 05 — PixelLab constraints
==================================

Source read: [pixellab-mcp on npm](https://npmjs.com/package/pixellab-mcp). Read 2026-11-07.

What the tool offers
--------------------

- Eight MCP tools: text-to-pixel-art (`generate_image_pixflux`), style-matched generation from a reference image
  (`generate_image_bitforge`, with a `style_strength` 0–100), `rotate` to another facing, `inpaint` with a mask,
  `estimate_skeleton` and `animate_with_skeleton`, `animate_with_text` (1–20 frames), and `get_balance`.
- Recommended sizes 32 / 64 / 128 / 256; options for outline, shading and detail level; no tileset tool.
- Configured as an MCP server with a secret; the game's client is Claude or Cursor, which means generation briefs are
  prose, and consistency across hundreds of assets comes from the reference image, not from the prompt.

Decisions this changed in the design
------------------------------------

1. **Three style anchors are the first assets** and every later generation uses `generate_image_bitforge` against one
   of them at style strength 70 or higher. Coherence is enforced by the anchor, then checked by the validator.
2. **The tile grid is 32 px** because the tool's native sizes start there; 16-px tiles would be downscaled AI output,
   which loses the pixel discipline the style depends on.
3. **Tiles are generated as 64 × 64 panels and cut** by `rose-tools cut` (Phase F) because there is no tileset tool;
   autotile sets are assembled from cut quarters plus hand-fixed corners.
4. **Animations go through the skeleton path** for people (eight-frame walks) and the text path for organs and VFX
   (four to six frames), and every frame passes the palette lock individually.
5. **The manifest carries the prompt and the negative** so the brief for every slot is generated, not typed, and the
   prompt sheet (`docs/pixellab-prompts.md`) is the hand-off to the artist.
