# Public JSON Browse Cache Policy

## Problem

Public Client JSON browse and search list routes are authenticated, access
filtered, and sometimes user-state dependent. Frontend and SDK consumers combine
pagination, filters, search text, sort keys, Library Access, and watch-state
facts on these routes. Without an explicit cache policy, browsers or proxies can
guess cache behavior from generic HTTP defaults instead of Nako's access model.

Selected artwork byte routes already have a private ETag and `304 Not Modified`
contract. Playback byte routes already use conservative `no-store`. This task
defines the first conservative JSON-list cache slice for dynamic Public Client
browse/search responses.

## Requirements

- The covered Public JSON list responses that depend on Library Access, current
  principal, user playback state, search index freshness, or mutable catalog
  membership must use `Cache-Control: no-store` until a dedicated validator
  design exists.
- The slice must cover the media browse/search routes currently used by Admin
  Web media surfaces:
  - `GET /items`
  - `GET /search`
  - `GET /libraries`
  - `GET /libraries/{library_id}/sources`
  - `GET /libraries/{library_id}/items`
- Keep selected artwork image routes on their existing
  `Cache-Control: private, max-age=86400` plus ETag/304 contract.
- Do not add `ETag`, `Last-Modified`, `304`, total-count, DTO, OpenAPI, SDK, or
  database behavior in this slice.
- Do not touch unrelated dirty Admin API files:
  `crates/nako-api/src/admin/incident_bundle.rs` and
  `crates/nako-api/src/admin/managed_artwork.rs`.

## Acceptance Criteria

- Focused HTTP route tests prove the affected Public Client JSON list responses
  include `Cache-Control: no-store`.
- Existing selected artwork cache behavior remains covered by its existing
  tests and is not changed.
- Trellis task validation passes.
- Rust formatting and focused `nako-server` gates pass.
- The final commit stages only files related to this task.

## Scope Boundaries

In scope:

- `crates/nako-server` HTTP response assembly for the listed Public Client JSON
  browse/search list routes.
- Focused `crates/nako-server/src/http/tests/catalog.rs` coverage.
- Task evidence.

Out of scope:

- Public image route cache helpers and ETag matching.
- Playback byte or HLS cache policy.
- Admin API list cache policy.
- Public Client generated SDK/OpenAPI changes.
- Repository query behavior or migrations.
- Search ranking, sort, filter, or pagination semantics.

## Implementation Evidence

- Added a shared `no_store_json` HTTP helper in `crates/nako-server/src/http.rs`
  for JSON DTO responses that must set `Cache-Control: no-store`.
- Applied the helper to:
  - `GET /items`
  - `GET /search`
  - `GET /libraries`
  - `GET /libraries/{library_id}/sources`
  - `GET /libraries/{library_id}/items`
- Added focused route coverage in
  `crates/nako-server/src/http/tests/catalog.rs`.
- Captured the reusable contract in
  `.trellis/spec/nako-server/backend/http-api-patterns.md`.

## Verification Plan

- `cargo nextest run -p nako-server public_json_browse_routes_use_no_store_cache_policy --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo fmt --all -- --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-public-json-browse-cache-policy`
- `git diff --check`
