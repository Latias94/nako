# Admin Intake Evidence Summary

## Goal

Make the Admin operator-readiness Media Library Scan drilldown more actionable by exposing one redaction-safe intake evidence summary that ties together configured library scan posture, Source Fingerprint Hash backlog, and Watch Folder runtime evidence.

## Requirements

- Add an `intake_evidence` object to `AdminOperatorReadinessMediaLibraryScanDetail`.
- Derive the summary only from existing safe aggregate/read-model fields already used by operator readiness:
  - library scan posture counts,
  - source fingerprint hash coverage and queue summary,
  - watch-folder runtime diagnostics and latest tick summary,
  - existing media library scan readiness check.
- The summary must include the selected check status, reason, source reason, total attention count, and component attention counts for library scans, source fingerprint hashing, and watch folders.
- Preserve the existing Media Library Scan readiness priority order; this task adds an explanatory summary, not a new decision engine.
- Update Admin API contract generation and generated Admin Web contract artifacts from `nako-api`.
- Add focused API/server tests that prove the new field is present and remains redaction-safe.

## Acceptance Criteria

- [ ] `GET /admin/v1/operator-readiness` returns `details.media_library_scan.intake_evidence`.
- [ ] `intake_evidence.status`, `reason`, `source_reason`, and `attention_count` match the already selected Media Library Scan readiness check.
- [ ] `library_scan_attention_count` counts failed, pending, and never-completed configured libraries.
- [ ] `source_fingerprint_hash_attention_count` counts queued, running, delayed-retry, and failed Source Fingerprint Hash work.
- [ ] `watch_folder_attention_count` counts unsupported/missing watch-folder coverage and latest tick pressure statuses.
- [ ] The response does not expose raw paths, Source Locators, storage URIs, job input JSON, job summary JSON, raw errors, tokens, fingerprints, hashes, credentials, backend URLs, or file names.
- [ ] `nako-api` admin contract tests pass and generated Admin Web TypeScript contract output is refreshed.
- [ ] Focused `nako-server` operator-readiness HTTP tests pass.

## Definition Of Done

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_operator_readiness_returns_safe_drilldown_read_model --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-17-06-17-admin-intake-evidence-summary`
- `git diff --check`

## Technical Approach

Add the new DTO in `crates/nako-api/src/admin.rs`, extend the Admin contract generator output, regenerate `apps/admin-web/src/adminApi/generated/contract.ts`, and compose the value in `crates/nako-server/src/http/admin.rs` near the existing `AdminOperatorReadinessMediaLibraryScanDetail` mapping.

The server-side helper should be pure and should reuse existing readiness inputs rather than re-querying storage. This keeps the route read-only and ensures the field is only a summary of already redacted facts.

## Decision (ADR-lite)

**Context**: Operators can currently inspect library scan posture, source hash queue pressure, and watch-folder diagnostics separately. The route lacks a stable single object that says which intake evidence is currently demanding attention.

**Decision**: Add one derived `intake_evidence` summary under the existing Media Library Scan detail and reuse the already selected readiness check for top-level status/reason.

**Consequences**: Clients can render or automate against one small object without duplicating readiness logic. The summary remains conservative because it only counts safe component pressure and does not introduce new mutation, scheduling, or raw diagnostic surfaces.

## Out Of Scope

- No new Admin route.
- No Admin Web UI rendering changes beyond generated contract/mocks required for type safety.
- No scan enqueue, repair, scheduler execution, watch-folder runtime change, or source hash execution change.
- No schema migration.
- No Public Client, OpenAPI public route, or generated Public SDK change.

## Technical Notes

- Applicable specs:
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- Generated Admin Web contract command:
  `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
- Existing dirty files outside this task must not be staged or reverted:
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/sdk.rs`
  - `crates/nako-client-protocol/src/catalog.rs`
  - `crates/nako-reference-addon/src/lib.rs`
