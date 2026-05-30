# Admin Media Management Context Links - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is newly split from CSAPA-040. It is a frontend execution lane for
the current `web/` product frontend.

The backend contract is already complete:

- Public Client route: `GET /management/context-links`
- SDK method: `NakoClient.managementContextLinks(query)`
- Backend route names:
  `library.scan`, `library.metadata_profile`, `item.metadata_refresh`,
  `jobs.filtered`, `playback.support`, `playback.runtime`,
  `access.library_policies`

## Active Task

- Task ID: AMCL-020
- Owner: codex
- Scope: `web/src/api/public`, route resolver, and focused tests.
- Status: READY

## Next Recommended Action

Implement AMCL-020:

1. Add typed frontend link models around the generated SDK DTOs.
2. Add one route resolver for backend `route_name` values.
3. Test live SDK query params, fixture fallback, unknown route handling, and
   safe output params.
4. Keep Media Web free of Admin API imports.

## Guardrails

- `web/` is the product frontend. Do not implement new product UI in
  `apps/admin-web`.
- Backend link state is authoritative. Frontend should not recompute roles or
  Library Access.
- Mutating actions must enter Admin-owned confirmation/mutation paths.
- Media Web must not render raw paths, Source Locators, provider payloads,
  FFmpeg details, storage handles, tokens, or secrets.

## Parallelism

AMCL-020 should be single-owner because it defines the shared resolver. After
AMCL-020, AMCL-030 Media rendering and AMCL-040 Admin handoffs can be split
between workers if the resolver contract is stable.
