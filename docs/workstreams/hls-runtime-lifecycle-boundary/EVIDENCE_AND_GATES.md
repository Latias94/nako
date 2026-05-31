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

Status: Pending

Evidence to collect:

- lifecycle state and transition table;
- readiness and segment wait semantics;
- cleanup ownership map;
- test coverage map;
- follow-on split decision for artifact I/O pressure and resource admission.

## Residual Risks

- HLS lifecycle, artifact I/O pressure, and storage health can easily overlap.
  Keep implementation tasks serialized when they touch `resource.rs` or
  `hls_artifact.rs`.
- Do not let this lane become a catch-all for LL-HLS, DASH/CMAF, remote
  workers, hardware policy, or player UX.
