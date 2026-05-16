# Crate Boundary and Public Protocol Hardening Milestones

Status: Completed
Last updated: 2026-05-17

## M28.0: Boundary Baseline And Scope Freeze

Status: completed.

Outcome: the crate and protocol seams are documented before code moves.

Deliverables:

- Boundary audit of `taru-api`, `taru-core`, `taru-library`, `taru-nfo`,
  `taru-streaming`, `taru-transcode`, and `taru-server` playback surfaces.
- Clear split between public client wire types and server adapter DTOs.
- Explicit first proof slice for the public protocol boundary.
- Workstream docs, phase note, and goal-map entry aligned.

Exit criteria:

- `docs/GOALS.md` names the active M28 goal.
- `docs/ROADMAP.md` names the M28 workstream.
- `docs/README.md` and `docs/workstreams/README.md` point at this lane.
- `git diff --check`

Primary evidence:

- `docs/workstreams/crate-boundary-hardening/DESIGN.md`
- `docs/workstreams/crate-boundary-hardening/TODO.md`
- `docs/workstreams/crate-boundary-hardening/PHASE28_0_CRATE_BOUNDARY_BASELINE.md`

## M28.1: Public Client Protocol Extraction

Status: completed for the first public system-envelope slice.

Outcome: public client wire types live in a permissive protocol crate, and
`taru-api` becomes a server adapter over that boundary.

Deliverables:

- Create or solidify `taru-client-protocol` as the permissive public wire
  boundary.
- Move the first stable public response/request types out of `taru-api`.
- Keep `taru-api` as the AGPL mapping layer for server internals.
- Add dependency-direction checks that the public protocol crate does not
  import server internals.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-api`
- focused `cargo nextest run -p taru-client-protocol`
- `git diff --check`

Primary evidence:

- `crates/taru-api`
- `crates/taru-client-protocol`
- `cargo tree -p taru-client-protocol` shows no server-internal dependency.

## M28.2: Core Module Deepening

Status: completed for media and repository module deepening; repository trait
narrowing remains a future behavior-preserving follow-on.

Outcome: `taru-core` becomes easier to navigate because media and repository
concepts are grouped by concept instead of by historical accretion.

Deliverables:

- Split `crates/taru-core/src/media.rs` into concept-sized module files.
- Split `crates/taru-core/src/repository.rs` into narrower trait groupings if
  it improves locality.
- Keep repository behavior stable while the file layout changes.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-core`
- focused `cargo nextest run -p taru-db`
- `git diff --check`

Primary evidence:

- `crates/taru-core/src/media.rs`
- `crates/taru-core/src/repository.rs`
- `crates/taru-core/src/media/*`
- `crates/taru-core/src/repository/*`

## M28.3: Library And NFO Decomposition

Status: completed.

Outcome: `taru-library` and `taru-nfo` expose clearer workflow modules for
scan, index, probe, codec, import, export, and orchestration code.

Deliverables:

- Split `crates/taru-library/src/lib.rs` into scan, index, probe, local
  inference, and summary modules.
- Split `crates/taru-nfo/src/lib.rs` into codec and workflow modules.
- Keep public behavior stable while the internal shape becomes shallower.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-library`
- focused `cargo nextest run -p taru-nfo`
- `git diff --check`

Primary evidence:

- `crates/taru-library/src/lib.rs`
- `crates/taru-nfo/src/lib.rs`
- `crates/taru-library/src/{summary,scan,index,probe,local_inference,failure}.rs`
- `crates/taru-nfo/src/{codec,summary,workflow,import,export}.rs`

## M28.4: Playback Seam Clarification

Status: completed.

Outcome: playback planning, runtime, and server orchestration have explicit
responsibility lines.

Deliverables:

- Clarify what belongs in `taru-streaming` versus `taru-transcode`.
- Keep `taru-server::app::playback` as orchestration and HTTP translation.
- Remove any leftover playback helper code that only exists because the old
  seam was blurry.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-streaming`
- focused `cargo nextest run -p taru-transcode`
- focused `cargo nextest run -p taru-server`
- `git diff --check`

Primary evidence:

- `crates/taru-streaming/src/lib.rs`
- `crates/taru-transcode/src/lib.rs`
- `crates/taru-server/src/app/playback/*`
- `crates/taru-streaming/src/{selection,direct}.rs`
- `crates/taru-transcode/src/{plan,hardware,ffmpeg,session,runtime,remux,hls,runner_util}.rs`

## M28.5: Closeout And Follow-On Split

Status: completed.

Outcome: the lane closes with a durable record of what changed and what should
remain separate later.

Deliverables:

- Final close-out note mapping M28 goals to code, tests, and docs.
- Updated workstream docs for any follow-on lane that emerges from the public
  protocol boundary.
- Recorded validation evidence for the full crate-boundary pass.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`
