# Source fingerprint hash job diagnostics first slice

## Goal

Make source fingerprint hash jobs directly inspectable from the existing Admin Jobs surface so operators can filter the durable queue down to the relevant source-hash work after the overview summary reports pressure or failures.

## Requirements

- Keep this slice read-only.
- Reuse the existing `/admin/v1/jobs` route and Admin Web Jobs page instead of adding a source-hash-specific route.
- Preserve the existing Admin Jobs DTO shape: job `id`, `kind`, `status`, `resource_class`, `library_id`, `source_id`, input/summary/error presence booleans, and timestamps.
- Make source fingerprint hash diagnostics reachable with stable filters:
  - `kind=source_fingerprint_hash`
  - `resource_class=disk.scan.source_fingerprint_hash`
  - optional `status`, `library_id`, and `source_id`
- Expose the already-supported `source_id` filter in the Admin Web Jobs filter bar.
- Add a Jobs-page quick filter for source fingerprint hash jobs so operators do not need to remember the exact kind/resource class strings.
- Keep URL search params authoritative and reset `offset` to `0` when a filter changes.
- Keep job input JSON, summary JSON, error bodies, Source Locators, raw Source Fingerprints, raw content hashes, storage URIs, and local paths out of Admin Jobs responses and rendered UI.
- Add focused tests that prove source fingerprint hash jobs can be listed through the existing Jobs route and that raw job payloads remain redacted.

## Acceptance Criteria

- [ ] `GET /admin/v1/jobs?kind=source_fingerprint_hash&resource_class=disk.scan.source_fingerprint_hash` returns only source fingerprint hash jobs.
- [ ] `source_id` remains accepted by the Admin Jobs query parser and can narrow job results.
- [ ] The Admin Jobs JSON body does not include raw source hash job input, summary, locator, path, fingerprint, hash, or storage URI values.
- [ ] Admin Web can apply the source fingerprint hash quick filter and source-id filter through URL-owned search state.
- [ ] Admin Web localized Jobs route copy covers the new controls.
- [ ] Existing overview source fingerprint hash summary remains unchanged in this slice.

## Technical Approach

Reuse the existing route and contract rather than introducing a new diagnostics endpoint. The backend already has the necessary `JobListFilter` fields and repository filtering, so the backend work should be a focused HTTP route regression test for the source-hash job kind/resource-class/source-id combination. The frontend work should expose the existing `source_id` filter and provide one quick-filter button that writes the exact source-hash `kind` and `resource_class` values into the route search params.

## Decision (ADR-lite)

Context: The previous slice added aggregate source fingerprint hash coverage and queue pressure to Admin overview, but operators still need a way to inspect the actual durable jobs behind those counts.

Decision: Use the existing Admin Jobs list as the drill-down path. Do not add a source-hash-specific route until the product needs richer per-source diagnostics beyond generic durable job fields.

Consequences: This keeps the slice small and contract-compatible. Operators get immediate visibility into job status, library, source, and timing, but they still do not see job payload details or retry controls.

## Out of Scope

- No operator enqueue, retry, cancellation, or bulk action.
- No duplicate-source candidate diagnostics or merge workflow.
- No new source hash runtime loop or scheduler behavior.
- No database schema or repository contract changes unless implementation disproves the current assumption that existing filters are sufficient.
- No new Admin DTO for raw source hash job payloads.
- No Public Client API exposure.

## Technical Notes

- Existing Admin Jobs route: `GET /admin/v1/jobs`.
- Existing route query parser: `crates/nako-server/src/http/query.rs::JobListQuery`.
- Existing route handler: `crates/nako-server/src/http/admin.rs::list_admin_jobs`.
- Existing response DTO: `AdminJobListItem` in `crates/nako-api/src/admin_contract.rs` generated output and `nako-api` Admin DTO source.
- Existing job kind string: `source_fingerprint_hash`.
- Existing source hash resource class: `disk.scan.source_fingerprint_hash`.
- Existing Admin Web Jobs route already owns `source_id` search state but does not render a source-id input.
- Existing Admin Web contract already includes `AdminJobsQuery.source_id` and `AdminJobListItem.source_id`.

## Definition of Done

- Rust formatting check passes for touched Rust code.
- Focused server Jobs route test passes.
- Cross-crate API/server check passes if Rust route/query code changes.
- Admin Web generated contract is left untouched unless the Rust contract changes.
- Admin Web check/test gate covering the Jobs route passes or any unavailable gate is reported.
