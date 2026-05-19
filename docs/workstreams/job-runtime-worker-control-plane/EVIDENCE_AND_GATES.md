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
cargo nextest run -p taru-db managed_artwork_ingest --no-fail-fast
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
| 2026-05-19 | `JRWCP-010` inventory | Pass | `DESIGN.md` now lists existing job execution surfaces, runtime primitives, and the minimal Managed Artwork worker contract. |
| 2026-05-19 | `cargo nextest run -p taru-server managed_artwork_ingest_worker --no-fail-fast` | Pass | Initial worker success tracer passed before the test was renamed to the workstream filter prefix. |
| 2026-05-19 | `cargo nextest run -p taru-server managed_artwork_ingest_worker --no-fail-fast` plus config tests | Pass | Worker success path and artwork config defaults/round-trip passed; command also proved the `RuntimeSupervisor` visibility warning was removed. |
| 2026-05-19 | `cargo nextest run -p taru-server job_runtime_worker --no-fail-fast` | Pass | Formal `JRWCP-020` server gate passed after renaming the worker test to the workstream prefix. |
| 2026-05-19 | `cargo nextest run -p taru-server config_round_trips_from_toml config_uses_default_runtime_settings --no-fail-fast` | Pass | Artwork worker config parses from TOML and remains disabled by default. |
| 2026-05-19 | `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests` | Pass | Cross-crate test builds for the touched core/db/api/server boundary. |
| 2026-05-19 | `cargo fmt --all -- --check` | Pass | Workspace formatting is clean after `cargo fmt --all`. |
| 2026-05-19 | `cargo nextest run -p taru-db job_runtime_worker --no-fail-fast` | No tests | No taru-db tests match `job_runtime_worker`; gate was corrected to the Managed Artwork ingest state-machine filter. |
| 2026-05-19 | `cargo nextest run -p taru-db managed_artwork_ingest --no-fail-fast` | Pass | Existing DB state-machine gate proves failed Managed Artwork ingest requeue/claim semantics remain intact. |
| 2026-05-19 | `git diff --check` | Pass | Diff is whitespace-clean; Git reports only line-ending normalization warnings. |
| 2026-05-19 | Redaction inventory command | Reviewed | Output is large; new worker hits are config/docs/test assertions. Raw locator/storage terms remain internal code, docs, or redaction fixtures. |

## Review Checklist

- Worker lifecycle is registered with runtime supervision.
- Job-kind execution remains typed.
- Claim/lease/recovery semantics are transactional.
- Resource class limits are explicit.
- Admin controls are redacted.
- Manual Admin `process-next` remains compatible or is deliberately retired
  with tests and docs.
