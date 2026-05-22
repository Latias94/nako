# Admin Operations Read Models Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

AORM-010 through AORM-060 are complete. Closeout validation passed.

Shipped routes:

- `GET /admin/v1/events`
- `GET /admin/v1/storage/staging`
- `GET /admin/v1/system/config`

## Scope Guardrails

- Keep all new DTOs admin-owned in `nako-api::admin`.
- Do not add routes or DTOs to `nako-client-protocol`.
- Keep public OpenAPI and generated TypeScript SDK free of `/admin/*`.
- Route tests must check redaction, not only status codes.
- Do not expose raw event payload JSON, staging local paths, raw config values,
  resolved secrets, or local filesystem paths.

## Completed Gates

- `cargo fmt --all -- --check`
- `cargo check -p nako-db --tests`
- `cargo nextest run -p nako-db outbox --no-fail-fast`
- `cargo check -p nako-api --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`
- `cargo nextest run -p nako-api public_openapi --no-fail-fast`
- `cargo nextest run -p nako-api typescript_sdk_excludes_admin_internal_and_secret_surfaces --no-fail-fast`
- `git diff --check`
- `git diff --name-only -- crates/nako-client-protocol`

## Follow-Ons

- Full VFS cache object/failure list route if operators need URI-level cache
  debugging. This workstream intentionally shipped only safe cache counters.
- Event outbox detail route only if list rows and existing webhook attempt
  detail are insufficient.
- Runtime config mutation/editing remains out of scope.
