AGENTS — working in Cyberpunk: ROSE
==================================

For every agent that opens this repository: the backend agent (`@claude`) and the art agent (`@pixel-lab-agent`).
Read this file, then the register's open rows, then the document your row points at. Nothing here replaces the
design; it tells you where the design is and how work moves through the register.

Where the truth lives
---------------------

| file | what it is | who writes it |
| --- | --- | --- |
| `docs/03/FullDesign.md` | the design, one document; every decision a stage makes is written back here; anything the user's passes do not specify is marked "this design's" | backend agent, the user |
| `todo.PACER.csv` | the task register (spec below) | every agent |
| `overlord.PACER.csv` | the plan for the register's open rows: stage by stage, each step mapped onto the files it touches, what it stands on, what it hands on and its check (spec below) | backend agent |
| `changelog.PACER.csv` | one row a commit: date, commit, author, PAC, kind, scope, what changed, the evidence (spec below) | every agent |
| `docs/03/CURRENT-STATE.md` | the latest review: the build measured, the design against the build, the weak points with IDs | backend agent |
| `docs/03/VERIFICATION-N.md` | evidence logs, one section per PAC as it closes | backend agent |
| `docs/03/art-bible.md` | Sewer Noir 32: grid, scales, line, value, ramps, per-module plans | the user, backend agent |
| `docs/03/pixellab-prompts.md` | per-slot generation briefs, **generated** from the manifest | `rose-tools prompts` |
| `docs/03/telemetry/*.md` | the proofs' reports, **generated** | `rose cadence`, `city`, `cleansing`, `economy`, `campaign` |
| `docs/research/NN-*.md` | research written before each stage's design freeze; each ends with the decisions it changed | backend agent |
| `docs/01/`, `docs/02/` | Pass 1 (superseded) and Pass 2 (the pillars); reference, never edited | the user |
| `game/assets/manifest/*.toml` | every drawable as a slot id; the art agent's work list | both lanes (fields below) |
| `assets/shots/` | native screenshots, **generated** by `game/tests/shots.gd` and `play.gd` | the test scripts |

The verification logs by number: 1 Phase 1 (the M1 engine, the old `docs/` copy), 2 Phase A, 3 Phase B, 4 Phase C,
5 Phase D, 6 Phase E, 7 Phase 1 (the old root copy), 8 Phase G. A phase that opens later takes the next free number.

Never hand-edit a generated file: change its source (the manifest, the proof, the script) and regenerate. The
generators write the house style: tables with `| --- |` rows, and in the prompts file each slot as its id over a
`---` line.

The register: `todo.PACER.csv`
------------------------------

PACER (Project Actions, Constraints & Evidence Register) is one CSV, one row a task, sixteen columns:

| column | how this repository fills it |
| --- | --- |
| ID | `PAC-NNN`, in blocks by phase: A 001–014, B 020–028, C 040–045, D 060–063, E 080–083, F 090–093, G 100– |
| Title | what the row delivers, in a line |
| Phase | `A Foundations`, `B The run`, `C The city`, `D Pressure`, `E Progression`, `F Art`, `G Current state` |
| Status | `TODO` → `DOING` → `REVIEW` → `DONE` (`DOING` → `TODO` rolls back, `REVIEW` → `DOING` sends it back) |
| DoD | the definition of done: observable, measurable, nothing a reader has to take on trust |
| BlockedBy | comma-separated IDs; a row goes `DONE` only when every hard blocker is `DONE` |
| Assignee | `@claude` (backend) or `@pixel-lab-agent` (art) |
| StartedAt, DoneAt | UTC, ISO 8601 (`2026-11-13T20:38:00Z`); rows before PAC-100 carry dates only |
| Notes | what changed, with the constants and modules named |
| Context | why the row exists: the finding, the design section, the CURRENT-STATE ID |
| Instructions | the order to work in, where it matters |
| ExpectedOutput | the files or results the row produces |
| ValidationCriteria | the check, and the verification entry that holds its evidence |
| DependencyType | `hard` (gates `DONE`) or `soft` (order only) |
| Priority | `high`, `medium`, `low` |

Rules:

- **One thread.** A phase is one coherent piece of work with its tasks inside it, never a pile of independent tasks;
  finish a phase's foundation row before the rows that stand on it.
- **Evidence, not assertion.** A row is `DONE` when its DoD is met and the numbers are in a verification entry: a
  proof's report, a test, a screenshot looked at, a measurement. Say what was not checked.
- **The design moves with the code.** A rule changed in code is changed in `FullDesign.md` in the same commit, with
  the measurement that justified it; this design's inventions are marked as such.
- **Verify in your own runtime.** Build, run the proofs, the smoke and the screenshots yourself; never park a test
  for the user.
- **Quote every field that holds a comma** (PAC-063 was split across two extra columns by one unquoted DoD).
  Edit the CSV with a CSV writer, then check that every row has sixteen fields and every blocker exists.

The plan and the changelog
--------------------------

`overlord.PACER.csv`, fourteen columns:

| column | how this repository fills it |
| --- | --- |
| ID | `OVR-NNN`, in blocks by stage: S0 001–009, S1 010–019 … S10 100–109, the lanes and rules 110– |
| PAC | the register rows the step serves |
| Stage | `S0 Map and baseline` to `S10 Code health`, worked in order; the stage order is the plan's, the register's BlockedBy still gates `DONE` |
| Kind | `map` (how the code connects today), `step`, `gate` (the step that closes a PAC), `rule` |
| Title, Status | as the register: `TODO` → `DOING` → `REVIEW` → `DONE` |
| Approach | the mechanism, with the constants and functions named |
| Touches | the files, with lines where they matter |
| Imports, Exports | what the step stands on; what later steps take from it |
| BlockedBy | OVR IDs |
| Check, Evidence | the command or test that proves the step; the verification entry that holds its numbers |
| Origin | the CURRENT-STATE ID or design section it answers; "this plan's invention" where neither specifies it |

`changelog.PACER.csv`, twelve columns: `ID` (`CHG-NNN`), `Date` (UTC, the commit's), `Commit` (short hash),
`Author`, `PAC`, `Kind` (feature, fix, docs, structure, assets, review, plan, revert, merge, config, prototype),
`Scope` (the crates and folders touched), `Summary` (the commit's subject), `Files`, `Delta` (+insertions
−deletions), `Evidence`, `Notes` (what the subject leaves out). Every commit adds its row in the same commit; a row
cannot hold its own hash, so its `Commit` reads `pending` until the next commit fills it in.

A step found along the way becomes an overlord row before it becomes code. Both files are edited with a CSV writer
and checked like the register: every row its field count, every ID unique, every blocker present.

The backend lane (`@claude`)
----------------------------

Owns `core/`, `bridge/`, `tools/`, `game/` (scripts and scenes), `data/`, the docs above it writes, and every slot's
`id`, `kind`, `size`, `frames`, `states` and `expand`. Every rule and every number runs in Rust; Godot reads arrays
and draws. A drawable is an `AssetSlot(id)` with a manifest row and a placeholder before any art exists; a new visual
that is drawn as rectangles has no way to receive art (CURRENT-STATE ART-01, PAC-109).

    cargo build --release && cp target/release/librose_godot.so game/bin/      the core, the bridge, the tools
    cargo test --release --workspace --exclude rose-godot                         192 tests at PAC-100
    ./target/release/rose --plain city | cleansing | economy | campaign           the proofs (campaign: ~25 min)
    godot --headless --path game --script res://tests/smoke.gd                   every screen, headless
    xvfb-run -a godot --path game --rendering-driver opengl3 --script res://tests/shots.gd   every screen to assets/shots/
    xvfb-run -a godot --path game --rendering-driver opengl3 --script res://tests/play.gd    a scripted session, review by eye
    ./run.sh /path/to/godot                                                       all of it

Commit to `main` with a message that names the PAC and what changed; rebase on `origin/main` before pushing. Before
every commit check that no secret is staged: `git diff --cached | grep -cE "pixl_[0-9a-f]{8}"` must print 0 (the PixelLab token
belongs in the agent's MCP configuration, never in the repository).

The art lane (`@pixel-lab-agent`)
---------------------------------

Owns `game/assets/art/`, `game/assets/anchors/`, and in the manifest each slot's `prompt`, `negative` and `status`.
Never edits Rust, GDScript or `data/`; a visual that needs a new slot, size or state is a register row for the
backend lane.

Read first: `docs/03/art-bible.md` (Sewer Noir 32), `docs/research/04-pixel-art-craft.md`,
`docs/research/05-pixellab-constraints.md`, `docs/03/pixellab-prompts.md`, `game/assets/manifest/README.md`.

The order, by register row:

1. **PAC-090, the style anchors**: `anchor.tile` (64 × 64), `anchor.slimo` (32 × 48) and `anchor.item` (32 × 32) into
   `game/assets/anchors/`; every later slot is generated against its anchor; the user approves the three by eye.
2. **PAC-091, the manifest**: slot family by family; one file a state, `game/assets/art/<id>[.<state>].png`; a
   strip of frames runs left to right at the slot's frame size; item icons expand to `icon.item.<item id>`.
3. **PAC-092, the 5 × 7 font** through the `ui.font.mono` slot.
4. PAC-093 (shaders, parallax, water, lighting, palettes) is code on top of the art and stays with the backend lane.

Every PNG must pass `rose-tools validate`: the exact size, the SEEPAGE v2 palette (`game/assets/palette/SEEPAGE_v2.gpl`,
at most 2 % of pixels off it), and the one-pixel Void outline on sprites, props and icons. Then set the slot's `status`
from `placeholder` to `generated`; the user's approval makes it `approved`. Run `rose-tools prompts` after editing a
prompt and `rose-tools sheet` to see the slot beside its neighbours. The engine draws any PNG it finds in `art/` in
place of the placeholder, with no code change and without checking it (`game/autoload/Assets.gd:20`): never leave a
file there that has not passed `validate`.

Rules the art bible sets and the user holds hard: one pixel-art style on every screen, placeholders included; the ooze
is purple and pink, never green; no smooth gradients, blur or anti-aliasing; light comes from emitters.
