# Provider Review Global Queue Search - Design

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`admin-candidate-review-list-navigation` made durable Metadata Candidate
Reviews discoverable from a Media Item context. Operators still need a global
queue/filter surface when they are triaging provider review work across a
library and do not already know the item ID.

The next correct slice is a read-only Admin queue before batch governance or
related hierarchy application. Queue/search changes query shape and operator
workflow; it should not be hidden inside the closed item-scoped navigation lane.

## Relevant Authority

- ADRs:
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/admin-candidate-review-list-navigation/CLOSEOUT.md`
  - `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md`
  - `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`

## Problem

The existing Admin API can load a Candidate Review by `review_id` and can list
reviews for one `item_id`. That is enough for item detail navigation, but not
for operational review queues such as "all pending Bangumi reviews updated
recently" or "accepted reviews waiting for explicit application". Without a
global read path, operators must discover item IDs elsewhere and cannot triage
review work at library scale.

## Target State

When this lane closes:

1. Admin API exposes a paginated global Metadata Candidate Review queue route.
2. Queue filters include status and provider/source. Broader search across
   review ID, item ID, source key, or root summaries remains a follow-on until
   a redaction-safe projection/index owns that scope.
3. Queue entries remain summaries for triage/navigation and link to the
   existing detail/apply route for full evidence.
4. Web Admin can browse the global queue and route into the existing
   Candidate Review detail/apply page.
5. Batch governance, bulk apply, related hierarchy application, and Public
   Client API remain follow-ons.

## In Scope

- Core repository query contract for global Candidate Review queue reads.
- SQLite/PostgreSQL query adapters and indexes if needed for stable ordering.
- Admin DTOs and route inventory for the global queue.
- Server HTTP read path using the repository query.
- Generated Admin TypeScript contract sync.
- Web Admin queue/filter route-state and data-source mapping.
- Redaction, pagination, filter, no-write, and browser smoke tests.

## Out Of Scope

- Public Client API.
- Status mutation, accept/reject, apply, or batch governance.
- Related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- Raw provider payload, token, header, proxy URL, local path, image URL, source
  fingerprint, or raw idempotency-key exposure.
- Provider endpoint depth changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Durable Candidate Review records already contain enough summary data for queue rows. | High | `admin-candidate-review-list-navigation` list DTOs and tests. | Add a queue summary projection before Web work. |
| Existing storage only supports item-scoped listing today. | High | `MetadataCandidateReviewRepository` only has `list_metadata_candidate_reviews_for_item`. | `PRGQ-020` must add a repository query contract before Web work. |
| Global queue ordering should be `updated_at_ms DESC, id ASC` for deterministic pagination. | High | Existing item-scoped list uses the same order. | Add tests for stable ordering and cross-backend parity. |
| Queue/search can stay Admin-only until operator semantics settle. | High | Prior lanes kept Candidate Review governance out of Public Client API. | Do not add protocol DTOs or public SDK surface in this lane. |

## Architecture Direction

### Global Queue Before Bulk Mutation

The queue is discovery and triage. It should expose enough summary facts to
find work, then route operators into the existing detail/apply page. Bulk
accept/apply has different idempotency, stale guard, and partial-failure
semantics and stays out of this lane.

### Repository Query Contract, Not HTTP-Local Filtering

Filtering and pagination belong behind the repository contract so SQLite and
PostgreSQL can prove the same ordering, limits, and redaction behavior. HTTP
must not load broad rows and filter them in memory.

### Search Policy Is Explicit

Free-text search must be limited to redaction-safe fields. If root summary
search needs a durable projection or index, split that projection before
claiming queue search complete. Do not search raw provider JSON.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | COVERED | `CONTEXT.md` | Uses Media Item, Metadata Candidate Review, Provider Subject, Provider Mapping, Admin API, and Public Client API terms. |
| Metadata ADRs | COVERED | ADR 0007, ADR 0018, ADR 0021 | Keeps provider evidence separate from Canonical Metadata and prevents raw provider/secret leakage. |
| Prior navigation closeout | COVERED | `docs/workstreams/admin-candidate-review-list-navigation/CLOSEOUT.md` | Confirms item-scoped discovery is done and global queue is the residual gap. |
| Repository seam | COVERED | `crates/nako-core/src/repository/metadata.rs` | Confirms only item-scoped listing exists today. |
| Database adapters | COVERED | `crates/nako-db/src/sqlite/metadata_candidate_review.rs`; `crates/nako-db/src/postgres/metadata_candidate_review.rs` | New queue must prove SQLite/PostgreSQL query parity. |
| Architecture lane map | COVERED | `docs/architecture/LANES.md`; `docs/architecture/LIBRARY_PIPELINE.md` | Routes this as the active library-metadata-control-plane follow-on. |

## Closeout Condition

This lane can close when:

- global queue target state is implemented;
- Admin API/Web gates pass with fresh evidence;
- docs reflect shipped behavior;
- and batch governance, related hierarchy application, and provider endpoint
  depth remain split or explicitly deferred.

Closeout status: met at `PRGQ-040`.
