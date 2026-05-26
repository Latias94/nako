# Credential Session Auth - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

This workstream is complete. Nako now has backend local password credentials,
durable user sessions, Admin API local password provisioning, Public Client
login/current-account/logout, and Bearer session resolution into
`AuthenticatedPrincipal`.

The configured bootstrap administrator token remains supported for first setup
and automation. Local session tokens are opaque client credentials and only
their hashes are stored.

## Active Task

- Task ID: none
- Status: complete
- Review: no blocking workstream or code-quality findings remain.
- Evidence: see `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Session tokens will be opaque Bearer-compatible credentials.
- The configured bootstrap administrator token remains supported for setup and
  automation.
- Public registration, cookies, invitations, SSO, and UI are follow-ons.
- Baseline migration consolidation remains allowed while Nako has no production
  users.
- Admin API may provision local credentials for existing users, but Public
  Client owns login/current-account/logout.
- Public account responses expose current user/session data only; password
  hashes, token hashes, role mutation internals, and bootstrap token state stay
  out of Public Client DTOs.

## Blockers

- None.

## Next Recommended Action

- Open a focused frontend or UX lane only after the current frontend work is
  ready to connect to this backend contract. Recommended splits:
  - Admin Web account credential controls.
  - Media Web login/account switching.
  - Browser cookie transport on top of the session authority.
  - Invitation onboarding and account recovery.
  - Management Context Links between media browsing and admin operations.
