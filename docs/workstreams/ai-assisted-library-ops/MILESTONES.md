# AI Assisted Library Ops — Milestones

Status: Active
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

Exit criteria:

- Existing Automation Artifacts have stable proposal/readiness semantics for
  AI-assisted library operations.
- Targets, provenance, confidence/explanation, stale-target checks, and status
  transitions are explicit.
- Repository/app tests prove proposals do not mutate canonical metadata,
  sidecars, Media Sources, Managed Import artifacts, or library files.

Primary evidence:

- `crates/taru-core/src/automation.rs`
- `crates/taru-db`
- `crates/taru-automation`
- `crates/taru-server/src/app/automation.rs`

## M2 — Admin Proposal Diagnostics

Exit criteria:

- Admin-only routes expose bounded Generated Artifact proposal diagnostics.
- Admin TypeScript contract and typed client/mocks are synchronized.
- Public Client API and `taru-client-protocol` remain unchanged.
- Redaction tests cover prompts, raw generated payloads, provider secrets, raw
  Source Locators, local paths, and provider responses.

Primary evidence:

- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/http/admin.rs`
- `apps/admin-web/src/adminApi`
- HTTP/Admin tests

## M3 — Acceptance Planning Without Autonomous Writes

Exit criteria:

- Accept/reject planning is explicit, idempotent, and auditable.
- Accepted proposals route through existing metadata/NFO/apply authority
  boundaries rather than direct AI writes.
- Stale target and authority conflict checks block unsafe acceptance.
- Tests prove no autonomous write occurs before explicit acceptance.

Primary evidence:

- app/db acceptance tests
- redacted Admin audit diagnostics

## M4 — Closeout And Follow-On Split

Exit criteria:

- Final gates pass with fresh evidence.
- Workstream status and completed tasks are updated.
- Parent post-RPD umbrella re-scores Addon runtime/distribution, provider
  adapters, local model runtime, embeddings/vector search, protocol downloaders,
  and Public Client display.
- Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/ai-assisted-library-ops/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
