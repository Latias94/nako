# Addon Token Grants Side Effects TODO

Status: Completed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] ATGSE-010 [owner=codex] [deps=none] [scope=docs/workstreams/addon-token-grants-side-effects,docs/workstreams/addons-automation,docs/workstreams/README.md]
  Goal: Open the focused ARF-006 lane with problem, target state, non-goals,
  gates, and first executable audit task.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `docs/workstreams/addons-automation/TODO.md`.
  Handoff: Continue with ATGSE-020 before changing addon token or grant code.

## M1 - Current Boundary Audit

- [x] ATGSE-020 [owner=codex] [deps=ATGSE-010] [scope=crates/nako-addon-protocol,crates/nako-core/src/addon.rs,crates/nako-core/src/repository/addon.rs,crates/nako-db/src/addons.rs,crates/nako-db/migrations,crates/nako-server/src/app/addons.rs,crates/nako-server/src/http/addons.rs,crates/nako-api/src/extension.rs,docs]
  Goal: Audit current addon manifest auth, registration persistence, granted
  scope semantics, route/API behavior, and tests; classify gaps for Addon
  Token lifecycle, accepted grants, library scope, and Addon Side Effect
  intake.
  Validation: `rg "Addon|addon|scope|token|grant|manifest" crates/nako-addon-protocol crates/nako-core crates/nako-db crates/nako-server crates/nako-api docs`; `git diff --check`.
  Review: no ADR amendment is required if ATGSE-030 follows ADR 0020; split an
  ADR only for OAuth-first, broad Admin API reuse, or direct storage authority.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with ATGSE-030. Use first-class token and accepted-grant
  records; do not overload registration `granted_scopes`.

## M2 - Token And Grant Contract

- [x] ATGSE-030 [owner=codex] [deps=ATGSE-020] [scope=crates/nako-core,crates/nako-db,crates/nako-server/src/app/addons.rs,crates/nako-server/src/http/addons.rs,crates/nako-api/src/extension.rs,docs/api]
  Goal: Implement or design-to-code the Addon Token issuance, revocation,
  rotation, redacted response, secret hash storage, accepted Addon Permission,
  and Library-Scoped Addon Grant contract.
  Validation: `cargo check -p nako-core --tests`; `cargo check -p nako-db --tests`; `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests`; `cargo nextest run -p nako-db addon --no-fail-fast`; `cargo nextest run -p nako-server addon --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: self-review completed against the workstream and ADR 0020 before
  verification; no blocking findings remained.
  Evidence: migration, repository, app-service, API docs, and tests proving
  issued tokens are only shown once and persisted secrets are not plaintext.
  Handoff: Continue with ATGSE-040 after runtime access checks are enforceable.

## M3 - Runtime Addon Principal Enforcement

- [x] ATGSE-040 [owner=codex] [deps=ATGSE-030] [scope=crates/nako-server,crates/nako-api,crates/nako-core,crates/nako-db,docs/api]
  Goal: Add an Addon principal authentication path for addon-to-Nako calls that
  resolves a token into addon registration identity, accepted permissions, and
  optional Media Library grant set.
  Validation: `cargo check -p nako-core --tests`; `cargo check -p nako-db --tests`; `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests`; focused `cargo nextest run -p nako-server addon --no-fail-fast`; `cargo nextest run -p nako-db addon --no-fail-fast`; `cargo nextest run -p nako-server http::tests::system::bearer_auth --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: addon tokens are accepted only by the addon-owned runtime route family
  and are rejected by Admin API bearer auth.
  Evidence: HTTP tests for missing token, invalid token, revoked token,
  missing permission, wrong library, and valid library-scoped grant.
  Handoff: Continue with ATGSE-050 once protected routes can depend on addon
  principal context.

## M4 - Addon Side Effect Intake Proof

- [x] ATGSE-050 [owner=codex] [deps=ATGSE-040] [scope=crates/nako-core,crates/nako-db,crates/nako-server,crates/nako-api,docs/api]
  Goal: Implement the smallest Addon Side Effect intake proof that validates
  actor, target, permission, library scope, idempotency key, provenance, safe
  error mapping, and audit persistence before any canonical/library-file write
  is applied.
  Validation: focused `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`; relevant `nako-db` tests; `git diff --check`.
  Review: self-review completed for side-effect semantics and leakage risk;
  route persists rejected intake after a trustworthy addon principal is
  resolved and never returns raw token/hash/payload/provenance/raw paths.
  Evidence: tests for accepted intake, denied permission, wrong library,
  revoked token, duplicate idempotency key, malformed target, and redacted
  response.
  Handoff: Split concrete metadata/artwork/subtitle/Library File Write handlers
  if they exceed the proof scope.

## M5 - Docs, Gates, And Closeout

- [x] ATGSE-060 [owner=planner] [deps=ATGSE-050] [scope=docs/workstreams/addon-token-grants-side-effects,docs/workstreams/addon-protected-writes,docs/workstreams/README.md]
  Goal: Close the lane or split follow-ons for concrete Addon Side Effect
  handlers.
  Validation: `cargo fmt --all -- --check`; `git diff --check`.
  Review: closeout self-review found no blocking workstream compliance issues;
  concrete apply behavior is intentionally split.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  `docs/workstreams/addon-protected-writes/`.
  Handoff: Lane is closed. Continue with APW-020 in
  `docs/workstreams/addon-protected-writes/` before applying concrete
  metadata, artwork, subtitle, NFO, or Library File Write behavior.
