# Playback Runtime Settings Generated Route Contract

## Problem

Nako already implements `GET/PUT /admin/v1/settings/playback/runtime`, with DTOs,
validation, persistence, restart-effect reporting, and server route tests. The
Settings page already displays playback, staging, and runtime policy facts from
system diagnostics, but the dedicated playback runtime settings route is still
excluded from generated Admin Web route constants because the UI did not yet own
the workflow.

That exclusion now hides a real management-plane contract from the generated
Admin API surface. It also keeps Admin Web from calling the typed settings route
even though the backend contract is stable.

## Scope

- Promote `GET/PUT /admin/v1/settings/playback/runtime` into generated Admin
  route constants.
- Remove the matching route exclusion.
- Regenerate Admin TypeScript contracts from `nako-api`.
- Add Admin Web client and data-source methods for reading/updating playback
  runtime settings.
- Add a Settings page workflow for live-backed playback runtime settings using
  explicit edit/prepare/confirm semantics.
- Add Admin Web client, data-source, route, i18n, and redaction tests.

## Non-Goals

- Do not change backend playback runtime setting validation, persistence, or
  restart-effect semantics.
- Do not introduce per-library playback profiles or client capability policies.
- Do not expose paths, device nodes, backend URLs, FFmpeg command lines, tokens,
  or proxy values.
- Do not add scheduled task configuration or a Jellyfin-compatible configuration
  API.

## Acceptance Criteria

- Generated Admin contracts include `NAKO_ADMIN_ROUTES.settingsPlaybackRuntime`.
- Admin route inventory exclusions no longer list `settings/playback/runtime`.
- Admin Web calls the route through `AdminApiClient` and `AdminDataSource`.
- Settings UI can edit the live playback runtime settings only after an
  explicit confirmation step.
- Mock fallback disables save actions honestly instead of fabricating success.
- Focused API/server route inventory tests pass.
- Admin Web check/test pass.
