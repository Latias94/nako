# NSPA-060 Journal — Export Write Failure Slice

Date: 2026-05-21
Owner: codex
Status: PARTIAL

## Scope

Added the next partial-failure gate for accepted NFO export apply: storage write
failure before sidecar mutation.

## Behavior Proven

- A failing storage backend can pass preview/acceptance reads but reject `.nfo`
  writes during apply.
- Export apply records `FailedBeforeMutation` instead of `Committed`.
- No sidecar file is created after the injected write failure.
- Outcome diagnostics keep `writes_library`, `storage_mutation`, and
  `metadata_mutation` false and include safe counts only.
- Operator-facing outcome JSON does not include raw OS paths or raw XML.

## Validation

```powershell
cargo nextest run -p taru-server nfo_sidecar_apply_export_write_failure_records_failed_before_mutation --no-fail-fast
cargo fmt --all -- --check
cargo nextest run -p taru-server nfo_sidecar_apply --no-fail-fast
cargo nextest run -p taru-server nfo --no-fail-fast
```

All commands passed.

## Remaining NSPA-060 Work

NSPA-060 is still not complete. Remaining gates should inject failures across:

- backup restore or rollback paths;
- metadata commit failures;
- retention diagnostic failures.

Each remaining gate must prove one of `FailedBeforeMutation`,
`RollbackComplete`, or `RepairPending`, with redacted diagnostics.
