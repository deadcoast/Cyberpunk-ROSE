Cyberpunk: ROSE — Rise of the Slimecity Empire
==============================================

An extraction colony-builder on a living purple ooze network under Night City. Design: `docs/FullDesign.md`.
Art: `docs/art-bible.md` (Sewer Noir 32). Tasks: `docs/todo.PACER.csv`. Status: Phase C, the city, closed (PAC-020–028, PAC-040–045 done); Phase D, pressure, is next.

    ./run.sh /path/to/godot4.5

builds the Rust core, runs its tests, regenerates placeholders and validates them, installs the GDExtension,
runs the Godot shell headless through all fourteen screens, and writes native screenshots to `docs/shots/`.

Layout
------

    Cargo.toml         workspace root
    core/crates/       mat · logistics · items · inventory · bench · people · colony · game · save · run · render · cli (`rose`)   — rules and numbers, no engine; `game` is the whole shift
    bridge/rose-godot  GDExtension: RoseSim, RoseItems, RoseStash, RoseRun, RosePeople                         — typed arrays only
    tools/             `rose-tools`: manifest -> placeholders, validate, prompts, contact sheet
    game/              Godot 4.5 project: Main, autoloads (Assets, Screens, Sim), ui/, modes/<screen>/, tests/
    game/assets/       palette/ manifest/ placeholders/ anchors/ art/   (read as files at runtime; .gdignore)
    data/              items.toml (164 items), chunks.toml (19 run chunks), loot.toml, creatures.toml, recipes.toml (60), traits.toml (40), names.toml, structures.toml (25), laws.toml (8), strings.toml
    docs/              design · art bible · research/ (01-17) · telemetry/ · pixellab-prompts.md · shots/ · todo.PACER.csv · VERIFICATION-*
    src/, history/     the Python prototypes and the M1 archive

Where things come from
----------------------

- A drawable in Godot is an `AssetSlot(id)`; the engine resolves `game/assets/art/<id>.png` when it exists and has
  passed `rose-tools validate`, else the placeholder the tool generated from `game/assets/manifest/*.toml`.
- Items, strings and (from Phase C) structures, laws, traits and barks are data files; balancing never recompiles.
- One PCG32 stream per simulation; a save is seed + inputs + snapshots and replays exactly.

Controls in the shell
---------------------

ESC menu · nav bar on every screen · Colony: LMB with no tool selects a slimo, a structure or a tile of Mat and opens its card above the tool bar (a structure's UPGRADE / CANCEL GHOST, a Pit's quota, a stockpile's PRIORITY −/+ and category filter, who works its slots; a slimo's job and needs, FOLLOW, SLIMO CARD); hover shows the same card small; RMB orders the selected slimo to the structure under the cursor for the next shift, else cancels the ghost or the cut there, and with a tool armed puts it down · the tool bar: PIT and RELAY arm Build with them, BAIT smears a Common Chrome from the Stash, SALT lays a tile from a bag of road salt (eight a bag; drag for a line), SEVER marks Mat to be cut by Build (drag; RMB unmarks), BUILD opens the Build menu, ZONE paints STOCKPILE / HAB / QUARANTINE / KILL by dragging a rectangle or ERASEs, OVERLAY cycles nutrient, signal, light, zones, traffic · R turns a footprint · the alert queue on the right edge (click to jump), the minimap bottom-right (click to go there) · SPACE pause, + / − speed (2, 4, 8 Hz), 1–9 return to a view and Ctrl + 1–9 set it, F follow the selected slimo, TAB view (Street / District / Stack; Stack frames the stratum as the circuit board with traffic as light, jams white), WASD pan, wheel zoom · Build: tabs HOMES / WORK / MAT / STORES / ORDER, each def's cost against what the Stash holds (Hot Pink where short), PLACE ON THE COLONY · Jobs: a row a slimo, click a cell to cycle its priority 0–4, drag to paint that value across, the foreman's mark where the district's stands over, a district filter · Slimo: the schedule row cycles Work / Rest / Any;
a D5's card can DISSOLVE it into the Heart · Charter: SIGN and REPEAL the eight laws with Authority; ANSWER or REFUSE a
district's petition (two refusals dissolve the district) · Stash: the tabs page between the Heart's grid, its floor (what the grid has no room for) and each stockpile; filter buttons select the view, SORT re-packs the grid; click an item for
its card and the four verbs (Sell to a grate market, Feed to a Pit, Craft to the Bench, Keep pins); SELL ALL / FEED
ALL act on the unpinned view; BACK pulls a queued item out; at the shift end after a run the queue leaves the Heart as
cargo and each item pays or feeds when it lands · Bench:
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

`rose city --shifts 20` plays the whole game headless — a bot run each shift, a steward's standing orders, the shift in `game` — twice, asserts that it replays bit-exact, that cargo lands within the next shift, that the build plan stands, that the cord to the first grate stands or a Relay can go out to win it back, that nobody starves and the Heart keeps its signal, and writes the merged run-and-city cadence to `docs/telemetry/city.md` (the full log to `out/city.log`). The game writes the city's log to `user://telemetry/city.log`.

`rose cadence --runs 20 --strata 1,2,3` plays seeded runs with the cautious bot (`run::bot`) and writes every run's cadence report and the aggregate against the design's budget to `docs/telemetry/cadence.md` (`--logs DIR` keeps each run's full log). Every run played in the game writes the same log to `user://telemetry/`.
