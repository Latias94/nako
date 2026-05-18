# Metadata Merge Policy Unification Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
cargo nextest run -p taru-metadata merge --no-fail-fast
cargo nextest run -p taru-nfo nfo_service --no-fail-fast
```

These gates should capture the current duplicated provider and NFO merge
behavior before code moves.

## Gate Set

### Characterization Gate

```powershell
cargo nextest run -p taru-metadata merge --no-fail-fast
cargo nextest run -p taru-nfo nfo_service --no-fail-fast
```

Proves provider and NFO expectations are test-visible before refactor.

### Shared Boundary Gate

```powershell
cargo check -p taru-core --tests
cargo check -p taru-metadata --tests
cargo check -p taru-nfo --tests
cargo nextest run -p taru-metadata merge --no-fail-fast
cargo nextest run -p taru-nfo nfo_service --no-fail-fast
```

Proves the shared policy compiles across the dependency boundary and that both
callers preserve behavior.

### Closeout Gate

```powershell
cargo fmt --all -- --check
cargo nextest run -p taru-metadata --no-fail-fast
cargo nextest run -p taru-nfo --no-fail-fast
git diff --check
```

Broaden to `cargo check --workspace --tests` or `cargo nextest run --workspace
--no-fail-fast` if the implementation touches shared `taru-core` APIs beyond
metadata policy types.

### Review Gate

Run `review-workstream` before accepting MMP-030 and again before closeout.
Record blocking findings, missing gates, and residual risks here or link to the
review note.

## Evidence Anchors

- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `crates/taru-core/src/media/item.rs`
- `crates/taru-core/src/media/metadata.rs`
- `crates/taru-core/src/media/profile.rs`
- `crates/taru-metadata/src/merge.rs`
- `crates/taru-metadata/src/confirmation.rs`
- `crates/taru-nfo/src/import.rs`

## Fresh Evidence

2026-05-18, MMP-010:

- Workstream opened from ARF-002 / ARF-040.
- Current duplicated merge anchors identified in `taru-metadata` and `taru-nfo`.
- First executable task set to characterization before policy movement.

Fresh verification is required before marking any implementation task or lane
complete.

