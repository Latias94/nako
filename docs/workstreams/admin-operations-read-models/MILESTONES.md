# Admin Operations Read Models Milestones

Status: Completed
Last updated: 2026-05-18

## M57 — Event Outbox List/Filter

Exit criteria:

- `GET /admin/v1/events` exists.
- It supports kind, status, library_id, source_id, limit, and offset filters.
- It returns admin-owned redacted DTOs.
- It does not expose event `payload_json`, `idempotency_key`, raw
  `last_error`, secret values, or local paths.
- Public OpenAPI/SDK and `nako-client-protocol` remain unchanged.

Primary gates:

- `cargo check -p nako-db --tests`
- `cargo nextest run -p nako-db outbox --no-fail-fast`
- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`

## M58 — Storage Staging/Cache Diagnostics

Exit criteria:

- `GET /admin/v1/storage/staging` exists.
- It supports purpose, state, limit, and offset filters.
- It returns staging/cache summaries and redacted staging rows.
- It does not expose `local_path`, full `source_uri`, staging roots, local
  filesystem paths, or validation error text.
- It records VFS cache listing/failure list as a follow-on if not implemented
  by this slice.

Primary gates:

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`

## M59 — Sanitized Server Config Diagnostics

Exit criteria:

- `GET /admin/v1/system/config` exists.
- It exposes config capabilities, counts, safe numeric settings, provider
  enablement, and secret-reference names.
- It does not expose database URL, local roots, FFmpeg paths, staging root,
  WebDAV base URL, WebDAV username, metadata proxy value, literal header
  values, resolved secrets, or tokens.
- Public OpenAPI/SDK and `nako-client-protocol` remain unchanged.

Primary gates:

- `cargo check -p nako-api --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`

## Closeout

Exit criteria:

- `docs/GOALS.md`, `docs/api/HTTP_API.md`, and admin-web-console notes reflect
  the shipped routes.
- Workstream TODO and evidence are updated.
- `cargo fmt --all -- --check`, focused test gates, public leakage checks,
  `git diff --check`, and `git diff --name-only -- crates/nako-client-protocol`
  pass.
