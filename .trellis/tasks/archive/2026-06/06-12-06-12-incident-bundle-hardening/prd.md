# diagnostics: incident bundle hardening

## Goal

Strengthen the Admin-only incident bundle workflow so operators can trust its
access boundary, quickly triage bundle quality from the Admin Web page, and keep
the diagnostics route from inflating the initial Admin Web bundle.

## Requirements

* Add focused server auth smoke coverage for
  `GET /admin/v1/diagnostics/incident-bundle`, including missing credentials
  and non-admin principals.
* Add an Admin Web section status summary for the incident bundle page using
  only already-safe bundle projections.
* Keep JSON copy/download export on an explicit safe projection and continue to
  reject unsafe injected raw fields.
* Convert the incident bundle route component to lazy loading so the diagnostics
  page is not part of the first Admin Web route chunk.

## Acceptance Criteria

* [ ] Non-authenticated incident bundle requests return the existing `401`
      bearer challenge response.
* [ ] Non-admin authenticated incident bundle requests return the existing
      Admin `403` response.
* [ ] Admin authenticated incident bundle requests still return the redacted
      JSON-only bundle.
* [ ] Admin Web renders a compact health/status summary for core bundle
      sections in English and zh-Hans.
* [ ] Admin Web tests continue proving rendered/exported bundle data omits
      unsafe raw fields and values.
* [ ] Incident bundle page code is lazy-loaded from the route declaration.

## Definition of Done

* Focused Rust and Admin Web tests pass.
* `npm run check --prefix apps/admin-web` passes.
* `npm run build --prefix apps/admin-web` passes or any remaining bundle warning
  is explicitly documented.
* `cargo fmt --all -- --check` and `git diff --check` pass.
* Task is validated, archived, and committed with a Conventional Commit.

## Technical Approach

Use existing Admin route auth layers rather than adding new handler checks.
Add a focused route test around the current incident bundle endpoint. In Admin
Web, keep network access behind `AdminDataSource`, add a route-local summary
helper over the safe response DTO, and preserve the existing explicit export
projection.

## Decision (ADR-lite)

Context: The incident bundle is now exportable, but access boundary regression
and triage readability need focused coverage before expanding support tooling.

Decision: Harden the existing JSON-only Admin route and page instead of adding
zip/upload/share workflows.

Consequences: Operators get better local diagnostics without introducing new
support transport, storage, or public API surface. Future bundle viewers can
reuse the section summary model if needed.

## Out of Scope

* Zip archives, upload/share support transport, support ticket integration, or
  remote log streaming.
* New incident bundle DTO fields or generated contract changes.
* Public Client, OpenAPI, or SDK exposure.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  * `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  * `.trellis/spec/nako-server/backend/http-api-patterns.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
* Sensitive data families must stay omitted: raw paths, locators, bearer tokens,
  token env names, credentials, FFmpeg command lines, provider payloads,
  backend URLs/query strings, raw job payloads, and unbounded logs.
