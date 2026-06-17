# Admin intake action plan

## Goal

Add a redaction-safe Media Library Scan intake action plan to Admin operator
readiness so self-hosted operators can see which backend subsystem needs
attention after a scan posture warning. The response must explain what area is
blocked or degraded without exposing raw library roots, source locators, hashes,
job payloads, backend URLs, or errors.

## What I Already Know

- `GET /admin/v1/operator-readiness` already returns a safe drilldown for setup,
  media library scan, playback, durable jobs, storage, network, and backup.
- Media Library Scan drilldown already includes configured library posture,
  library scan posture, source fingerprint hash overview, watch-folder runtime,
  and an aggregate intake evidence summary.
- Admin Web contract artifacts are generated from `crates/nako-api`; generated
  TypeScript files are not source of truth.
- This task crosses `nako-api`, `nako-server`, and Admin Web contract/mock data.

## Requirements

- Extend the Admin-only operator readiness DTO with a Media Library Scan intake
  action plan.
- The plan must be read-only and must not enqueue scans, repair work, execute
  watch-folder reconciliation, or mutate scheduler state.
- The plan must expose stable component identities for:
  - library scan evidence
  - source fingerprint hash evidence
  - watch-folder runtime evidence
- Each component entry must include a readiness status, readiness reason,
  optional safe source reason code, attention count, and optional existing Admin
  action target.
- The action plan must be derived only from existing safe summaries and queue
  pressure facts already used by operator readiness.
- Update generated Admin TypeScript contracts for both Admin Web locations.
- Update deterministic Admin Web mock data so generated contract users continue
  to type-check.
- Update Trellis API contract spec so the current drilldown shape documents
  intake evidence and action plan fields.

## Acceptance Criteria

- [ ] `AdminOperatorReadinessMediaLibraryScanDetail` includes the new intake
  action plan field.
- [ ] Generated Admin TypeScript contracts include the new plan and component
  DTOs.
- [ ] Server route tests prove the plan is present, deterministic, read-only,
  redaction-safe, and uses existing Admin actions.
- [ ] Pure helper coverage proves component counts and priority reasons are
  stable for library scan, source hash, and watch-folder evidence.
- [ ] Admin Web check passes after regenerated contracts and mock data update.
- [ ] No public client, OpenAPI, SDK, or mutation route receives the Admin-only
  readiness details.

## Definition of Done

- Rust formatting passes.
- Focused `nako-api` contract tests pass.
- Focused `nako-server` operator readiness tests pass.
- Admin Web check passes if generated contracts or mock data change.
- Trellis task validates, is archived after completion, and the implementation
  is committed with a Conventional Commit message.

## Technical Approach

Add the contract at the API DTO layer, then implement server composition from
the already-safe readiness facts:

- `library_scan` entry summarizes failed, pending, and never-completed scan
  pressure.
- `source_fingerprint_hash` entry summarizes queued, running, delayed-retry, and
  failed source hash work.
- `watch_folder` entry reuses existing watch-folder tick-pressure and coverage
  priority, keeping tick pressure ahead of coverage gaps.
- Entries should point to the existing Admin action that best helps the operator
  inspect the issue, not create a new mutation.

## Decision (ADR-lite)

Context: The previous intake evidence summary answers "how much evidence needs
attention" but not "which subsystem should I inspect first." A self-hosted
operator needs a bounded next-step view without granting the readiness endpoint
execution authority.

Decision: Add a small, explicit action-plan DTO under Media Library Scan
drilldown. Keep component identities stable, reuse readiness status/reason/action
vocabularies, and derive the plan from existing safe facts.

Consequences: The Admin Web contract grows, but readiness remains read-only and
safe to cache as a diagnostic response. Future UI work can render this plan
without reverse-engineering counts from unrelated sub-summaries.

## Out of Scope

- Starting scans, repairs, transcodes, or watch-folder reconciliation from the
  readiness endpoint.
- Adding a new Admin Web visual section beyond deterministic mock/contract
  compatibility.
- Changing durable job persistence, watch-folder scheduler behavior, or source
  fingerprint hash retry policy.
- Adding addon lifecycle readiness.

## Technical Notes

- Authority: ADR 0027 and ADR 0053.
- Relevant specs:
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Existing unrelated dirty files must not be staged or reverted:
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-api/src/sdk.rs`
  - `crates/nako-client-protocol/src/catalog.rs`
  - `crates/nako-reference-addon/src/lib.rs`
