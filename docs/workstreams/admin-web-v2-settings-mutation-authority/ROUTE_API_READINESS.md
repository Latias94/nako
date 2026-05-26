# Settings Mutation Route/API Readiness

Status: ASM-020 complete
Last updated: 2026-05-26

## Current Baseline

- Admin Web V2 `/settings` is route-owned and read-only.
- `SettingsPage` loads data through `AdminDataSource.loadSettings()`.
- `AdminDataSource.loadSettings()` calls `AdminApiClient.getSystemConfig()`.
- `AdminApiClient.getSystemConfig()` calls `GET /admin/v1/system/config`.
- `crates/nako-server/src/http/admin.rs` registers
  `GET /admin/v1/system/config` and no adjacent system-config mutation route.
- `docs/api/HTTP_API.md` documents `GET /admin/v1/system/config` as sanitized
  diagnostics.
- `crates/nako-server/src/config.rs` has `load_config()` and
  `example_config()`, but no accepted server-config save/update path.

## Existing Mutation Nearby

- Media Library Metadata Profile editability is handled through the library
  boundary, not the global settings route:
  - `GET /admin/v1/libraries/{library_id}/metadata-profile`
  - `PUT /admin/v1/libraries/{library_id}/metadata-profile`
- That path persists to library options and has a separate restart authority
  decision in `metadata-profile-configuration-authority`.

## Missing For Global Settings Editing

- No Admin API route for changing system config.
- No DTO for settings update requests, review plans, validation failures, or
  mutation results.
- No accepted persistence target for process-wide `NakoServerConfig` changes.
- No restart-required/runtime-applied distinction in Admin API.
- No audit/event model for settings changes.
- No UI contract for preventing deterministic mock fallback from pretending to
  mutate real server state.

## Candidate First Slices

| Candidate | Pros | Risks | Initial decision |
| --- | --- | --- | --- |
| Network exposure policy summary controls | High operator value; existing diagnostics/readiness model. | URLs, origins, trusted proxy sources, and tunnel tokens are sensitive; external endpoint changes may require restart and TOML authority. | Needs deeper ASM-020 decision before implementation. |
| Worker/runtime budget controls | Mostly numeric, no secrets, easier validation. | Runtime services may not support live resize; persistence still unresolved. | Possible first mutation if runtime-only or restart-required semantics are accepted. |
| Metadata provider runtime policy | Existing diagnostics and provider concepts. | Secret refs, proxy URLs, provider credentials, and rate-limit behavior are sensitive. | Defer unless scoped to non-secret policy fields. |
| Staging/playback/artwork policy | Useful operational controls, mostly numeric/boolean. | Paths/roots and active runtime behavior can be sensitive or restart-bound. | Possible after source-of-truth decision. |
| Raw TOML editor | Broad coverage. | Leaks secrets/paths/URLs, high corruption risk, poor validation UX. | Rejected for this lane. |

## ASM-020 Questions

- Which settings field group has a safe source of truth today?
- Are changes applied immediately, on restart, or persisted as desired state for
  next startup?
- Where does the mutation live if TOML remains the startup source?
- What is the idempotency key or conflict model?
- What facts can the UI show without exposing raw config material?
- Does this need a backend configuration-authority workstream before UI work?

## Initial Conclusion

Admin Web V2 must not add editable controls yet. The next executable task is a
focused ASM-020 decision that either selects a narrow mutation slice backed by a
real Admin API route or splits a backend source-of-truth lane before any save UI
is built.

## ASM-020 Decision

Status: DONE_WITH_CONCERNS.

No global settings field group is safe to expose as an Admin Web save control
today.

Evidence:

- `NakoServerConfig` is loaded from TOML at command startup through
  `load_config()`.
- `config.rs` has no save/update path for global server config.
- `NakoAppComposition` stores an immutable config clone.
- `NakoRuntimeResources::build()` creates scan, metadata, and webhook
  semaphores from startup config.
- `StorageBackendRegistry` and `LibraryStorageBackend` create playback stream
  and remote-stage semaphores from startup config.
- `ManagedArtworkAppService` and the artwork ingest pipeline clone
  `ArtworkConfig` at service construction.
- `build_router()` copies `AuthConfig` and `NetworkAccessConfig` into auth and
  network middleware state.
- `nako-db` has no global settings repository or migration.

Therefore:

- Network/auth changes are not hot-applied with the current router.
- Runtime budget changes are not hot-applied without deliberate resource
  replacement/resizing.
- Persisted global setting changes have no accepted restart merge semantics.
- A frontend save button would be fake unless a backend configuration-authority
  route exists first.

## Split Follow-On

Opened `docs/workstreams/admin-settings-configuration-authority/`.

That lane owns:

- first settings field-group selection;
- persisted versus runtime-only source-of-truth semantics;
- TOML/admin/runtime precedence;
- restart-required versus hot-applied effect reporting;
- backend persistence/runtime implementation;
- Admin API route shape and generated contract.

This settings mutation lane remains active but blocked for implementation work
until that backend route exists. Admin Web may still improve read-only
affordances, but it must not expose save controls for global settings before
ASCA hands back a real Admin API mutation surface.

## ASCA Handoff

Status: backend predecessor complete with concerns.

`docs/workstreams/admin-settings-configuration-authority/` implemented and
closed the first safe settings mutation route:

- `GET /admin/v1/settings/metadata/raw-cache`
- `PUT /admin/v1/settings/metadata/raw-cache`

The route owns only metadata raw cache retention settings:

- `metadata.raw_cache_retention_ms`
- `metadata.maintenance.raw_cache_cleanup_on_startup`

Admin values are persisted as desired-state overrides. TOML is merged first at
startup, then the Admin override is applied before services are built. PUT can
report `effect = requires_restart`; after restart, the same override reports
`effect = active`.

ASM-040 may build Admin Web controls for this field group only. All other
global settings remain read-only until a separate backend authority exists.
