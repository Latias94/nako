# Incident Bundle Operator Usability And Route Inventory Hardening

## Goal

Improve the Admin incident bundle route from a read-only diagnostics page into a safer operator handoff surface: operators can copy or download the redacted JSON artifact, and the generated Admin route inventory remains explicitly protected by focused tests.

## What I Already Know

* `GET /admin/v1/diagnostics/incident-bundle` already exists and is generated as `NAKO_ADMIN_ROUTES.incidentBundle`.
* Server route tests already cover Admin-only access, safe section composition, and response redaction for the actual Axum route.
* Admin Web already renders the incident bundle read-only through `AdminDataSource.loadIncidentBundle()`.
* Current Admin Web tests cover live loading, fallback, zh-Hans route copy, and unsafe injected field omission from rendered text.
* The remaining operator gap is export usability: the page has refresh but no explicit copy/download affordance for the redacted JSON support artifact.
* A safe export must not serialize arbitrary extra fields injected into runtime objects. It should project the known generated DTO shape before copying or downloading.

## Requirements

* Add Admin Web copy and download actions on `/diagnostics/incident-bundle`.
* The exported artifact must be deterministic, pretty-printed JSON from a known safe projection of `AdminIncidentBundleResponse`.
* The exported artifact must include the Admin/public API versions, generated timestamp, artifact summary, overview, system, network, playback, storage, jobs, and redaction sections.
* The exported artifact must not include unknown extra fields such as injected paths, token values, backend URLs, command lines, job payloads, or raw logs.
* Copy should use `navigator.clipboard.writeText` when available and show a visible success/failure status.
* Download should create a JSON `Blob` with a deterministic filename based on `generated_at_ms`, then revoke the object URL.
* Keep the page read-only. Do not add upload/share transport, zip generation, server-side artifact persistence, or a new API route.
* Add focused route inventory coverage that asserts the incident bundle route is generated, implemented, and not excluded.

## Acceptance Criteria

* [ ] Incident Bundle page renders copy and download controls.
* [ ] Copy writes a safe redacted JSON projection and reports success.
* [ ] Copy failure reports a visible error without leaking sensitive text.
* [ ] Download emits a redacted JSON blob and revokes the generated URL.
* [ ] Frontend tests reject unsafe injected fields in copied/downloaded JSON.
* [ ] Server route inventory test explicitly covers the incident bundle route.
* [ ] `npm run check --prefix apps/admin-web` passes.
* [ ] Focused Admin Web tests for `App`, `adminApi/client`, and `adminApi/dataSource` pass when relevant.
* [ ] `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passes.
* [ ] `cargo fmt --all` and `git diff --check` pass.

## Definition Of Done

* Code and tests are updated.
* Trellis task validates.
* Changes are committed with a Conventional Commit.

## Technical Approach

Use the existing Admin Web page-level pattern rather than adding a new data-source or API contract. Implement a feature-local helper that builds a fresh JSON-safe object with only the named `AdminIncidentBundleResponse` fields, then serialize that object for both copy and download. This keeps the rendering and export redaction boundary in one place without changing the server DTO shape.

Add a focused assertion in the existing `admin_route_inventory` test module so the route inventory gate documents this route's intended generated/Admin-only status.

## Decision (ADR-lite)

**Context**: The current incident bundle page is safe to view but less useful for support handoff, and direct object serialization could accidentally include extra fields if a malformed or future response carries them.

**Decision**: Add frontend-only copy/download controls backed by an explicit safe projection of the generated DTO shape. Keep the server route and API contract unchanged.

**Consequences**: Operators get a practical JSON handoff path now. Zip/upload/share support remains a future task with its own contract. Any future incident bundle DTO additions must be intentionally added to the export projection and tests.

## Out Of Scope

* Zip archives or bundled file attachments.
* Remote support upload/share transport.
* Server-side artifact persistence.
* New incident bundle DTO fields.
* Public Client SDK/OpenAPI exposure.
* Raw logs, paths, provider payloads, commands, storage locators, backend URLs, or job payloads.

## Technical Notes

* Read `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
* Read `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`.
* Read `.trellis/spec/nako-api/backend/quality-guidelines.md`.
* Read `.trellis/spec/nako-server/backend/http-api-patterns.md`.
* Read `.trellis/spec/nako-server/backend/quality-guidelines.md`.
* Follow the generated Admin contract boundary; do not edit `apps/admin-web/src/adminApi/generated/contract.ts`.
