# Admin Intake Last-Run Evidence

## Goal

Expose redaction-safe recent execution evidence for Media Library Scan intake
components inside the existing Admin operator readiness drilldown. Operators
should be able to connect each intake action-plan component to the most recent
safe execution facts without opening raw durable job payloads, local paths, or
source locators.

## Requirements

- Extend `GET /admin/v1/operator-readiness` only. Do not add a new route or a
  mutation.
- Add a read-only recent-evidence block under
  `details.media_library_scan`.
- Cover the existing intake components in deterministic order:
  `library_scan`, `source_fingerprint_hash`, and `watch_folder`.
- For `library_scan` and `source_fingerprint_hash`, derive bounded recent job
  evidence from existing durable job rows and safe queue summaries.
- For `watch_folder`, derive recent runtime evidence from the existing
  redaction-safe latest tick diagnostics.
- Evidence may expose only component, status, reason, source reason code,
  bounded counts, safe job kind/status/resource class, timestamps, and safe
  booleans such as `has_error`.
- Evidence must not expose durable `input_json`, `summary_json`, raw `error`,
  retry linkage, attempt counts, Media Source locators, local paths, root paths,
  file names, fingerprints, content hashes, backend URLs, credentials, tokens,
  or provider payloads.
- Reuse existing readiness/action-plan semantics when choosing component status
  and reason. Do not introduce a second readiness policy.
- Regenerate generated Admin TypeScript contracts from `nako-api`.
- Add focused API/server tests for serialization, mapping, redaction, and route
  response shape.

## Acceptance Criteria

- [ ] `AdminOperatorReadinessMediaLibraryScanDetail` contains a
  `recent_evidence` block with three component entries.
- [ ] Recent evidence entries remain read-only and do not include executable
  route metadata.
- [ ] Library scan and source fingerprint hash entries can include a latest
  redaction-safe durable job summary when a matching job exists.
- [ ] Watch folder entry can include latest redaction-safe tick facts when a
  tick exists.
- [ ] Empty runtime/job state produces deterministic empty evidence shapes.
- [ ] Serialized API/server responses omit raw path, locator, token, durable
  payload, fingerprint/hash, backend URL, and raw error terms.
- [ ] Generated Admin Web contracts are refreshed from the Rust generator.
- [ ] Focused `nako-api` and `nako-server` checks pass.

## Definition of Done

- Rust DTO, server mapping, generated Admin contract, and tests are updated.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Focused nextest/API/server gates pass or any skipped gate is recorded with a
  concrete reason.
- Trellis task context validates.
- Commit includes only intended task files and code changes.

## Technical Approach

Use the existing operator-readiness context as the composition point:

- Add explicit Admin DTOs for media-library intake recent evidence in
  `nako_api::admin`.
- Query bounded latest durable jobs for `JobKind::LibraryScan` with
  `resource_class = "disk.scan"` and `JobKind::SourceFingerprintHash` with
  `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS`.
- Map jobs into a deliberately smaller evidence DTO rather than reusing
  `AdminJobListItem`, because job drilldown fields such as source bindings and
  diagnostics are more detailed than this operator-readiness summary needs.
- Build the three component entries with existing intake action-plan helpers so
  status, reason, source reason, and attention counts stay aligned.
- Reuse `AdminOverviewWatchFolderRuntimeSummary.diagnostics[*].last_tick` for
  watch-folder recent runtime evidence.

## Decision (ADR-lite)

Context: The operator-readiness route already aggregates safe scan, source-hash,
watch-folder, storage, jobs, playback, setup, network, and backup posture. The
new product gap is explainability: operators can see what to do next but not
the most recent safe execution evidence behind each intake component.

Decision: Extend the existing operator-readiness Media Library Scan detail with
a read-only `recent_evidence` block. Do not add a new route, schema migration,
job history endpoint, or execution command.

Consequences: This keeps the first slice small and contract-safe. The evidence
is enough for Admin Web to show "what happened recently" but not a full durable
job timeline. A later task can add a richer Admin Jobs/history page if product
needs it.

## Out of Scope

- New durable job persistence, migrations, or indexes.
- Raw job logs, job input, job summary, or error drilldown.
- Starting scans, retrying jobs, running watch-folder ticks, or repairing state
  from operator readiness.
- Admin Web rendering for this new block.
- Addon lifecycle or addon task execution evidence.

## Technical Notes

- Project glossary: `CONTEXT.md`.
- Relevant contract/spec authority:
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`.
- Existing operator-readiness implementation:
  `crates/nako-server/src/http/admin.rs`.
- Existing Admin DTOs:
  `crates/nako-api/src/admin.rs` and
  `crates/nako-api/src/admin/operations.rs`.
- Existing durable job listing:
  `nako_core::JobRepository::list_jobs` via
  `nako-server::app::jobs::JobAppService::list_jobs`.
