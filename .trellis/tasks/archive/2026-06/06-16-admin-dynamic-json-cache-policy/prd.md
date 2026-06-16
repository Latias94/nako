# Admin Dynamic JSON Cache Policy

## Goal

Admin API read-model, list, and diagnostic JSON routes describe volatile
operator state: durable jobs, readiness, storage health, playback runtime,
remote access posture, and incident support evidence. Browsers, reverse
proxies, or tunnels should not reuse stale Admin JSON for these dynamic
surfaces unless Nako has an explicit validator design.

This task applies the same conservative cache principle used by Public Client
dynamic JSON browse routes to a focused Admin operator-read-model slice.

## Requirements

- Covered Admin dynamic JSON read responses must include
  `Cache-Control: no-store`.
- The slice must cover these Admin GET routes:
  - `GET /admin/v1/overview`
  - `GET /admin/v1/diagnostics/incident-bundle`
  - `GET /admin/v1/jobs`
  - `GET /admin/v1/storage/backends`
  - `GET /admin/v1/storage/staging`
  - `GET /admin/v1/network/access`
  - `GET /admin/v1/system/config`
  - `GET /admin/v1/access/summary`
  - `GET /admin/v1/playback/runtime`
  - `GET /admin/v1/playback/renderers`
  - `GET /admin/v1/playback/support`
- Reuse the existing shared JSON no-store response helper instead of adding a
  second Admin-only helper.
- Preserve current Admin auth, route status, DTO shape, redaction, pagination,
  and generated contract behavior.
- Keep selected artwork image routes, playback byte/HLS routes, Public Client
  dynamic JSON routes, and mutating Admin commands out of this slice.
- Do not add `ETag`, `Last-Modified`, `304`, OpenAPI/SDK generation,
  total-count behavior, repository changes, migrations, or frontend changes.
- Do not touch unrelated dirty Admin API files:
  `crates/nako-api/src/admin/incident_bundle.rs` and
  `crates/nako-api/src/admin/managed_artwork.rs`.

## Acceptance Criteria

- Focused HTTP route tests prove each covered Admin dynamic JSON route returns
  `Cache-Control: no-store`.
- Existing route status, auth guard, API version header, and response bodies
  remain compatible.
- Trellis task validation passes.
- Rust formatting, focused `nako-server` test gate, and `cargo check -p
  nako-server --tests` pass.
- Final commit stages only files related to this task.

## Technical Approach

- Keep the cache policy at HTTP response assembly time.
- Use `no_store_json` from `crates/nako-server/src/http.rs` for covered Admin
  DTO responses.
- Add a route-level regression test under
  `crates/nako-server/src/http/tests/system.rs` using the existing Axum router
  helpers and a small table of covered routes.
- Document the executable Admin dynamic JSON cache contract in
  `.trellis/spec/nako-server/backend/http-api-patterns.md`.

## Scope Boundaries

In scope:

- `crates/nako-server/src/http/admin.rs` response assembly for the covered GET
  routes.
- `crates/nako-server/src/http/tests/system.rs` route coverage.
- `crates/nako-server` HTTP code-spec update.
- Task evidence.

Out of scope:

- Admin DTO/schema changes in `crates/nako-api`.
- Generated Admin Web contract or frontend changes.
- Admin mutating command cache policy.
- Public Client browse/search cache policy, already covered separately.
- Image/artwork, playback byte, and HLS cache behavior.
- Database, repository, or pagination behavior.

## Implementation Evidence

- Reused `no_store_json` from `crates/nako-server/src/http.rs` for the covered
  Admin dynamic JSON read routes.
- Applied the helper in `crates/nako-server/src/http/admin.rs` for:
  - `GET /admin/v1/overview`
  - `GET /admin/v1/diagnostics/incident-bundle`
  - `GET /admin/v1/jobs`
  - `GET /admin/v1/storage/backends`
  - `GET /admin/v1/storage/staging`
  - `GET /admin/v1/network/access`
  - `GET /admin/v1/system/config`
  - `GET /admin/v1/access/summary`
  - `GET /admin/v1/playback/runtime`
  - `GET /admin/v1/playback/renderers`
  - `GET /admin/v1/playback/support`
- Added focused route coverage in
  `crates/nako-server/src/http/tests/system.rs`.
- Captured the reusable Admin cache contract in
  `.trellis/spec/nako-server/backend/http-api-patterns.md`.

## Verification Plan

- `cargo nextest run -p nako-server admin_dynamic_json_read_routes_use_no_store_cache_policy --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo fmt --all -- --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-admin-dynamic-json-cache-policy`
- `git diff --check`
