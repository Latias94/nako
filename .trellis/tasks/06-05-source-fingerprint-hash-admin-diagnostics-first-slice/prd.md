# Source fingerprint hash admin diagnostics first slice

## Goal

Expose a read-only, redaction-safe source fingerprint hash diagnostic summary through the existing Admin overview response so operators can see whether source fingerprint hashing is making progress without opening a new route or leaking source material.

## What I Already Know

- The user asked to continue the source fingerprint hash work after scheduler integration and evidence persistence.
- The agreed first slice is Admin/operator diagnostics, not Public API exposure, duplicate merge behavior, mutations, or a new runtime loop.
- `GET /admin/v1/overview` already aggregates safe Admin diagnostics and is consumed by Admin Web.
- Source fingerprint hash execution already persists a redacted `MediaSource.fingerprint` / `SourceState.fingerprint` and keeps job summaries free of raw fingerprint/hash/locator/content material.
- `summarize_job_queue_pressure()` already provides bounded aggregate job queue pressure by `JobKind`, `JobStatus`, and `resource_class`.
- Existing media repositories do not currently expose an accurate aggregate source fingerprint coverage summary; the implementation should add a repository aggregate instead of scanning all sources in the overview handler.

## Requirements

- Add a nested source fingerprint hash diagnostics block to `AdminOverviewResponse`.
- Keep the block read-only and available only through the existing Admin overview route.
- Include useful aggregate counters:
  - total media sources known to the repository;
  - media sources with any persisted fingerprint;
  - media sources with source hash content evidence fingerprints;
  - queued/running/succeeded/failed/cancelled source fingerprint hash jobs;
  - claimable and delayed retry queued source fingerprint hash jobs;
  - oldest queued and next retry timestamps when available.
- Compute source coverage through a repository-level aggregate query for SQLite/PostgreSQL, not by unbounded HTTP-layer pagination.
- Derive job counters from existing queue pressure summaries filtered to `JobKind::SourceFingerprintHash` and `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS`.
- Do not expose raw source locator/path, raw fingerprint value, raw content hash, raw storage URI, or raw job input/summary/error body.
- Regenerate Admin Web TypeScript contracts from `nako-api`; do not hand-edit generated contract artifacts.
- Update Admin Web mock data and the overview page only as much as needed to surface the new aggregate block.

## Acceptance Criteria

- [ ] `GET /admin/v1/overview` returns `source_fingerprint_hash` with snake_case aggregate fields.
- [ ] Empty repositories report zero source coverage and zero source-hash job counters.
- [ ] Fixtures containing sensitive locators, raw fingerprints, and content strings do not leak those values in the overview JSON body.
- [ ] Admin contract tests include the new DTO shape and generated Admin Web contracts match the generator.
- [ ] Admin Web overview can render the new summary from mock/live data without adding a new page or route.

## Out of Scope

- No Public Client API exposure.
- No operator mutation, retry, cancellation, or enqueue control.
- No duplicate-source reconciliation or merge workflow.
- No schema migration unless an aggregate query requires none; this slice should reuse existing persisted columns.
- No raw fingerprint/hash/path/locator display, even on Admin surfaces.

## Technical Notes

- Primary backend DTO: `crates/nako-api/src/admin.rs`.
- Admin contract generator: `crates/nako-api/src/admin_contract.rs`.
- Existing route handler: `crates/nako-server/src/http/admin.rs::get_admin_overview`.
- Existing overview route test: `crates/nako-server/src/http/tests/system.rs::admin_v1_overview_composes_safe_read_only_diagnostics`.
- Source hash app service: `crates/nako-server/src/app/source_hash.rs`.
- Existing source hash focused tests: `crates/nako-server/src/app/tests/source_hash.rs`.
- Existing queue pressure aggregate: `nako_core::JobRepository::summarize_job_queue_pressure`.
- Relevant specs: `nako-api` Admin diagnostic DTO rules, `nako-server` HTTP/quality rules, Admin Web generated-contract rules, and the cross-layer guide.

## Definition of Done

- Rust formatting check passes.
- Focused `nako-api` Admin contract tests pass.
- Cross-crate API/server check passes.
- Focused server overview/source fingerprint diagnostics tests pass.
- Admin Web generated contract is refreshed and relevant TypeScript checks/tests are run or explicitly reported if unavailable.
