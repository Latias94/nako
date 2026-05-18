# Addon Token Grants Side Effects Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

ATGSE-020 is complete. The audit found that current addon support covers
manifest registration, outbound Taru-to-Addon resource calls, and coarse
registration `granted_scopes`, but has no addon-to-Taru token principal,
accepted grant model, Library-Scoped Addon Grant storage, or Addon Side Effect
intake.

No token, grant, schema, API, or runtime behavior has been changed yet.

## Active Task

- Task ID: ATGSE-030
- Owner: unassigned
- Files: `crates/taru-core`, `crates/taru-db`,
  `crates/taru-server/src/app/addons.rs`,
  `crates/taru-server/src/http/addons.rs`,
  `crates/taru-api/src/extension.rs`, `docs/api`
- Validation: `cargo check -p taru-core --tests`; `cargo check -p taru-db
  --tests`; `cargo nextest run -p taru-db addon --no-fail-fast`; focused server
  addon route tests
- Status: NEEDS_CONTEXT
- Review: run review-workstream before accepting schema/API changes
- Evidence: migration, repository, app-service, API docs, and tests proving
  issued tokens are only shown once and persisted secrets are not plaintext

## Decisions Since Last Update

- Open a new focused workstream instead of continuing to hide ARF-006 as one
  unchecked Post-M5 TODO in `addons-automation`.
- Treat Addon Tokens as addon principals, not admin credentials.
- Keep manifest-requested permissions separate from accepted runtime grants.
- Require Library-Scoped Addon Grants before enabling protected library writes
  unless a global grant is explicitly accepted.
- Route protected mutations through Addon Side Effect intake before concrete
  metadata, artwork, subtitle, or Library File Write handlers expand.
- ADR 0020 already covers the strategic direction. No ADR amendment is required
  before ATGSE-030 unless the implementation chooses OAuth-first authorization,
  broad Admin API reuse, or direct storage/file authority.
- Keep existing `AddonAuth` as outbound Taru-to-Addon authentication. Addon
  Token is a separate inbound addon-to-Taru credential.
- Do not overload `AddonRegistrationRecord.granted_scopes` for token-bound
  accepted permissions. Add first-class token and accepted-grant records.

## Blockers

- None known.

## Next Recommended Action

- Run ATGSE-030: design-to-code the Addon Token and accepted-grant contract.
  Start with core records, DB migrations/repository methods, and admin/API
  shapes for issue/rotate/revoke/redacted inspection. Keep addon-principal
  runtime route enforcement for ATGSE-040.
