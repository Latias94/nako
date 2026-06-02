# Provider Governance Bulk Review - Design

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`provider-review-global-queue-search` made durable Metadata Candidate Reviews
discoverable across items. Operators can now find review work, but accepted
reviews still apply one at a time. A Jellyfin/Plex-class self-hosted server
needs batch governance for library-scale metadata maintenance without turning
review acceptance into a hidden mutation path.

The lane shipped a read-only batch application plan, bounded backend
confirmation through the existing single-review application authority, and Web
Admin selection/plan/confirm/result governance. Remaining expansion is split
into follow-ons instead of extending this lane.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/provider-review-global-queue-search/CLOSEOUT.md`
- `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md`
- `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md`

## Problem

Nako has safe single-review governance, global review discovery, and explicit
root Provider Mapping application. What is missing is a batch governance
boundary that can scale operator review work while preserving:

- review snapshot freshness checks;
- idempotency-key fingerprinting and replay visibility;
- redaction-safe Admin DTOs;
- root-only Provider Subject / Provider Mapping application;
- separation from related hierarchy application and provider endpoint breadth.

Without a batch plan boundary, batch apply would either duplicate single-review
logic in Admin handlers or hide long-running control-plane behavior inside a
request helper.

## Target State

When this lane closes:

1. Admin API exposes a read-only batch application plan for selected Metadata
   Candidate Reviews.
2. The plan reuses existing single-review application planning semantics and
   summarizes eligible, noop, stale, blocked, conflict, and ineligible rows.
3. Any confirmed batch mutation uses the single-review application service as
   the authority for each row, with bounded selection size, per-review stale
   checks, idempotency-key fingerprints, replay behavior, and redacted partial
   failure results.
4. If batch execution becomes long-running or retryable, it is routed through a
   durable job or runtime-supervisor boundary per ADR 0053 rather than raw
   background spawning.
5. Web Admin can select reviews from the global queue, inspect the batch plan,
   and confirm only the explicit accepted subset.
6. Batch governance remains Admin-only and does not add Public Client API
   surface, related hierarchy application, or provider endpoint breadth.

## In Scope

- Workstream opening and lane routing.
- Read-only Admin API batch plan DTOs/routes for selected review IDs.
- Backend planning that delegates to the existing Candidate Review application
  plan semantics instead of duplicating policy.
- Generated Admin TypeScript contract sync when API shapes change.
- Bounded batch confirmation semantics after the read-only plan is proven.
- Web Admin selection and confirmation workflow after backend semantics are
  stable.
- Redaction, no-write, stale guard, idempotency, pagination/selection bound,
  and partial-failure evidence.

## Out Of Scope

- Public Client API or SDK expansion.
- Related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- Douban TV/episode endpoint support.
- New provider fetch/breadth behavior.
- Reusing Generated Artifact apply outcome tables as the Candidate Review batch
  executor.
- Raw provider payloads, raw cache bodies, image URLs, local paths, proxy URLs,
  headers, tokens, source fingerprints, or raw idempotency keys in Admin rows.
- Unbounded batch size or request-local hidden background work.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing single-review application planning can be reused per selected review. | High | `PGBR-020` reused the existing planner per selected review. | If future row classes expose drift, deepen the single-review planning interface before batch confirmation. |
| A read-only batch plan can ship before batch mutation. | High | `PGBR-020` shipped the plan-first Admin API route. | If product scope changes, record an ADR/product decision before changing mutation order. |
| Initial batch selection can be bounded by review IDs rather than an unbounded queue filter snapshot. | Medium | Current queue rows expose review IDs and route state. | If false, add a filter-snapshot token or durable selection model as a separate task. |
| Batch mutation may stay synchronous only if bounded and fast. | High | `PGBR-030` caps selected reviews at 50 and returns synchronous partial results without background work. | If future UX needs retry/cancel/progress, split durable job execution before expanding the backend route. |

## Architecture Direction

Keep the batch workflow under the `library-metadata-control-plane` lane.

The plan boundary should live above repository storage and below Admin HTTP DTO
translation. It should consume selected review IDs, load durable review
snapshots, and use existing Candidate Review application planning to classify
each row. Admin DTOs should expose only redaction-safe plan facts and page or
selection limits.

Confirmed batch apply must not become a second metadata application executor.
It should call the same single-review application service per row, preserve
per-row stale checks and idempotency, and report partial results explicitly.
If execution needs retries, cancellation, or resource admission, the lane must
route that work through the existing control-plane job/runtime boundary.

## Source Coverage Audit

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| User goal and autonomy | COVERED | Active Codex goal, auto-commit authorization. | Opens and can execute a focused lane. |
| Domain glossary | COVERED | `CONTEXT.md` read. | Uses Candidate Review, Provider Subject, Provider Mapping, Admin API terms. |
| ADR 0007/0018/0021 | COVERED | Read before opening. | Keeps merge policy, provider diagnostics, and Media Item identity boundaries. |
| ADR 0053 | COVERED | Read before opening. | Prevents hidden background batch execution. |
| Prior queue/apply closeouts | COVERED | PRGQ, AWPDG, ARPMA closeouts. | Defines next scope and non-goals. |
| Inventory scripts | MISSING | `scripts/` has no `workstream_inventory.py`, `program_status.py`, or `validate_orchestration_state.py`. | Replaced with read-only JSON status scan before opening. |

## Closeout Condition

This lane can close when:

- batch plan and any accepted batch mutation target state is implemented;
- Admin API/Web gates pass with fresh evidence;
- docs reflect shipped behavior;
- and related hierarchy application, provider endpoint depth, Public Client API,
  durable job expansion, and broader provider governance are split or deferred.
