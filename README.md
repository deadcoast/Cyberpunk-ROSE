Cyberpunk: ROSE — Rise of the Slimecity Empire
==============================================

An extraction colony-builder on a living purple ooze network under Night City. Design: `docs/FullDesign.md`.
Art: `docs/art-bible.md` (Sewer Noir 32). Tasks: `docs/todo.PACER.csv`. Status: Phase C, the city (PAC-020–028 and PAC-040 done).

    ./run.sh /path/to/godot4.5

builds the Rust core, runs its tests, regenerates placeholders and validates them, installs the GDExtension,
runs the Godot shell headless through all fourteen screens, and writes native screenshots to `docs/shots/`.

Layout
------

    Cargo.toml         workspace root
    core/crates/       mat · logistics · items · inventory · bench · people · save · run · render · cli (`rose`)   — rules and numbers, no engine
    bridge/rose-godot  GDExtension: RoseSim, RoseItems, RoseStash, RoseRun, RosePeople                         — typed arrays only
    tools/             `rose-tools`: manifest -> placeholders, validate, prompts, contact sheet
    game/              Godot 4.5 project: Main, autoloads (Assets, Screens, Sim), ui/, modes/<screen>/, tests/
    game/assets/       palette/ manifest/ placeholders/ anchors/ art/   (read as files at runtime; .gdignore)
    data/              items.toml (164 items), chunks.toml (19 run chunks), loot.toml, creatures.toml, recipes.toml (60), traits.toml (40), names.toml, strings.toml
    docs/              design · art bible · research/ (01-13) · telemetry/ · pixellab-prompts.md · shots/ · todo.PACER.csv · VERIFICATION-*
    src/, history/     the Python prototypes and the M1 archive

Where things come from
----------------------

- A drawable in Godot is an `AssetSlot(id)`; the engine resolves `game/assets/art/<id>.png` when it exists and has
  passed `rose-tools validate`, else the placeholder the tool generated from `game/assets/manifest/*.toml`.
- Items, strings and (from Phase C) structures, laws, traits and barks are data files; balancing never recompiles.
- One PCG32 stream per simulation; a save is seed + inputs + snapshots and replays exactly.

Controls in the shell
---------------------

ESC menu · nav bar on every screen · Colony: LMB apply tool, TAB view (Street / District / Stack), SPACE pause,
1/2/3 speed, WASD pan, wheel zoom · Stash: filter buttons select the view, SORT re-packs the grid; click an item for
its card and the four verbs (Sell to a grate market, Feed to a Pit, Craft to the Bench, Keep pins); SELL ALL / FEED
ALL act on the unpinned view; BACK pulls a queued item out; queues resolve when the shift ends after a run · Bench:
pick a recipe, click a slot then a Stash item that carries its tag, CRAFT queues (parts leave the Stash), CANCEL
returns them; jobs finish at shift end · Slimo: the roster left, click a name for its record; click a job cell to
cycle its priority 0–4; MEMORIAL for the dead; SEND ON A RUN · Route
Board: click a card or 1–3 to pick a route, click a crew name to make them the Runner and COMES ALONG / STAYS HOME
for up to two followers (the roster's eligible slimos), click Stash items into the hand, quick slots and pack,
DEPART or ENTER · Run: WASD move, SHIFT sprint, CTRL sneak, Q (hold) dim veins, SPACE dodge, J swing, K shoot, F + direction dig,
E open a source (pry bar when carried) or hold E on the Mat or under a grate to extract, R take all, V drop the last
thing taken, 1–4 use a quick slot (antiseptic, stim, salt line), Z/X/C order followers (follow / hold / guard), L lantern,
TAB stash. The run is `run::sim::RunSim` in Rust at 20 Hz; the screen
is its view. `game/tests/play.gd` plays a scripted run under Xvfb and saves review frames to docs/shots/play_*.png.

`rose route --seed N --stratum S [--sweep 200]` assembles a run route from `data/chunks.toml`, renders it and its
light to `out/`, reports the authored light bands, a lantern-on / dimmed sight and exposure comparison and a sprint
step's noise footprint, with `--sweep` chunk, container and spawn averages and the worst connectivity per stratum, and
with `--loot N` opens every container on N seeds per stratum and reports value per container, tier mix, notable rate and
the stratum-to-stratum ratio.

`rose cadence --runs 20 --strata 1,2,3` plays seeded runs with the cautious bot (`run::bot`) and writes every run's cadence report and the aggregate against the design's budget to `docs/telemetry/cadence.md` (`--logs DIR` keeps each run's full log). Every run played in the game writes the same log to `user://telemetry/`.
