Cyberpunk: ROSE — engine (M1)
=============================

The core is Rust; the shell is Godot 4.5. Godot owns the clock, camera and picture; `matsim` owns every number.

    engine/
      crates/matsim        the Mat CA + nutrient field, flow fields, cargo + agent columns, PNG renders
      crates/matsim-cli    `matsim grow | evidence | scale | bench`  (headless, designed terminal output)
      crates/rose-godot    GDExtension: class `RoseSim`  (start_scale, step, feed, sever, salt, bait, place_relay, arrays, stats)
      godot/               greybox project: Main.tscn, Main.gd (clock, input, camera), World.gd (image-texture renderer, glyph LOD)
      godot/tests/smoke.gd headless scene test: 3,000 slimos, every verb, view cycling, budget check
      run.sh               the one command

One command
-----------

    ./run.sh /path/to/godot4.5

Builds release, runs the 12 core tests, builds and installs the extension, writes `out/evidence_seed11.png`,
`out/scale_stack_seed11.png`, `out/scale_street_seed11.png`, then runs the Godot smoke test headless.
Without a Godot path it does everything except the smoke test.

Play the greybox
----------------

Open `godot/` in Godot 4.5 (or run `godot --path godot`). Controls: LMB feed 5 units, RMB sever, Shift+S salt tool,
B bait, R relay, F feed, Space pause, 1/2/3 = 1/2/4 Hz, Tab cycles Street / District / Stack view, WASD pans, wheel zooms.

Numbers (release, one thread, 192x48 stratum)
----------------------------------------------

    CA only                      0.3-0.5 ms / tick
    CA + 3,000 slimos + cargo    0.7 ms / tick
    CA + 20,000 slimos + cargo   1.0 ms / tick
    Godot headless, 3,000 slimos, full step through the extension    ~0.7 ms

Determinism: one PCG32 stream per sim; (seed, inputs) replays a run exactly (tested).
