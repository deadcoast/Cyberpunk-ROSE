# VERIFICATION — Phase G (from the current state), running log

One section per PAC as it closes. Every claim here was produced by a command in this repo; re-run with
`./run.sh /path/to/godot`.

## PAC-100 — the current state reviewed, the agent spec written, the moved docs linked (2026-11-13)

**The review**: `docs/03/CURRENT-STATE.md`. Its numbers come from `cargo test`, the four proofs as run at PAC-083 on
the same code (`docs/03/telemetry/`), `rose bench`, a shift-by-shift timing of `rose campaign --aim world_eater`
(0.31 s of wall time for shift 10 at 26 slimos; 3.32 s for shift 110 at 1,726 slimos and 18,511 joined cells, a
5.3-minute city shift), `game/tests/shots.gd` and `play.gd` under Xvfb (23 screens and 34 frames, rewritten into
`assets/shots/`), and the source at the lines it cites. Two claims were corrected before it was written down: the
run's key line fades after its first seconds (`Run.gd:422`), so it is not listed; and the "loss lapses" verdicts are
quiet stretches past two minutes at a player's pace, so Pillar 5 is partial, not met.

**The agent spec**: `AGENTS.md` at the root: where the truth lives, the verification logs by number, the register's
sixteen columns as this repository fills them, the rules, and the two lanes. The art lane's facts were read from the
code: anchors at 64 × 64, 32 × 48 and 32 × 32 (`game/assets/manifest/anchors.toml`), the `ui.font.mono` slot
(`ui.toml:74`), the validator's checks (`game/assets/manifest/README.md`), and that `Assets.gd` draws any PNG in
`art/` without checking it — which the manifest README did not say, now CURRENT-STATE ART-04.

**The register**: Phase G added, PAC-100 to PAC-110, each row's Context naming the CURRENT-STATE IDs it answers;
PAC-090 to PAC-092 assigned to `@pixel-lab-agent` with `AGENTS.md` as their contract; PAC-093's note says what
stays code. PAC-063 repaired: its DoD's commas were unquoted when it was written (`d5f24e2`), which split it into 18
fields, and its DONE edit (`2091d5a`) wrote the dates over BlockedBy and Assignee; it now reads as first written with
the done dates and notes kept. Every row has sixteen fields, every ID is unique and every blocker exists (checked with
Python's `csv`). The bundled PACER validator also accepts the new rows' timestamps; it rejects every phase name,
because its list is another project's.

**The moved docs**: the user moved the design, logs and telemetry into `docs/03/` and the screenshots into
`assets/shots/`. Every reference now points there: the README, the design, the art bible, research 05, the
verification logs, the register, `run.sh`, the `rose` report defaults, `shots.gd`, `play.gd` and `rose-tools`.
`grep` for `docs/telemetry`, `docs/shots`, `docs/pixellab-prompts`, `docs/FullDesign`, `docs/art-bible` and
`VERIFICATION-phase` outside `target/` and `out/` finds nothing.

**The generators keep the hand edits**: `rose-tools prompts` writes `docs/03/pixellab-prompts.md` with each slot as
its id over a `---` line, the format the file was hand-edited to; regenerated, the file is byte-identical to the
committed one (`diff`, no output). The proofs write their tables with `| --- |` rows into `docs/03/telemetry/`;
`rose city` regenerated `city.md` identical to the hand-edited one, and no `docs/telemetry/` was created.

**Still green**: `cargo test --release --workspace --exclude rose-godot` 191 passed, 0 failed, 1 ignored; `rose city`
PASS; the headless smoke PASS; `shots.gd` wrote 23 screens and `play.gd` 34 frames to `assets/shots/` and reached
its ending (Hive, warchest 743).
