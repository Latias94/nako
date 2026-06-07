# Addon Credential Generated Route Contract

## Problem

Nako already implements Addon token and grant Admin routes, and Admin Web already
uses them for Addon onboarding. These routes are still excluded from the
generated Admin Web route inventory, so the frontend derives credential paths by
appending `/tokens` and `/grants` to `addonDetail`.

That leaves an avoidable drift point: the backend route exists, the DTOs exist,
and Admin Web depends on the route, but the generated contract does not name the
route.

## Scope

- Promote Addon token and grant routes into generated Admin API route constants:
  - `GET/POST /admin/v1/addons/{addon_id}/tokens`
  - `POST /admin/v1/addons/{addon_id}/tokens/{token_id}/rotate`
  - `POST /admin/v1/addons/{addon_id}/tokens/{token_id}/revoke`
  - `GET/PUT /admin/v1/addons/{addon_id}/grants`
- Remove the corresponding explicit route exclusions.
- Replace Admin Web credential/grant path derivation with generated route keys.
- Regenerate Admin Web TypeScript contracts from `nako-api`.
- Update focused Admin Web client/data-source tests that assert route paths.

## Non-Goals

- Do not change Addon token issuance, rotation, revocation, hashing, grant
  replacement, or permission semantics.
- Do not expose raw Addon tokens outside one-time issue/rotation responses.
- Do not copy Jellyfin API key shape or route naming.
- Do not promote Addon task-run routes in this slice.

## Acceptance Criteria

- Generated Admin contracts include route keys for Addon tokens, token rotation,
  token revocation, and grants.
- Admin route inventory exclusions no longer list these four routes.
- Admin Web uses generated route constants instead of string-appending from
  `addonDetail`.
- Focused API/server route inventory tests pass.
- Admin Web check/test pass.
- Generated contracts under `apps/admin-web` and `web` are refreshed by the
  generator.
