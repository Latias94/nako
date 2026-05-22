# Public API Contract Hardening Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M30 is closed. Nako now has a documented public API v1 contract with
protocol-owned error codes, a compatible `code/message` error envelope,
`/health.version`, `x-nako-api-version`, public/internal route boundary docs,
and route-level evidence for the first public client surface.

## Active Task

- None. The workstream is completed.

## Decisions Since Last Update

- Public API v1 can begin with protocol constants and `/health.version`
  before path or header version negotiation.
- The v1 error envelope remains `code/message` for compatibility.
- Public clients should branch on stable error code values, not messages.
- Admin/internal routes may reuse the baseline error envelope, but only the
  public route subset gets the first client compatibility promise.

## Blockers

- None.

## Next Recommended Action

- Choose a new goal. Strong follow-ons are OpenAPI/SDK generation for
  `nako-client-protocol`, auth/session boundary design, or route path/header
  negotiation when multiple API versions need concurrent support.
