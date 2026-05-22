# Phase 22.0: Ingestion Failure Diagnostics

## Summary

M22 makes scan and probe failures durable and queryable. Ingestion no longer
only reports transient per-job failure strings. Scan directory/object failures
and probe source failures are recorded in SQLite, can be retried by rerunning
scan/probe, and can be ignored when the operator decides a failure is expected.

## Failure Model

`nako-core::ingestion` defines:

- `IngestionFailurePhase`: `scan` or `probe`;
- `IngestionFailureClass`: storage, probe, database, invalid input,
  unsupported, or unknown;
- `IngestionFailureStatus`: open, resolved, or ignored;
- `IngestionFailureRecord` and `NewIngestionFailure`.

SQLite persists these records in `ingestion_failures`. Records are keyed by
`library_id + phase + target_uri`, so repeated failures update the same row and
increment `attempts`. A successful scan/probe of the same target resolves the
open failure.

## Pipeline Behavior

Scan behavior:

- unreadable scan entries are recorded as scan failures;
- readable sibling directories/files continue to be indexed;
- partial scans do not tombstone missing sources because the scan is not a
  complete source-of-truth pass;
- scan summaries include `failed_entries`.

Probe behavior:

- per-source open/stage/ffprobe/persist failures are recorded as probe
  failures;
- successful probes and skipped existing probes resolve previous probe
  failures for the same source locator;
- probe summaries retain per-source failure details and `failed_sources`.

## Diagnostics Surface

HTTP:

- `GET /libraries/{library_id}/ingestion/failures` lists open failures by
  default.
- Query parameters support `phase`, `status`, `limit`, and `offset`.
- `POST /libraries/{library_id}/ingestion/failures` accepts
  `IgnoreIngestionFailureRequest` and marks a target failure ignored.

CLI:

- `nako-server ingestion-failures --library-id <id>` lists open failures.
- `--phase scan|probe`, `--status open|resolved|ignored`, and `--all` narrow or
  broaden the query.

## Validation

Close-out validation:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```
