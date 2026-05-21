# NSPA-060 Journal — Audit Commit Repair-Pending Slice

Date: 2026-05-21
Owner: codex
Status: PARTIAL

## Scope

Implemented the first partial-failure slice for accepted NFO sidecar apply:
final audit commit failure after a successful mutation.

## Behavior Proven

- Export sidecar apply writes the accepted `.nfo`, then an injected final audit
  commit failure records `RepairPending` instead of a false `Committed` state.
- Import sidecar apply mutates canonical metadata and local-authority locks,
  then an injected final audit commit failure records `RepairPending` instead
  of a false `Committed` state.
- Re-applying a `RepairPending` record returns the terminal diagnostic as an
  idempotent replay and does not repeat the mutation.
- Repair-pending outcome JSON records safe booleans such as
  `storage_mutation`, `metadata_mutation`, `repair_required`, and
  `audit_commit_completed`; it does not include raw OS paths or raw XML.

## Validation

```powershell
cargo fmt --all -- --check
cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast
cargo nextest run -p taru-server nfo --no-fail-fast
cargo check -p taru-server
```

All commands passed. `cargo check -p taru-server` still reports existing
dead-code/unused warnings outside this slice.

## Remaining NSPA-060 Work

NSPA-060 is not complete yet. Remaining gates should inject failures across:

- export write before mutation reporting;
- backup restore or rollback paths;
- metadata commit failures;
- retention diagnostic failures.

Each remaining gate must prove one of `FailedBeforeMutation`,
`RollbackComplete`, or `RepairPending`, with redacted diagnostics.
