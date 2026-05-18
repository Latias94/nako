# Addon Token Grants Side Effects Handoff

Status: Active
Last updated: 2026-05-18

## Current State

ATGSE-040 is complete. Taru now has first-class Addon Token issuance,
rotation, revocation, redacted inspection, accepted Addon Permission /
Library-Scoped Addon Grant storage, and an addon-owned runtime principal
authorization seam.

The lane still does not have Addon Side Effect intake. That remains the next
slice.

## Active Task

- Task ID: ATGSE-050
- Owner: unassigned
- Files: `crates/taru-core`, `crates/taru-db`,
  `crates/taru-server/src/app/addons.rs`,
  `crates/taru-server/src/http/addons.rs`, `crates/taru-api/src/extension.rs`,
  `docs/api`
- Validation: focused `cargo nextest run -p taru-server addon_side_effect
  --no-fail-fast`; relevant `taru-db` tests; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: run review-workstream for side-effect semantics and leakage risk
- Evidence: tests for accepted intake, denied permission, wrong library,
  revoked token, duplicate idempotency key, malformed target, and redacted
  response

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

- Addon Token issuance, rotation, and revocation are admin-only management
  operations under `/admin/v1/addons/{addon_id}`.
- Raw tokens are returned only on issue and rotate.
- Persisted token verifier material is hashed, and token rotation is bound to
  the owning addon in the DB layer.
- Accepted grants are stored separately from registration scope strings and may
  be global or Library-Scoped.
- Addon runtime route families are separate from the admin bearer middleware.
  `/addon/v1/access-check` resolves an Addon Token into an Addon principal and
  enforces accepted permission plus library scope.
- Addon Tokens cannot authenticate `/admin/v1/*` routes.

## Blockers

- None known.

## Next Recommended Action

- Run ATGSE-050: implement the smallest Addon Side Effect intake proof on top
  of the Addon principal seam. Persist actor, target, permission, library scope,
  idempotency key, provenance, validation result, and safe response state before
  adding concrete metadata/artwork/subtitle/Library File Write handlers.
