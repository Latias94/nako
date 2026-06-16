# Public Browse Combined Filters Contract

## Problem

Admin Web and future Public Client surfaces already let users combine media item
search text, kind facets, sort keys, sort order, watch-state filters, and
pagination. The backend has most of this shape, but the contract needs explicit
evidence that frontend-style query serialization reaches the Public Client
routes safely and that repository-backed filtering remains access-safe before
pagination.

This task covers the first U2 slice from
`docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`.

## Requirements

- Audit the current frontend, generated SDK, HTTP query parser, app-service, and
  repository paths for media browse and search query composition.
- Strengthen the smallest missing contract for combined browse/search filters,
  sorting, watch-state, and pagination.
- Keep Library Access enforcement before response projection and pagination.
- Keep public route responses bounded and redaction-safe.
- Do not touch unrelated dirty Admin API files:
  `crates/nako-api/src/admin/incident_bundle.rs` and
  `crates/nako-api/src/admin/managed_artwork.rs`.
- Do not change DTOs, generated SDKs, OpenAPI, or database adapters unless the
  audit finds a real backend contract gap.

## Acceptance Criteria

- The task records which frontend/SDK query shapes are supported by the backend.
- At least one focused backend test covers any missing combined-query shape
  found during audit, or the task records why existing coverage is sufficient.
- Focused `nako-server` and/or `nako-db` gates pass for the touched behavior.
- Trellis task validation passes.
- The final commit stages only files related to this task.

## Implementation Evidence

- Admin Web media browse uses `facet`, `sort`, `order`, `watch_state`, `limit`,
  and `offset` for library item pages, and uses `q`, `facet`, `limit`, and
  `offset` for search pages.
- The generated TypeScript Public Client SDK accepts `facet` as either a string
  or string array and renders arrays as comma-separated query values.
- The generated Kotlin Public Client SDK renders library item facets with the
  same CSV helper used by other capability-style query lists.
- Server query parsing already accepts repeated and comma-separated facets for
  `/search`, and accepts repeated and comma-separated `kind:*` facets for
  `/libraries/{library_id}/items`.
- Repository contracts already cover library-item sort keys, watch-state
  filters, facet semantics, null ordering, deduplication, and pagination after
  filtering.
- This task added route-level evidence for the remaining library item browse
  shape: CSV facet values combined with `watch_state`, `sort`, `order`, and
  bounded pagination.

## Scope Boundaries

In scope:

- Public Client `/libraries/{library_id}/items` query parsing and route tests.
- Public Client `/search` query parsing and route tests when needed.
- Repository contract tests only if adapter behavior changes or an uncovered DB
  semantic gap is found.
- Task evidence and implementation notes.

Out of scope:

- Admin Web UI redesign or frontend interaction changes.
- New filter dimensions beyond the existing public query vocabulary.
- Total-count semantics.
- Cache validators unless this slice changes cache behavior.
- Broad search ranking, metadata matching, or Addon search behavior.

## Verification Plan

- `cargo nextest run -p nako-server library_items_route_applies_kind_watch_state_and_last_played_query --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo fmt --all -- --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-public-browse-combined-filters-contract`
- `git diff --check`
