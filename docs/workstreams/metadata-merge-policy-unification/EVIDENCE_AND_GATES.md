# Metadata Merge Policy Unification Evidence And Gates

Status: Completed
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
- `crates/taru-core/src/media/merge.rs`
- `crates/taru-core/src/media/metadata.rs`
- `crates/taru-core/src/media/profile.rs`
- `crates/taru-metadata/src/confirmation.rs`
- `crates/taru-nfo/src/import.rs`

## Fresh Evidence

2026-05-18, MMP-010:

- Workstream opened from ARF-002 / ARF-040.
- Current duplicated merge anchors identified in `taru-metadata` and `taru-nfo`.
- First executable task set to characterization before policy movement.

2026-05-18, MMP-020:

- Added `hierarchy_confirmation_allows_source_authority_to_refresh_own_locked_fields`
  to characterize provider-source locks during hierarchy confirmation.
- Added `nfo_service_allows_nfo_authority_to_refresh_nfo_locked_fields` to
  characterize NFO-source locks during NFO Import.
- `cargo nextest run -p taru-metadata hierarchy_confirmation_allows_source_authority_to_refresh_own_locked_fields --no-fail-fast`
  passed: 1 passed, 27 skipped.
- `cargo nextest run -p taru-nfo nfo_service_allows_nfo_authority_to_refresh_nfo_locked_fields --no-fail-fast`
  passed: 1 passed, 19 skipped.
- `cargo nextest run -p taru-metadata merge --no-fail-fast` passed: 2 passed,
  26 skipped.
- `cargo nextest run -p taru-nfo nfo_service --no-fail-fast` passed: 16
  passed, 4 skipped.
- `cargo nextest run -p taru-metadata full_refresh_replaces_unlocked_existing_values --no-fail-fast`
  passed: 1 passed, 27 skipped.
- Review result: no blocking findings for MMP-020. Residual risk for MMP-030:
  provider refresh currently respects all locks through `MetadataMergePolicy`,
  while hierarchy confirmation and NFO import use source-aware lock filtering.
  The shared boundary must preserve that distinction or deliberately change it
  with tests.

2026-05-18, MMP-030:

- Shared `MetadataMergePolicy`, `MetadataMergeMode`, `MetadataLockScope`, and
  `populated_metadata_fields` now live in `crates/taru-core/src/media/merge.rs`.
- `taru-metadata` re-exports `MetadataMergePolicy` for compatibility with
  existing internal callers.
- Provider refresh still uses all-lock protection through
  `MetadataMergePolicy::from_locks_and_mode`.
- Hierarchy confirmation now uses
  `MetadataMergePolicy::for_source_refresh_mode`, preserving source-aware lock
  behavior without a private lock-filter helper.
- NFO import now uses `MetadataMergePolicy::for_nfo_import` and
  `populated_metadata_fields`, removing its private Canonical Metadata merge
  loop and populated-field enumeration.
- `cargo check -p taru-core --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p taru-metadata merge --no-fail-fast` passed: 2 passed,
  26 skipped.
- `cargo nextest run -p taru-metadata full_refresh_replaces_unlocked_existing_values --no-fail-fast`
  passed: 1 passed, 27 skipped.
- `cargo nextest run -p taru-metadata hierarchy_confirmation_allows_source_authority_to_refresh_own_locked_fields --no-fail-fast`
  passed: 1 passed, 27 skipped.
- `cargo nextest run -p taru-nfo nfo_service --no-fail-fast` passed: 16
  passed, 4 skipped.

2026-05-18, MMP-040:

- README and DESIGN now describe the shipped `taru-core` merge-policy boundary.
- TODO, HANDOFF, and WORKSTREAM state now point to MMP-050 closeout.
- Evidence anchors now use `crates/taru-core/src/media/merge.rs` instead of the
  removed `crates/taru-metadata/src/merge.rs`.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

2026-05-18, MMP-050 closeout:

- `cargo check -p taru-core --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.
- `cargo nextest run -p taru-metadata --no-fail-fast` passed: 28 passed, 0
  skipped.
- `cargo nextest run -p taru-nfo --no-fail-fast` passed: 20 passed, 0 skipped.
- Review result: no blocking workstream or code quality findings. Remaining
  non-blocking follow-ons are provider priority configuration and user/admin
  merge diagnostics.

2026-05-18, resume closeout verification:

- Corrected `WORKSTREAM.json` continue policy to a single `default_action`.
- Updated workstream status headers to `Completed` across README, DESIGN,
  MILESTONES, TODO, EVIDENCE_AND_GATES, and HANDOFF.
- `Get-Content -Raw docs/workstreams/metadata-merge-policy-unification/WORKSTREAM.json | ConvertFrom-Json`
  passed.
- `cargo check -p taru-core --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Windows line-ending warnings only.
- `cargo nextest run -p taru-metadata --no-fail-fast` passed: 28 passed, 0
  skipped.
- `cargo nextest run -p taru-nfo --no-fail-fast` passed: 20 passed, 0 skipped.

Fresh verification is required before marking any implementation task or lane
complete.
