Asset manifest
==============

One TOML file per slot family. Every drawable in the game is a slot id; the engine resolves
`game/assets/art/<id>[.state].png` when it exists and has passed validation, else
`game/assets/placeholders/<id>[.state].png`, which `rose-tools placeholders` generates from these files.

Fields per `[[slot]]`:

    id          dotted id, e.g. "slimo.d2.walk"            (unique across all files)
    kind        tile | sprite | icon | ui | panel | vfx | anchor
    size        [w, h] of ONE frame in pixels
    frames      frames in the horizontal strip (1 = still)
    states      visual tiers or variants; one file per state, "" = single file
    ramp        noir | industrial | mud | bile | drain | flesh | ooze | neon | white  (placeholder fill, PixelLab hint)
    silhouette  capsule | rect | hatch | disc | frame | strip | pile   (placeholder hint)
    tag         3 characters stamped on the placeholder
    anchor      the style anchor PixelLab matches against (anchor.tile | anchor.slimo | anchor.item)
    prompt      PixelLab description; the style suffix is added by `rose-tools prompts`
    negative    PixelLab negative description
    status      placeholder | generated | approved
    expand      optional: "items" expands this slot into one icon per entry of data/items.toml

Art that lands in `art/` must pass `rose-tools validate`: exact size, SEEPAGE v2 palette (<= 2% off-palette),
and the 1 px Void outline on sprites and icons.
