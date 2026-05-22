# Admin Web Addon Credential and Grant Onboarding Handoff

Status: Completed
Last updated: 2026-05-22

## Current State

The workstream is complete. Generated Admin API contract now includes explicit
one-time Addon Token issue/rotation DTOs, token revoke DTO, and grant
replacement request types. Admin Web client/data-source actions exist for
issue/rotate/revoke/replace grants, and the UI renders token controls, grant
replacement, one-time token notice, and enable readiness checklist.

## Next Task

Recommended follow-on:

- Secret Reference configuration UX; or
- use the new `nako-official-addons` metadata scraper skeleton to drive a
  real Addon end-to-end smoke flow after credentials/grants are configured.

## Boundaries To Preserve

- Raw tokens are one-time action outputs only.
- Grants are accepted authority, not manifest declarations.
- Taru does not manage sidecar installation or process lifecycle.
