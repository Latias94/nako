# Public library browse scale contract

## Goal

Strengthen the backend contract for large-library browsing and search so the
current frontend sorting/filtering/search UI has stable server behavior under
self-hosted operator conditions.

## Plan Anchor

This is the next code-bearing slice from
`docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`,
unit U2.

## Requirements

* Public library browse/search routes must keep access filtering in the app
  service, not in HTTP handlers.
* Large-library browse/search responses must keep bounded pagination and stable
  ordering semantics.
* Existing browse facets and sort keys must continue to round-trip through the
  backend contract without exposing internal storage details.
* Cache validator behavior must remain redaction-safe and consistent for
  browse/search responses.
* No new frontend runtime behavior is required in this slice.
* No schema migration or unrelated public API expansion is allowed unless a
  concrete contract hole is discovered during implementation.

## Acceptance Criteria

* [ ] Browse/search contract gaps for large libraries are identified and fixed
      in backend code or documented as deferred with evidence.
* [ ] Public routes and app services continue to enforce access filtering on
      the server side.
* [ ] Pagination, sort, and facet behavior are covered by focused tests for the
      changed contract.
* [ ] Any cache validator or response metadata changes remain redaction-safe.
* [ ] No unrelated auth, playback, addon, or storage mutation behavior changes.
* [ ] Focused `nako-server` / `nako-api` verification passes for the touched
      layers.

## Definition of Done

* Browse/search contract is either tightened or the remaining gap is clearly
  documented.
* Trellis context validates.
* `git diff --check` passes.
* Focused tests pass.
* Commit message is conventional.

## Out of Scope

* Playback reason contract.
* Addon lifecycle.
* VFS repair mutation policy.
* Remote relay or central account work.
* Frontend UI changes.

## Technical Notes

Primary files to inspect:

* `crates/nako-server/src/app/library.rs`
* `crates/nako-server/src/app/catalog.rs`
* `crates/nako-server/src/http/library.rs`
* `crates/nako-server/src/http/catalog.rs`
* `crates/nako-server/src/http/query.rs`
* `crates/nako-server/src/http/tests/library.rs`
* `crates/nako-server/src/http/tests/catalog.rs`
* `crates/nako-api/src/public_client.rs`

