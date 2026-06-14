# M2 Source Fingerprint Job Diagnostics Drilldown

## Goal

Add a focused Admin Jobs diagnostic drilldown for `SourceFingerprintHash`
durable jobs so operators can understand pending, succeeded, and failed source
hash work without exposing raw storage or fingerprint material. This advances
M2 large-library reliability by making a concrete long-running job class more
diagnosable after retries, restarts, and queue pressure.

## What I Already Know

- Source duplicate reconciliation backend and Admin Web operator flow are
  already shipped and should not be reopened here.
- VFS cache repair already has a typed `AdminJobDiagnostics` branch.
- Source fingerprint hash has durable enqueue, retry, scheduler execution,
  redaction-safe summary persistence, Admin overview counters, and queue
  pressure visibility.
- `AdminJobDiagnostics` currently expands only `JobKind::VfsCacheRepair`;
  `JobKind::SourceFingerprintHash` gets generic job row facts but no typed
  drilldown.
- `SourceFingerprintHashJobSummary` already carries safe facts: mode, evidence
  kind, confidence, stale state, and bytes hashed.
- Redaction boundaries are strict: Admin Jobs must not expose raw job input,
  Source Locators, local paths, etags, backend URLs, credentials, raw hashes,
  raw fingerprints, or raw error blobs.

## Research References

- `research/source-fingerprint-job-diagnostics-gap.md` - local code and
  architecture gap analysis for this slice.
- `docs/architecture/CONTROL_PLANE.md` - durable job diagnostics and ADR 0053
  control-plane boundary.
- `docs/architecture/STORAGE_VFS.md` - source fingerprint shipped state and
  follow-ons.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` - evidence-driven
  Admin Jobs drilldown routing rule.
- `.trellis/tasks/archive/2026-06/06-14-06-14-m2-large-library-reliability-plan/prd.md`
  - M2 first-slice planning and alternatives.

## Requirements

- Extend `AdminJobDiagnostics` with an optional source-fingerprint-hash branch
  for `JobKind::SourceFingerprintHash`.
- Add typed Admin DTOs for source fingerprint job diagnostics using existing
  source-hash summary semantics.
- Classify diagnostic status as pending, summary available, or failed, matching
  the existing VFS cache repair diagnostic style where practical.
- Parse `SourceFingerprintHashJobSummary` from persisted summary JSON and
  expose only safe facts: hash mode, evidence kind, confidence, stale state,
  and bytes hashed.
- For failed jobs, expose a redacted failure diagnostic that identifies the job
  status and a safe generic message/retryability signal without raw error text.
- Keep existing Admin job list/detail row facts and route shapes stable; use
  the existing `diagnostics` field rather than adding a new route.
- Add focused tests proving source fingerprint diagnostics appear for matching
  jobs and remain absent for unrelated job kinds.
- Add redaction assertions that serialized Admin job responses do not contain
  durable input JSON, raw locator/path-like values, raw hashes, raw
  fingerprints, etags, credentials, or raw error text.

## Acceptance Criteria

- [ ] `GET /admin/v1/jobs` responses include
  `diagnostics.source_fingerprint_hash` for `SourceFingerprintHash` jobs.
- [ ] Succeeded source hash jobs with valid summary JSON expose typed safe
  summary facts.
- [ ] Queued/running source hash jobs without summary or error expose a pending
  diagnostic.
- [ ] Failed source hash jobs expose redacted failure diagnostics without raw
  error text.
- [ ] Existing VFS cache repair diagnostics remain unchanged.
- [ ] API contract tests cover source fingerprint diagnostic serialization.
- [ ] Server Admin Jobs tests cover the source fingerprint diagnostic and
  redaction boundary.

## Definition of Done

- Focused Rust tests pass for `nako-api` and `nako-server`.
- `cargo fmt --all` or a narrower justified format gate is run.
- Generated Admin contract handling is addressed if DTO changes require it.
- Architecture/spec docs are updated only if the implementation establishes a
  new durable pattern beyond this existing diagnostic branch.

## Technical Approach

Use the existing `AdminJobDiagnostics` optional-branch pattern. Add a
`source_fingerprint_hash` field alongside `vfs_cache_repair`, with a
source-hash-specific diagnostic DTO. Reuse `SourceFingerprintHashJobSummary`
as the parsing authority or mirror it with an Admin DTO if generated contract
stability requires API-owned types. Do not expose raw job input; the job row
already carries safe library/source bindings.

The initial implementation should stay in:

- `crates/nako-api/src/admin/operations.rs` for DTOs, summary parsing, and
  contract tests;
- `crates/nako-server/src/http/admin.rs` and focused Admin HTTP tests only if
  route-level serialization or redaction assertions need coverage;
- generated Admin Web contract artifacts only if the contract generator emits
  the new DTOs.

## Decision (ADR-lite)

Context: M2 needs deeper durable job operability, but source duplicate,
watch-folder reliability, and VFS repair automation have already moved forward.
The remaining durable-job drilldown gap should be tied to one concrete job
kind.

Decision: add a source-fingerprint-hash branch to existing Admin Jobs
diagnostics, using job-kind-specific DTOs and safe summary parsing.

Consequences: operators get better source hash job triage without a new job
platform. Future job kinds can follow the same optional-branch pattern only
when they have a safe summary contract and focused evidence.

## Out of Scope

- New Admin routes, Public Client routes, or schema migrations.
- Generic job detail platform, generic retry UI, or broad scheduler migration.
- Automatic source duplicate reconciliation, source merge, Media Item merge, or
  undo/confirm/reject flows.
- VFS cache repair behavior changes.
- Exposing durable input JSON, raw summary JSON, raw errors, Source Locators,
  local paths, backend URLs, etags, raw hashes, raw fingerprints, credentials,
  tokens, or provider payloads.

## Technical Notes

- Relevant code:
  - `crates/nako-api/src/admin/operations.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/app/source_hash.rs`
  - `crates/nako-library/src/source_hash.rs`
- Relevant specs:
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
- Likely verification:
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - focused `nako-api` operations serialization tests
  - focused `nako-server` Admin Jobs route tests
  - `cargo check -p nako-api -p nako-server --tests`

## Open Questions

- None for MVP. The implementation can proceed with the optional diagnostics
  branch approach.
