# NSPA-060 Journal — Import Metadata Commit Failure Slice

Date: 2026-05-21
Owner: codex
Status: PARTIAL

## Scope

Added the next partial-failure gate for accepted NFO import apply: metadata
commit failure before canonical mutation.

## Behavior Proven

- Import apply can inject a metadata commit failure after accepted preview
  revalidation and before `nako-nfo` commits canonical metadata.
- The apply record becomes `FailedBeforeMutation`, not `Committed`.
- Canonical metadata remains unchanged.
- Field locks remain unchanged.
- The sidecar file is not rewritten by import-only apply.
- Outcome diagnostics keep `writes_library`, `storage_mutation`, and
  `metadata_mutation` false and do not include raw OS paths or raw XML.

## Validation

```powershell
cargo nextest run -p nako-server nfo_sidecar_apply_import_metadata_commit_failure_records_failed_before_mutation --no-fail-fast
cargo fmt --all -- --check
cargo nextest run -p nako-server nfo_sidecar_apply --no-fail-fast
cargo nextest run -p nako-server nfo --no-fail-fast
```

All commands passed.

## Remaining NSPA-060 Work

NSPA-060 is still not complete. Remaining gates should inject failures across:

- backup restore or rollback paths;
- retention diagnostic failures.

Each remaining gate must prove one of `FailedBeforeMutation`,
`RollbackComplete`, or `RepairPending`, with redacted diagnostics.
