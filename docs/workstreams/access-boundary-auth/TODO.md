# Access Boundary And Token Authentication TODO

Status: Completed
Last updated: 2026-05-17

## M31.0 Scope And Boundary Baseline

- [x] ABA-010 [owner=planner] [deps=none] [scope=docs/adr, docs/workstreams/access-boundary-auth]
  Goal: Freeze M31 inbound auth problem, target state, integration-secret boundary, non-goals, and validation gates.
  Validation: ADR 0024, DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/access-boundary-auth/DESIGN.md and docs/adr/0024-inbound-token-authentication-boundary.md.
  Handoff: Continue with ABA-020 before adding broader auth models.

## M31.1 Protocol And Config Slice

- [x] ABA-020 [owner=codex] [deps=ABA-010] [scope=crates/taru-client-protocol, crates/taru-api, crates/taru-server/src/config.rs]
  Goal: Add stable auth failure error codes and inbound auth config with safe defaults and redaction.
  Validation: cargo fmt --all -- --check, cargo check -p taru-client-protocol --tests, cargo nextest run -p taru-client-protocol --no-fail-fast, cargo nextest run -p taru-server config --no-fail-fast.
  Evidence: `ClientErrorCode::Unauthorized` and `ClientErrorCode::Forbidden`; `AuthConfig` defaults to `enabled = true` and `token_env = "TARU_ADMIN_TOKEN"`; config tests cover `[auth]`.
  Handoff: Users, sessions, and RBAC remain follow-ons.

## M31.2 HTTP Middleware Slice

- [x] ABA-030 [owner=codex] [deps=ABA-020] [scope=crates/taru-server/src/http.rs, crates/taru-server/src/http/tests]
  Goal: Protect all non-health HTTP routes with bearer-token auth when enabled.
  Validation: cargo check -p taru-server --tests, cargo nextest run -p taru-server http::tests::system --no-fail-fast.
  Evidence: `http::auth::require_auth` protects non-health routes; system tests cover missing token, wrong token, correct token, public health, API-version header, and no token leakage.
  Handoff: Addon/webhook/provider outbound auth is unchanged.

## M31.3 Docs And Route Evidence Slice

- [x] ABA-040 [owner=codex] [deps=ABA-030] [scope=docs/api/HTTP_API.md, docs/development/LOCAL_SETUP.md, crates/taru-server/src/http/tests]
  Goal: Document client bearer token use and prove existing public/admin routes keep their expected behavior under test auth config.
  Validation: cargo nextest run -p taru-server http::tests --no-fail-fast.
  Evidence: HTTP API docs, local setup docs, and route-level tests.
  Handoff: OpenAPI auth scheme, login/session UX, user accounts, and RBAC remain follow-ons.

## M31.4 Closeout

- [x] ABA-050 [owner=planner] [deps=ABA-040] [scope=docs/workstreams/access-boundary-auth]
  Goal: Close M31 with a prompt-to-artifact audit against every explicit requirement.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, cargo nextest run --workspace --no-fail-fast, cargo tree -p taru-client-protocol, git diff --check.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Remaining auth/session, RBAC, OpenAPI, and tunnel follow-ons are recorded in HANDOFF.md.
