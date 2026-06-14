# Research: control-plane seam deepening

## Selected Candidate

The highest-leverage next deepening opportunity is the `nako-server` startup/composition/runtime seam:

* `crates/nako-server/src/app/composition.rs`
* `crates/nako-server/src/app/startup.rs`
* `crates/nako-server/src/app/runtime.rs`

## Why This Candidate Wins

* The server control plane already owns durable jobs, runtime supervision, diagnostics, and operator-facing startup behavior.
* The current code base has a broad `NakoApp` composition root plus a separate startup layer, but the workflow still reads like a single construction chain.
* This is a high-leverage seam: improving it should make future diagnostics, recovery, and job supervision easier without reopening unrelated playback or library boundaries.
* Playback and library intake already have deeper focused seams than the startup/control-plane path, so they are not the first place to spend the next refactor budget.

## Relevant References

* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/LANES.md`
* `docs/architecture/PLAYBACK.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `crates/nako-server/src/app.rs`
* `crates/nako-server/src/app/composition.rs`
* `crates/nako-server/src/app/startup.rs`
* `crates/nako-server/src/app/runtime.rs`

## Recommendation

Start with a control-plane seam deepening task scoped to startup, composition, and runtime supervision only. Preserve behavior, narrow ownership, and let tests prove the seam got deeper instead of just moving code around.

