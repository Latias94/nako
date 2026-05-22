# AI Assisted Library Ops — Milestones

Status: Complete
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Status: completed on 2026-05-22.

Exit criteria:

- [x] Workstream docs exist and agree.
- [x] Scope is Generated Artifact proposals and acceptance first.
- [x] Local model runtime, vector DB, provider-specific adapters, Addon
  distribution, downloader protocols, autonomous writes, and Public Client API
  changes are out of scope.
- [x] Parent `post-rpd-product-hardening` points at this lane.

Primary evidence:

- `docs/workstreams/ai-assisted-library-ops/DESIGN.md`
- `docs/workstreams/ai-assisted-library-ops/TODO.md`

## M1 — Generated Artifact Proposal Queue

Status: completed on 2026-05-22.

Exit criteria:

- [x] Existing Automation Artifacts have stable proposal/readiness semantics for
  AI-assisted library operations.
- [x] Targets, provenance, confidence/explanation, stale-target checks, and status
  transitions are explicit.
- [x] Repository/app tests prove proposals do not mutate canonical metadata,
  sidecars, Media Sources, Managed Import artifacts, or library files.

Primary evidence:

- `crates/nako-core/src/automation.rs`
- `crates/nako-db`
- `crates/nako-automation`
- `crates/nako-server/src/app/automation.rs`

## M2 — Admin Proposal Diagnostics

Status: completed on 2026-05-22.

Exit criteria:

- [x] Admin-only routes expose bounded Generated Artifact proposal diagnostics.
- [x] Admin TypeScript contract and typed client/mocks are synchronized.
- [x] Public Client API and `nako-client-protocol` remain unchanged.
- [x] Redaction tests cover prompts, raw generated payloads, provider secrets, raw
  Source Locators, local paths, and provider responses.

Primary evidence:

- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/http/admin.rs`
- `apps/admin-web/src/adminApi`
- `crates/nako-server/src/http/tests/system.rs`

## M3 — Acceptance Planning Without Autonomous Writes

Status: completed on 2026-05-22.

Exit criteria:

- [x] Accept/reject planning is explicit, idempotent, and auditable.
- [x] Accepted proposals route through existing metadata/NFO/apply authority
  boundaries rather than direct AI writes.
- [x] Stale target and authority conflict checks block unsafe acceptance.
- [x] Tests prove no autonomous write occurs before explicit acceptance.

Primary evidence:

- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/tests/automation.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`

## M4 — Closeout And Follow-On Split

Status: completed on 2026-05-22.

Exit criteria:

- [x] Final gates pass with fresh evidence.
- [x] Workstream status and completed tasks are updated.
- [x] Parent post-RPD umbrella re-scores Addon runtime/distribution, provider
  adapters, local model runtime, embeddings/vector search, protocol downloaders,
  and Public Client display.
- [x] Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/ai-assisted-library-ops/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
