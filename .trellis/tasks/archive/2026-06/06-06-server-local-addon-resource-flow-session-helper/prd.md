# Server-Local Addon Resource Flow Session Helper

## Goal

Refactor duplicated transient selection session mechanics in Addon Resource
Search and Addon Subtitle Search into a server-local helper. Preserve existing
Admin/API DTOs, route paths, response shapes, redaction behavior, and
resource-specific planning logic.

## Requirements

- Add a server-local helper module under `crates/nako-server/src/app/addons/`
  for host-owned Addon selection sessions.
- Keep the helper inside `nako-server`; do not move session policy into
  `nako-addon-protocol`.
- Preserve the current 15 minute TTL and 64 session max-count behavior.
- Preserve lookup validation for `(addon_id, manifest_id, search_id,
  selection_id)`.
- Refactor Resource Search and Subtitle Search to use the shared helper.
- Keep resource-specific selection payloads, subtitle import planning, link
  check context, acquisition intake handoff, and redaction helpers in their
  current resource modules.
- Keep public/Admin wire contracts unchanged; no generated TypeScript contract
  changes are expected.

## Acceptance Criteria

- [ ] Resource Search and Subtitle Search no longer each own bespoke prune /
      max-count / lookup session-store logic.
- [ ] A shared `SelectionSessionStore` or equivalent helper owns TTL pruning,
      oldest-session eviction, and addon/manifest/selection validation.
- [ ] Resource Search observable behavior remains unchanged for search,
      selection, link check, and redaction.
- [ ] Subtitle Search observable behavior remains unchanged for selected
      references, import plan/apply, and redaction.
- [ ] Existing tests for Addon resource search and subtitle import pass.
- [ ] No `nako-api`, generated Admin contract, `nako-addon-protocol`, or Admin
      Web changes are introduced.

## Definition Of Done

- `cargo check -p nako-server --tests` passes.
- Focused `cargo nextest` gates for resource search and subtitle flows pass.
- `cargo fmt --all -- --check`, `git diff --check`, and Trellis task
  validation pass.
- The task is archived with implementation/check context.
- If verified, commit the bounded change with a Conventional Commit message.

## Technical Approach

- Add `app/addons/resource_flow.rs` with a generic transient session helper.
- Model common metadata once: `search_id`, `addon_id`, `manifest_id`,
  `created_at_ms`, `expires_at_ms`, and typed selections keyed by
  `selection_id`.
- Keep returned handoff values resource-specific by letting callers define the
  typed selection payload.
- Replace `ResourceSearchSessionStore` and `SubtitleSearchSessionStore`
  internals with the shared helper while preserving their public module-local
  API if that keeps call-site churn low.
- Prefer small wrapper methods in each resource module so missing-selection
  errors and response construction remain locally readable.

## Decision (ADR-lite)

**Context**: The Addon boundary audit found duplicated TTL, max-count,
selection lookup, and addon/manifest validation mechanics in Resource Search
and Subtitle Search. The duplication is server host policy, not Addon Protocol
wire contract.

**Decision**: Introduce a server-local generic session helper and migrate the
two existing flows to it without changing public contracts.

**Consequences**: Future Addon resource flows get one reusable host-owned
session pattern. The helper must stay narrow: it owns session mechanics only,
not subtitle planning, acquisition intake, link checks, or protocol DTOs.

## Out Of Scope

- No Admin route, DTO, generated TypeScript contract, or Admin Web changes.
- No Addon Protocol changes.
- No durable selected-reference persistence.
- No Addon Manager product surface changes.
- No external acquisition materialization refactor.
- No Addon task/event execution policy convergence.

## Technical Notes

- Parent audit:
  `.trellis/tasks/06-05-addon-resource-flow-pattern-audit/research/host-owned-addon-resource-flow-pattern.md`.
- Relevant spec:
  `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`.
- Current duplicated files:
  `crates/nako-server/src/app/addons/resource_search.rs` and
  `crates/nako-server/src/app/addons/subtitles.rs`.
- The helper must preserve host-owned selected references: Admin callers keep
  opaque `search_id` / `selection_id` values and never resubmit raw URLs,
  local paths, tokens, materialization refs, or provider payloads.
