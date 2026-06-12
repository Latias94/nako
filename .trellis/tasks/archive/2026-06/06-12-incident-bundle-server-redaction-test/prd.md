# Incident Bundle Server Redaction Route Test

## Goal

Add a focused server HTTP test for `GET /admin/v1/diagnostics/incident-bundle` so the real route assembly, Admin-only boundary, server aggregation, and redaction contract are protected beyond DTO serialization tests.

## What I already know

* The JSON-only incident bundle implementation is committed in `ac42a4c6`.
* Existing API DTO tests prove the bundle type serializes without forbidden field-family names.
* Admin Web tests already prove injected unsafe frontend-only fields are not rendered.
* The missing safety net is a `nako-server` HTTP route test that exercises the actual Axum handler and app aggregation path.

## Assumptions

* This task should not change the public or Admin wire contract.
* This task should not regenerate TypeScript contracts.
* The route should stay Admin-only and reuse existing `admin::routes()` auth layering.

## Requirements

* Add a server route test for `/admin/v1/diagnostics/incident-bundle`.
* Build the test app with intentionally unsafe configuration values: token env names, metadata provider secret references, endpoint URLs, proxy sources/origins, WebDAV credentials, local roots, FFmpeg paths, cache paths, and database URLs.
* Seed representative unsafe runtime facts where practical: media source locators/fingerprints, transcode failure text, and durable job input/error payloads.
* Assert the route returns a typed `AdminIncidentBundleResponse` with JSON-only artifact metadata, safe system/network/playback/storage/job sections, and complete redaction flags.
* Assert the raw response body omits forbidden sensitive field families and test fixture secret values.

## Acceptance Criteria

* [ ] `cargo nextest run -p nako-server admin_v1_incident_bundle --no-fail-fast` passes.
* [ ] `cargo check -p nako-server --tests` passes.
* [ ] `cargo fmt --all` and `git diff --check` pass.
* [ ] No API DTO shape or generated contract file changes are introduced.

## Definition of Done

* Focused test added/updated.
* Trellis task metadata records completion.
* Changes are committed with a Conventional Commit.

## Out of Scope

* Zip/archive incident bundle packaging.
* Upload/share transport.
* New incident bundle DTO fields.
* Admin Web changes.
* Public Client SDK/OpenAPI changes.

## Technical Approach

Add the test in `crates/nako-server/src/http/tests/system.rs`, near the existing system config and playback support evidence redaction tests, because this is an Admin diagnostics route using the same router helpers and redaction fixtures.

## Technical Notes

* Read `.trellis/spec/nako-server/backend/http-api-patterns.md` and `.trellis/spec/nako-server/backend/quality-guidelines.md`.
* Read `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` and `.trellis/spec/nako-api/backend/quality-guidelines.md`.
* Follow the existing `tower::ServiceExt` / `build_router(app)` route test pattern.
