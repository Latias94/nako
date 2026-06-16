# feat: Admin playback support client capability evidence

## Goal

Extend the existing Admin playback support evidence response so an operator can
see the redaction-safe client capability facts that were persisted with a
playback session. This closes the support gap between "runtime is healthy" and
"this playback request asked for these concrete capability limits".

## Requirements

* Add an optional `client` section to `AdminPlaybackSupportEvidenceResponse`.
* Populate it only when support evidence is requested with a playback session
  whose `client_capabilities_json` can be parsed.
* Expose normalized, safe facts only:
  * `has_client_capabilities`
  * normalized known `device_family`
  * `profile_version`
  * recognized `profile_family`
  * `direct_play`
  * container/video/audio codec lists
  * max bitrate/resolution/audio-channel limits
  * HDR/subtitle booleans
  * HLS output policy/container
* Unknown raw `device_family` values must not be echoed. They may become
  `profile_family: "unknown"` with `device_family: null`.
* Malformed or legacy capability JSON must not fail the support route. Return
  `client: null` while keeping `session`, `source`, `runtime`, and `redaction`.
* Do not expose user agents, paths, source locators, request keys, bearer
  tokens, FFmpeg argv, stderr, or raw capability JSON.
* Regenerate generated Admin TypeScript contracts from `nako-api`.
* Keep Public Client DTOs, OpenAPI, Public SDKs, server playback behavior,
  planner decisions, and database schema unchanged.

## Acceptance Criteria

* `GET /admin/v1/playback/support?session_id=...` returns `client` with safe
  capability facts for a session with parseable client capabilities.
* The same route returns `client: null` for no session, no capabilities, or
  malformed capability JSON.
* Unknown raw `device_family` is not present in serialized support evidence.
* API serialization and server route tests cover safe projection and redaction.
* Generated Admin contracts under `apps/admin-web` and `web` contain the new
  support client evidence shape.

## Out Of Scope

* New Admin route.
* Admin Web UI rendering changes beyond generated contract refresh.
* Re-planning a historical playback decision from persisted facts.
* Persisting decision reports or client profile catalog snapshots.
* Public Client/OpenAPI/SDK contract changes.

## Technical Approach

Reuse the existing Admin playback profile normalization semantics:

* Add an `AdminPlaybackSupportClientEvidence` DTO in
  `crates/nako-api/src/admin/playback.rs`.
* Parse `PlaybackSessionRecord.client_capabilities_json` into
  `nako_playback::ClientPlaybackCapabilities`.
* Normalize device family through `normalize_playback_device_family` and
  `PlaybackProfileFamily::from_device_family`.
* For known profile families, expose the normalized family/device facts.
  For unknown families, expose only `profile_family: "unknown"`.
* Map HLS output values to the existing Admin HLS policy/container enums.
* Wire the DTO in `crates/nako-server/src/http/admin.rs` using the existing
  `PlaybackSupportEvidenceContext`.

## Relevant Files

* `crates/nako-api/src/admin/playback.rs`
* `crates/nako-api/src/admin_contract.rs`
* `crates/nako-core/src/repository/playback_session.rs`
* `crates/nako-db/src/sqlite/playback.rs`
* `crates/nako-db/src/postgres/playback_runtime.rs`
* `crates/nako-db/src/contract_tests.rs`
* `apps/admin-web/src/adminApi/generated/contract.ts`
* `web/src/api/admin/generated/contract.ts`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/tests/system.rs`
* `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
* `.trellis/spec/nako-api/backend/quality-guidelines.md`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `.trellis/spec/nako-db/backend/index.md`
* `.trellis/spec/nako-db/backend/quality-guidelines.md`
