# Addon Token Grants Side Effects TODO

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] ATGSE-010 [owner=codex] [deps=none] [scope=docs/workstreams/addon-token-grants-side-effects,docs/workstreams/addons-automation,docs/workstreams/README.md]
  Goal: Open the focused ARF-006 lane with problem, target state, non-goals,
  gates, and first executable audit task.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `docs/workstreams/addons-automation/TODO.md`.
  Handoff: Continue with ATGSE-020 before changing addon token or grant code.

## M1 - Current Boundary Audit

- [ ] ATGSE-020 [owner=codex] [deps=ATGSE-010] [scope=crates/taru-addon-protocol,crates/taru-core/src/addon.rs,crates/taru-core/src/repository/addon.rs,crates/taru-db/src/addons.rs,crates/taru-db/migrations,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-api/src/extension.rs,docs]
  Goal: Audit current addon manifest auth, registration persistence, granted
  scope semantics, route/API behavior, and tests; classify gaps for Addon
  Token lifecycle, accepted grants, library scope, and Addon Side Effect
  intake.
  Validation: `rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs`; `git diff --check`.
  Review: audit must decide whether ADR 0020 needs a narrow amendment before
  code changes.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with ATGSE-030 once token/grant record ownership is chosen.

## M2 - Token And Grant Contract

- [ ] ATGSE-030 [owner=unassigned] [deps=ATGSE-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-api/src/extension.rs,docs/api]
  Goal: Implement or design-to-code the Addon Token issuance, revocation,
  rotation, redacted response, secret hash storage, accepted Addon Permission,
  and Library-Scoped Addon Grant contract.
  Validation: `cargo check -p taru-core --tests`; `cargo check -p taru-db --tests`; `cargo nextest run -p taru-db addon --no-fail-fast`; focused server addon route tests.
  Review: review-workstream before accepting schema/API changes.
  Evidence: migration, repository, app-service, API docs, and tests proving
  issued tokens are only shown once and persisted secrets are not plaintext.
  Handoff: Continue with ATGSE-040 after runtime access checks are enforceable.

## M3 - Runtime Addon Principal Enforcement

- [ ] ATGSE-040 [owner=unassigned] [deps=ATGSE-030] [scope=crates/taru-server,crates/taru-api,crates/taru-core]
  Goal: Add an Addon principal authentication path for addon-to-Taru calls that
  resolves a token into addon registration identity, accepted permissions, and
  optional Media Library grant set.
  Validation: focused `cargo nextest run -p taru-server addon --no-fail-fast`;
  `cargo check -p taru-api --tests`.
  Review: ensure addon tokens cannot access Public Client or Admin API behavior
  outside their explicit route family.
  Evidence: HTTP tests for missing token, invalid token, revoked token,
  missing permission, wrong library, and valid library-scoped grant.
  Handoff: Continue with ATGSE-050 once protected routes can depend on addon
  principal context.

## M4 - Addon Side Effect Intake Proof

- [ ] ATGSE-050 [owner=unassigned] [deps=ATGSE-040] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,docs/api]
  Goal: Implement the smallest Addon Side Effect intake proof that validates
  actor, target, permission, library scope, idempotency key, provenance, safe
  error mapping, and audit persistence before any canonical/library-file write
  is applied.
  Validation: focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`; relevant `taru-db` tests; `git diff --check`.
  Review: review-workstream for side-effect semantics and leakage risk.
  Evidence: tests for accepted intake, denied permission, wrong library,
  revoked token, duplicate idempotency key, malformed target, and redacted
  response.
  Handoff: Split concrete metadata/artwork/subtitle/Library File Write handlers
  if they exceed the proof scope.

## M5 - Docs, Gates, And Closeout

- [ ] ATGSE-060 [owner=planner] [deps=ATGSE-050] [scope=docs/workstreams/addon-token-grants-side-effects,docs/api,docs/adr]
  Goal: Close the lane or split follow-ons for concrete Addon Side Effect
  handlers.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane from metadata/artwork/subtitle/library-file
  writes only after token/grant/intake evidence is stable.

