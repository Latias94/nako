# API scale and cache contracts

## Goal

Make public list/read APIs safe for large self-hosted libraries and multiple
principals by tightening browse/search ordering, pagination, and cache
contracts without adding frontend behavior.

## Problem

The current Public Client browse/search surface is usable, but large libraries
still depend on offset paging, ad hoc budget assumptions, and conservative
cache defaults. That is enough to ship a product, but not enough to guarantee
stable behavior under multi-user, large-library, self-hosted pressure.

The remaining work is not a rewrite. It is a contract hardening pass that keeps
ordering stable, keeps access filtering on the server side, and makes the cache
story explicit enough that future validator work does not cross Library Access
boundaries by accident.

## Requirements

- Keep server-side access filtering in the app/repository path.
- Keep browse/search/list responses bounded and deterministic.
- Preserve existing sort, facet, and watch-state parsing.
- Keep dynamic JSON list routes conservative until validator design is
  explicit.
- Add query-shape regression tests for page holes, sort ties, and repeated
  filters.
- Avoid frontend runtime changes in this slice.
- Avoid broad public API shape changes unless the tests prove the current
  contract cannot stay stable.

## Acceptance Criteria

- Browse/search/list ordering is proven stable for the supported sort paths.
- Hidden or inaccessible hits do not shift visible page boundaries.
- Public JSON browse/search list routes remain explicitly `no-store`.
- Query-shape tests cover combined sort/filter/search/page cases.
- Response-budget guidance is documented for the touched list surfaces.
- No unrelated auth, playback, addon, or storage mutation behavior changes.
- Focused `nako-db`, `nako-server`, and contract checks pass for the touched
  layers.

## Definition Of Done

- The current browse/search contract is either tightened or the remaining gap
  is explicitly deferred with evidence.
- Trellis context validates.
- `git diff --check` passes.
- Focused tests pass.
- The commit message is conventional.

## Scope Boundaries

### In Scope

- Large-library browse/search contract hardening.
- Repository query-shape coverage for filtered pagination and stable ordering.
- Public Client JSON response-budget and cache-policy reinforcement.
- Route and app-service tests that prove the current contract stays bounded.

### Deferred For Later

- A new cursor-based public paging shape, unless the current offset contract
  fails the regression tests.
- Conditional validators for dynamic browse/search responses.
- Any UI changes needed to consume a future cursor token.

### Outside This Product's Identity

- Playback reason contract.
- Addon lifecycle.
- VFS repair mutation policy.
- Remote relay or central account work.
- Frontend UI changes.

## Implementation Units

### IU1. Repository Query-Shape Hardening

- Prove stable ordering, page-hole behavior, and access filtering in the
  repository-backed browse/search paths.
- Keep sort and facet parsing out of SQL and keep the query contract deterministic.
- Add contract coverage for hidden-hit holes, duplicate-source deduplication,
  and combined filter/sort/page cases.

Files to inspect:

- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-db/src/sqlite/`
- `crates/nako-db/src/postgres/`
- `crates/nako-core/src/repository/`
- `crates/nako-server/src/app/catalog.rs`
- `crates/nako-server/src/app/library.rs`

Test scenarios:

- A hidden or inaccessible hit does not consume the first visible page.
- Stable tie-breaks do not change visible ordering across adapters.
- Access filtering happens before pagination for the affected browse/search
  projections.

### IU2. HTTP Response And Budget Contract

- Keep dynamic JSON browse/search list routes explicitly `no-store`.
- Add route tests that prove the list surfaces remain bounded and do not rely
  on total-count assumptions.
- Keep HTTP handlers thin and leave query parsing plus access decisions in the
  app/repository seam.

Files to inspect:

- `crates/nako-server/src/http/catalog.rs`
- `crates/nako-server/src/http/library.rs`
- `crates/nako-server/src/http/query.rs`
- `crates/nako-server/src/http/tests/catalog.rs`
- `crates/nako-server/src/http/tests/library.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `docs/api/HTTP_API.md`

Test scenarios:

- Covered browse/search list routes continue to emit `Cache-Control: no-store`.
- Combined sort/filter/search/page requests return bounded pages with stable
  ordering.
- Access rejection still happens before any response metadata can leak host
  details.

### IU3. Docs And Follow-On Decision

- Record the remaining cursor/validator follow-on decision in the architecture
  docs after the first hardening slice lands.
- Keep the plan aligned with the existing roadmap and workstream index.
- Update the task context with the concrete evidence that made the slice
  necessary.

Files to inspect:

- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
- `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`

## Technical Notes

- `PageInfo` is still offset-based today; treat any cursor addition as a
  deliberate public contract change.
- The selected-artwork validator contract is already separate and should stay
  that way.
- `nako-search` remains a pure scorer; do not add transport concerns there.
- `nako-server` should stay thin at the HTTP boundary and push real leverage
  into repository/query semantics.
