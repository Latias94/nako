# Named Playback Profile Preferences

## Goal

Let a self-hosted Nako user manage multiple named playback capability profiles
for their own devices, choose a default profile, and keep playback startup able
to fall back to the saved default when a request does not send explicit client
capability fields.

This is the productized successor to the current single
`/users/me/playback-profile` preference. The single route remains as a
compatibility facade for the current user's default profile while the backend
model moves to named profiles.

## Requirements

- Add a current-user scoped named playback profile model.
- Store resolved effective playback capabilities, not unresolved request JSON.
- Keep all public routes under `/users/me`; neither request nor response
  payloads may accept or expose `principal_id`, `user_id`, source locators,
  local paths, bearer tokens, FFmpeg/runtime facts, hardware probe facts, or
  Admin playback policy rows.
- Preserve the existing `GET|PUT|DELETE /users/me/playback-profile` route as a
  compatibility facade for the current user's default named profile.
- Add a bounded current-user collection API:
  - `GET /users/me/playback-profiles`
  - `POST /users/me/playback-profiles`
  - `GET /users/me/playback-profiles/{profile_id}`
  - `PUT /users/me/playback-profiles/{profile_id}`
  - `DELETE /users/me/playback-profiles/{profile_id}`
- Use plural capability-set request fields for stored profile bodies:
  `containers`, `video_codecs`, and `audio_codecs`.
- Continue to reject unsupported additive HLS enum values at the HTTP boundary
  before persistence.
- Make the first created profile the default if the current principal has no
  default profile.
- Allow default selection through profile writes. At most one profile per
  current principal may be default.
- Deleting the default profile leaves the principal with no default profile;
  the server must not silently promote another device profile.
- Playback decision, Direct Play startup/preflight, Remux startup/preflight,
  and HLS playlist startup continue to use the current principal's default
  saved profile only when the request sends no explicit capability query field.
- Existing browser playback tickets, renderer transport URLs, and
  playback-session-bound media requests continue to use the client context
  carried by their ticket or session.
- Keep the old single-profile repository/table only as a migration or
  compatibility bridge while callers move to the named profile repository.
  New behavior should depend on the named repository.

## Acceptance Criteria

- [ ] Current-user named playback profile CRUD is exposed in Public Client
      protocol, OpenAPI, generated SDKs, client-core, client, and server routes.
- [ ] Named profile list is bounded and scoped to the authenticated current
      principal.
- [ ] Profile create/update responses return resolved
      `ClientPlaybackCapabilitiesDto` payloads plus profile identity,
      display name, default flag, `updated_at`, and `version`.
- [ ] Existing `/users/me/playback-profile` responses stay compatible:
      missing default returns `preference: null`; a default profile maps to the
      existing resolved preference DTO.
- [ ] Default profile fallback is used for new playback planning/startup
      requests only when no explicit capability query fields are present.
- [ ] Explicit capability query fields remain authoritative and are not merged
      with a saved default profile.
- [ ] Invalid stored capability JSON fails loudly through repository/API
      mapping instead of silently falling back to defaults.
- [ ] SQLite and PostgreSQL migrations preserve existing single preferences by
      creating a default named profile during upgrade.
- [ ] Repository contract tests cover create, list, read, update, delete,
      default selection, idempotent delete, and legacy preference migration.
- [ ] Server route tests cover absent/list/create/read/update/delete/default
      behavior and unsupported additive HLS enum rejection.
- [ ] API contract tests prove Admin surfaces do not leak the new Public Client
      routes.

## Technical Approach

### Data Model

Introduce named playback profiles as the source-of-truth persistence model.

Suggested record fields:

- `profile_id`: stable opaque ID owned by the server.
- `principal_id`: authenticated current-user principal namespace.
- `name`: user-visible display name.
- `capabilities_json`: resolved effective capability JSON.
- `is_default`: whether this is the current principal's default fallback.
- `updated_at_ms`: server timestamp.
- `version`: optimistic response version incremented on replacement writes.

SQLite should keep `capabilities_json` as validated `TEXT`; PostgreSQL should
store it as `jsonb`. The storage layer should enforce current-principal
uniqueness and at-most-one default in repository logic even if dialect-specific
partial indexes differ.

### API Shape

The collection route is intentionally current-user scoped:

```http
GET /users/me/playback-profiles
POST /users/me/playback-profiles
GET /users/me/playback-profiles/{profile_id}
PUT /users/me/playback-profiles/{profile_id}
DELETE /users/me/playback-profiles/{profile_id}
```

The existing single route remains:

```http
GET /users/me/playback-profile
PUT /users/me/playback-profile
DELETE /users/me/playback-profile
```

`PUT /users/me/playback-profile` should upsert the current default named
profile, creating a default profile named `Default` when no default exists.
`DELETE /users/me/playback-profile` deletes the current default named profile,
matching the old "remove saved preference" behavior.

### Playback Use

For this task, playback planning uses only the default saved profile fallback
already introduced by the single-profile preference work. Per-request
`playback_profile_id` selection is explicitly deferred so the CRUD and default
semantics can stabilize first.

### Compatibility And Migration

The existing `user_playback_profile_preferences` table represents one default
profile per principal. The named-profile migration should copy those rows into
the new named table with a safe server-owned ID, name `Default`, and
`is_default = true`. After migration, application code should read and write
through the named-profile repository. The old table may remain for rollback
safety in this task, but should not remain on the hot path.

## Decision (ADR-lite)

**Context**: The current implementation persists one resolved playback
preference per principal. That unblocks default playback fallback, but it does
not match real self-hosted use where the same user may watch from a browser, a
TV app, a tablet, or a renderer with different codec/HLS capabilities.

**Decision**: Add named current-user playback profiles as the source-of-truth
model and treat `/users/me/playback-profile` as a default-profile
compatibility facade.

**Consequences**:

- Public clients get a real profile management surface without exposing
  internal principal IDs.
- Playback startup keeps its current stable fallback behavior while profile
  CRUD is introduced.
- Storage and API contracts grow across multiple crates, so implementation must
  land with repository, OpenAPI, SDK, server route, and playback fallback tests.
- Per-request profile selection remains a follow-on, which keeps this task from
  mixing profile management with playback-session/ticket semantics.

## Implementation Units

1. **Persistence and domain contract**
   - Add named profile records and repository trait in `nako-core`.
   - Add SQLite/PostgreSQL migrations and adapters in `nako-db`.
   - Add repository contract tests including migration from the old preference
     table.

2. **Public Client contract**
   - Add DTOs and route inventory in `nako-client-protocol`.
   - Add OpenAPI schemas/routes and generated TypeScript/Kotlin SDK output in
     `nako-api`.
   - Add client-core/client request builders as needed.

3. **Server application and HTTP routes**
   - Add current-user CRUD handlers and app-service methods.
   - Rewire the existing single `/users/me/playback-profile` facade to the
     named default profile model.
   - Preserve capability resolver validation and redaction-safe DTO mapping.

4. **Playback default fallback**
   - Keep existing fallback semantics but make the default lookup use the named
     repository.
   - Preserve explicit query-field precedence and ticket/session isolation.

5. **Cleanup and specification**
   - Remove or de-hot-path obsolete single-preference helpers once callers move.
   - Update specs and HTTP docs for named profiles and default facade behavior.

## Out Of Scope

- Per-request `playback_profile_id` selection for playback decision/stream/HLS
  startup.
- Admin-managed global device profile catalogs.
- Sharing profile definitions across users or households.
- Client UI for profile management.
- New runtime diagnostics, FFmpeg behavior, hardware policy, or Admin playback
  policy changes.
- Deleting the old single-profile database table in the same task.

## Technical Notes

- Existing behavior documented in `docs/api/HTTP_API.md`:
  `/users/me/playback-profile` stores one default preference and playback uses
  it only when explicit capability query fields are absent.
- Existing server spec:
  `.trellis/spec/nako-server/backend/http-api-patterns.md` defines current-user
  playback state access boundaries, saved preference fallback precedence, and
  ticket/session exclusions.
- Existing API/client specs:
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` and
  `.trellis/spec/nako-client-protocol/backend/index.md` define the current
  single-profile contract that must remain compatible.
- Existing DB spec:
  `.trellis/spec/nako-db/backend/database-guidelines.md` explicitly scopes the
  old `user_playback_profile_preferences` table to one row per principal and
  says it does not manage multiple named profiles. This task supersedes that
  limitation with a new named source-of-truth model.
- Playback architecture:
  `docs/architecture/PLAYBACK.md` identifies device capability profiles as the
  active lane for more accurate Direct/Remux/Transcode decisions.
