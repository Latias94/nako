# Admin Media Management Context Links - Design

Status: Active
Last updated: 2026-05-29

## Problem

Nako now has separate Media and Admin surfaces in the product frontend, and the
backend already exposes permission-gated Management Context Links at
`/management/context-links`. The remaining product gap is frontend composition:
operators should be able to jump from a media problem to the right management
workflow, while ordinary viewers must not see admin affordances or Admin API
state.

The wrong implementation would hard-code role checks and Admin URLs inside
Media Web. That would duplicate server authority, leak privileged concepts into
viewer UI, and make links drift from actual backend permissions.

## Target State

- `web/` has a small Management Context Link boundary around the generated
  Public Client SDK method.
- Media routes request context links with stable IDs only: library id, item id,
  source id, or playback session id.
- Link visibility and disabled state are driven by backend-computed
  `enabled`, `required_access`, and `disabled_reason`.
- Frontend code maps backend `route_name` values to known `web/` route targets
  or Admin command entrypoints through one explicit resolver.
- Media Web never imports Admin API DTOs or decides admin permissions by itself.
- Admin Web owns broad or mutating actions through existing confirmation and
  mutation flows.
- Disabled or unsupported links are explainable but do not expose raw paths,
  Source Locators, tokens, provider payloads, FFmpeg command lines, or storage
  details.

## Scope

- `web/src/api/public` Management Context Link data-source boundary.
- `web/src/features/media` rendering hooks for item, library, source/version,
  playback/watch, and playback-error contexts.
- `web/src/features/admin` route targets or command handoffs for library scan,
  metadata profile, item metadata refresh, jobs, playback support/runtime, and
  access policy contexts.
- Frontend tests for route mapping, visibility, disabled reasons, redaction,
  and fallback behavior.
- Browser smoke for representative Media-to-Admin and Admin-to-Media
  transitions.

## Non-Goals

- Adding or changing the backend `/management/context-links` contract.
- Adding new Admin API authorization rules.
- Implementing destructive actions directly in Media Web.
- Reopening `apps/admin-web` as the product frontend.
- Desktop native playback, account onboarding, recommendations, or new media
  browse contracts.

## Architecture Direction

The frontend should treat Management Context Links as a product navigation
contract, not an authorization engine.

Data flow:

```text
Media/Admin route context
  -> stable IDs only
  -> Public Client SDK managementContextLinks()
  -> frontend route resolver
  -> safe link group or Admin confirmation flow
```

The resolver is the only place that knows how a backend `route_name` maps to a
`web/` route. Unknown route names must render as unsupported or be omitted with
test coverage, not guessed.

Mutating links such as `library.scan` and `item.metadata_refresh` must route to
an Admin-owned confirmation/command path. Media Web can present the action when
the backend enables it, but it should not call Admin mutations directly.

## Key Inputs

- CSAPA-040 from
  `docs/workstreams/client-surface-and-access-product-architecture/`.
- BMPD-050 from `docs/workstreams/backend-media-product-deepening/`.
- `sdk/typescript/src/index.ts` `managementContextLinks()`.
- `web/src/api/public/media-data-source.ts`.
- `web/src/api/admin/client.ts` and existing Admin mutation data sources.
- `web/src/shell/nako-router.tsx`.

## Risks

- Media Web could accidentally become an Admin API consumer. Block this with
  import tests and data-source boundaries.
- Link labels can imply an action is available when the backend disabled it.
  Render disabled reasons explicitly and keep commands behind Admin ownership.
- Route names can drift from frontend mappings. Keep one resolver and a test
  fixture for every known backend route name.
- Safe IDs can become unsafe if query params start carrying raw paths or
  diagnostics. The resolver must accept only stable IDs and typed route names.
