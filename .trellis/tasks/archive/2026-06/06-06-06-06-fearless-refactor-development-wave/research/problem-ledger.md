# Fearless Refactor And Development Problem Ledger

## Scoring

- Impact: user/operator value or correctness risk.
- Architecture risk: chance the issue causes future feature conflict or hidden
  technical debt.
- Readiness: enough evidence exists to implement safely now.
- Conflict: likelihood of colliding with shared Admin/API/DB/runtime surfaces.

## Active Queue

| Rank | Finding | Impact | Architecture Risk | Readiness | Conflict | Proposed Task |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Source fingerprint hash has internal enqueue/execution but no Admin manual trigger. | High | Medium | High | Admin API | `admin-source-fingerprint-hash-trigger-first-slice` |
| 2 | Source Duplicate Relationship pair idempotency differs between SQLite and PostgreSQL baseline. | High | High | Medium | DB schema/repository | `source-duplicate-relationship-idempotent-pair-upsert` |
| 3 | Persisted source hash evidence has no read-only duplicate reconciliation plan. | Medium | High | Medium | Admin/API/source identity | `source-duplicate-reconciliation-plan-first-slice` |
| 4 | Source hash retry/requeue is available only as lower-level durable job primitives, not as a source-hash-safe Admin command. | Medium | Medium | Medium | Admin jobs | `source-hash-retry-requeue-admin-command` |
| 5 | Storage/source identity runtime parity still has SQLite-first risk. | Medium | Medium | Medium | DB/test infra | `storage-source-identity-postgres-runtime-harness` |

## Deferred

| Finding | Reason Deferred |
| --- | --- |
| Broad disk-scan job executor registry | Only two disk-scan job variants currently share the shape; abstraction would be premature. |
| Automatic duplicate reconciliation apply | Needs pair idempotency, read-only plan, redaction, operator visibility, and undo semantics first. |
| Cross-library duplicate relationships | Library Access and playback visibility implications are not specified yet. |
| Admin Web route helper cleanup | Lower value than backend source hash and source identity contract work. |

## New Findings During Wave

Add new findings here before opening new child tasks.
