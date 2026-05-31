# HLS Runtime Lifecycle Boundary - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

### HRLB-010 - Lifecycle invariant freeze

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

`HRLB-010` is docs/research-only. Do not run Rust gates unless code changes are
explicitly approved.

### HRLB-020 - Behavior-preserving lifecycle boundary

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Broaden to playback/session or storage gates only if the task scope is expanded
by the planner.

## Evidence Ledger

### HRLB-010 - Lifecycle invariant freeze

Status: Done with concerns

Evidence collected:

- lifecycle state and transition table;
- readiness and segment wait semantics;
- cleanup ownership map;
- test coverage map;
- follow-on split decision for artifact I/O pressure and resource admission.

Notes:

- `DESIGN.md` now freezes active same-generation request handling, finished
  session reuse, different-generation supersede, running playlist readiness,
  segment readiness and one-shot wait, cancellation/timeout cleanup, startup
  stale-session cleanup, terminal artifact cleanup, staging input release, and
  PAIP artifact I/O pressure split guidance.
- Artifact I/O pressure should be split into a PAIP follow-on, using the
  existing `proposed:hls-artifact-io-pressure-enforcement` lane name unless the
  planner chooses a different slug. It should not be implemented inside
  `HRLB-020`.
- Coverage concerns for `HRLB-020`: focused HLS timeout cleanup, HLS-specific
  startup stale-session recovery, and HLS remote staged-input lease release are
  not yet directly covered even though adjacent generic/runner/lease tests
  exist.

Fresh validation:

```text
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/hls-runtime-lifecycle-boundary docs/architecture/PLAYBACK.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Result: passed on 2026-05-31. `git diff --check` emitted only existing Git
line-ending normalization warnings for touched Markdown/JSON files and no
whitespace errors.

## Residual Risks

- HLS lifecycle, artifact I/O pressure, and storage health can easily overlap.
  Keep implementation tasks serialized when they touch `resource.rs` or
  `hls_artifact.rs`.
- Do not let this lane become a catch-all for LL-HLS, DASH/CMAF, remote
  workers, hardware policy, or player UX.
