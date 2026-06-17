# API Scale And Cache Contracts - Current State

## What Is Already Shipped

- Public Client library browse/search routes already exist and are access-aware.
- Public JSON browse/search list routes already use `Cache-Control: no-store`,
  including `items`, `search`, `libraries`, `libraries/{library_id}/sources`,
  `libraries/{library_id}/items`, `people`, `people/{person_id}/items`,
  `tags`, `tags/{tag_id}/items`, `genres`, and `genres/{genre_id}/items`.
- Selected artwork image responses already have a private validator contract with `ETag` and `304 Not Modified`.
- Search projection and deterministic in-memory scoring already exist.
- Current public paging is still offset-based through `PageInfo { limit, offset, returned }`.
- Response-budget guidance for public list routes is documented at the HTTP API boundary.

## What Is Still Missing

- There is no cursor or snapshot paging contract for large-library browse/search.
- Browse/search cache validators are not yet a first-class contract.
- Query-shape tests still need stronger coverage for cursor-style pagination, cross-principal combinations, and future validator behavior.

## Recommended First Slice

Start with repository and server contract hardening, not a frontend change.

- Prove stable ordering and page-hole behavior for the browse/search projections.
- Keep the current dynamic JSON `no-store` baseline explicit and tested.
- Add response-budget and query-shape regression coverage.
- Defer any public cursor-shape change until tests prove the offset contract cannot stay stable enough.

## Seams To Keep Thin

- `crates/nako-server/src/http/catalog.rs`
- `crates/nako-server/src/http/library.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-search/src/lib.rs`

The leverage is in repository/query semantics and route-contract tests, not in
growing the HTTP shell.
