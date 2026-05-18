# Multi-Library Hardening Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
cargo nextest run -p taru-server startup --no-fail-fast
```

This should capture configured-library startup behavior before reconciliation
semantics change.

## Gate Set

### Characterization Gate

```powershell
cargo check -p taru-server --tests
cargo nextest run -p taru-server startup --no-fail-fast
```

Proves startup/config behavior is visible before code movement.

### Reconciliation Gate

```powershell
cargo check -p taru-server --tests
cargo check -p taru-db --tests
cargo nextest run -p taru-server <reconciliation-filter> --no-fail-fast
```

Proves the server and SQLite adapter agree on Library desired-state
reconciliation.

### Closeout Gate

```powershell
cargo fmt --all -- --check
cargo nextest run -p taru-server --no-fail-fast
git diff --check
```

Broaden to workspace gates if the implementation changes shared repository
traits or public API shapes.

### Review Gate

Run `review-workstream` before accepting MLH-030 and before closeout. Record
blocking findings, missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md`
- `crates/taru-server/src/config.rs`
- `crates/taru-server/src/app/startup.rs`
- `crates/taru-server/src/app/jobs.rs`
- `crates/taru-server/src/app/library.rs`
- `crates/taru-server/src/app/metadata.rs`
- `crates/taru-db/src/library.rs`
- `crates/taru-core/src/media/library.rs`

## Fresh Evidence

2026-05-18, MLH-010:

- Historical M8 multi-library decisions promoted into standard workstream docs.
- First executable task set to startup/config behavior characterization.
- Public Client Source Locator redaction and Library Access are explicit
  non-goals for this lane.

Fresh verification is required before marking implementation tasks or the lane
complete.
