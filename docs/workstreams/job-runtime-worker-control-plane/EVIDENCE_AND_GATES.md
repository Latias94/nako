# Job Runtime Worker Control Plane Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Required Gates

Initial design/inventory gate:

```powershell
Get-Content docs\workstreams\job-runtime-worker-control-plane\WORKSTREAM.json | ConvertFrom-Json
git diff --check
```

Expected implementation gates once code changes begin:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_runtime_worker --no-fail-fast
cargo nextest run -p taru-db job_runtime_worker --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

## Redaction Inventory

Before any Admin response or diagnostics closeout, run:

```powershell
rg -n "input_json|summary_json|error|source_uri|storage_uri|managed-artwork://|cache_uri|local_path|artifact_root|payload_json|provenance_json|token|secret|cancel|requeue|lease|worker" crates/taru-api crates/taru-server docs/api docs/workstreams/job-runtime-worker-control-plane
```

Expected result:

- raw job inputs/errors are internal only;
- Admin DTOs use presence flags or safe summaries;
- tests may contain forbidden terms only as redaction fixtures/assertions;
- cancellation claims appear only where the worker can observe cancellation.

## Evidence Log

| Date | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-19 | `Get-Content ... WORKSTREAM.json \| ConvertFrom-Json` | Pass | Workstream JSON parses and reports `active` / `JRWCP-010`. |
| 2026-05-19 | `git diff --check` | Pass | Opening-doc diff is whitespace-clean; Git reports only line-ending normalization warnings. |

## Review Checklist

- Worker lifecycle is registered with runtime supervision.
- Job-kind execution remains typed.
- Claim/lease/recovery semantics are transactional.
- Resource class limits are explicit.
- Admin controls are redacted.
- Manual Admin `process-next` remains compatible or is deliberately retired
  with tests and docs.
