# Addon Token Grants Side Effects Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

This lane is closed. Nako now has first-class Addon Token issuance, rotation,
revocation, redacted inspection, accepted Addon Permission /
Library-Scoped Addon Grant storage, an addon-owned runtime principal
authorization seam, and the first Addon Side Effect intake proof.

Concrete Canonical Metadata, Managed Artwork, subtitle, NFO, and Library File
Write application behavior is split to
`docs/workstreams/addon-protected-writes/`.

## Active Task

- Task ID: ATGSE-060
- Owner: planner
- Files: `docs/workstreams/addon-token-grants-side-effects`,
  `docs/workstreams/addon-protected-writes`, `docs/workstreams/README.md`
- Validation: `cargo fmt --all -- --check`; `git diff --check`
- Status: DONE
- Review: closeout self-review found no blocking findings
- Evidence: ATGSE-050 code/tests/docs, ATGSE-060 closeout evidence, and
  `docs/workstreams/addon-protected-writes/`

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
- Keep existing `AddonAuth` as outbound Nako-to-Addon authentication. Addon
  Token is a separate inbound addon-to-Nako credential.
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
- `/addon/v1/side-effects` persists an Addon Side Effect intake record before
  any canonical metadata or Library File Write mutation is applied.
- Side-effect intake records accepted and rejected validation results once a
  trustworthy Addon principal is resolved. Missing/invalid/revoked tokens return
  `401` without creating an addon audit record.
- Idempotency is scoped to `(addon_id, idempotency_key)` and returns the
  existing record on replay.
- Safe responses omit raw Addon Token material, token hashes, payload JSON,
  provenance JSON, source locators, filesystem paths, and raw provider bodies.

## Blockers

- None known.

## Next Recommended Action

- Continue with APW-020 in `docs/workstreams/addon-protected-writes/` to audit
  concrete protected-write seams and choose the first apply target.
