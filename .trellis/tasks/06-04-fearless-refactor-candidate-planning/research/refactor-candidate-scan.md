# Research: fearless refactor candidate scan

- Query: identify a feature-backed refactor candidate after the bounded HLS
  start admission slice.
- Scope: local repo/docs scan
- Date: 2026-06-04

## Findings

* The main worktree is clean and `main` is ahead after the HLS start admission
  feature, archive, and journal commits.
* Current Trellis task is empty; the long-horizon architecture queue remains
  active but has no child tasks.
* `docs/architecture/LANES.md` lists playback-transcode, storage-vfs,
  web-product, and control-plane as idle areas with focused follow-ons.
* `crates/nako-server` is the largest crate in the repo. Within playback,
  `app/playback/mod.rs` remains the broadest app root file and still owns Remux
  lifecycle logic directly.
* HLS lifecycle has already been moved behind `hls_flow.rs`/`hls.rs`, and the
  latest HLS resource-admission work strengthened that boundary.
* A Remux lifecycle extraction would align Remux with the HLS boundary while
  preserving existing `PlaybackAppService` entry points.

## Candidate Comparison

### Candidate A: Playback Remux lifecycle extraction (recommended)

* Intent: reduce playback root orchestration complexity and create symmetry
  with HLS lifecycle ownership.
* Expected write scope: `crates/nako-server/src/app/playback/mod.rs`, a new or
  existing `app/playback/remux_flow.rs`, focused playback tests, and possibly
  server backend spec updates.
* Benefits: directly lowers future resource-admission/staging/runtime change
  cost; does not require public API/schema changes.
* Risk: Remux playback-session linkage and source output waits are coupled to
  the app root, so tests must protect behavior.

### Candidate B: Playback session/direct transport split

* Intent: shrink the playback root by separating session/direct-streaming
  entrypoints.
* Benefits: lower immediate implementation risk.
* Risk: less feature-backed by the current resource/runtime roadmap.

### Candidate C: Storage VFS cache repair execution

* Intent: continue from preview-only VFS repair actions toward executable repair.
* Benefits: product-visible operational hardening.
* Risk: this is feature work spanning VFS/API/Admin more than a refactor, and
  should be its own task.

### Candidate D: Provider/Web governance follow-on

* Intent: continue metadata governance/API/Admin work.
* Benefits: product feature progress.
* Risk: cross-layer API/Web work is not the right first fearless refactor lane.

## Recommendation

Prepare the first fearless refactor implementation task around **Playback Remux
lifecycle extraction**. Keep it behavior-preserving and module-deepening:

* no new crate;
* no public API/schema changes;
* no new abstract trait until multiple callers justify it;
* preserve all Remux/HLS/Direct Play behavior with focused tests;
* update specs only if the Remux lifecycle boundary becomes durable guidance.
