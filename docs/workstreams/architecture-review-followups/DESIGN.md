# Architecture Review Follow-Ups

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

The 2026-05-18 architecture review found several cross-cutting design risks in
Taru's current implementation. Most of them touch more than one crate, ADR, or
existing workstream. Leaving those findings only in the conversation would make
them hard to prioritize, revisit, or assign to future implementation lanes.

This lane exists to keep the findings visible and to route each one into the
right workstream shape before code changes begin.

## Relevant Authority

- ADRs:
  - `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Domain glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/server-architecture-hardening/`
  - `docs/workstreams/repository-seam-deepening/`
  - `docs/workstreams/metadata-refresh-seam/`
  - `docs/workstreams/catalog-hydration-lookup-deepening/`
  - `docs/workstreams/nfo-round-trip-preservation/`
  - `docs/workstreams/playback-source-selection-deepening/`
  - `docs/workstreams/transcode-runtime/`
  - `docs/workstreams/addons-automation/`
  - `docs/workstreams/public-api-contract/`

## Problem

Taru already has strong domain language and many completed architecture
workstreams, but several deeper consistency and contract questions remain:

- some workflow modules still require callers to understand broad persistence
  and configuration details;
- some multi-record updates are expressed as ordered app-level calls rather
  than one explicit commit interface;
- local metadata and provider metadata authority rules can drift across NFO and
  metadata refresh paths;
- Public Client DTOs may expose Source Locator details that are convenient for
  local debugging but risky for remote clients;
- Addon, playback, and hardware-acceleration seams are good foundations but
  still need follow-up before production-grade extensibility and streaming.

The risk is not that every item must be fixed immediately. The risk is losing
the dependency order and rediscovering the same findings repeatedly.

## Target State

- Every finding from the review has an owner lane and status.
- The first execution lane is chosen with clear rationale.
- Findings that need ADR changes are identified before implementation.
- Findings that fit an existing workstream are routed there instead of creating
  duplicate lanes.
- This lane remains a lightweight coordination record rather than a dumping
  ground for implementation details.

## In Scope

- Track architecture review findings and evidence anchors.
- Decide whether each finding should reuse an existing workstream or split into
  a new focused workstream.
- Record recommended execution order.
- Record non-goals so implementation lanes do not become broad rewrites.
- Update this lane when a finding is superseded, split, deferred, or closed.

## Out Of Scope

- Direct code changes.
- Schema migrations.
- Public Client API changes.
- Addon token issuance implementation.
- Transcode Profile implementation.
- Broad crate reshuffling.
- Copying behavior or source from `repo-ref/jellyfin`.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Metadata refresh atomicity is the highest-risk follow-up. | High | `crates/taru-metadata/src/strategy.rs`, `crates/taru-catalog/src/lib.rs`, `crates/taru-db/src/search.rs` | Reprioritize if another active feature will hit Public Client or Addon contracts first. |
| NFO/provider merge unification should be separate from NFO XML preservation. | High | `docs/workstreams/nfo-round-trip-preservation/WORKSTREAM.json` lists merge-policy redesign as a non-goal. | Reopen NFO lane only if the change is primarily XML preservation, not metadata authority. |
| Public Client Source Locator redaction may require contract discussion before code. | Medium | ADR 0023 and ADR 0027 define leakage constraints, but current DTOs include `locator`. | Create an ADR or public API contract follow-up before changing wire shape. |
| Addon side effects should reuse the existing addon/automation lane if possible. | Medium | ADR 0020 and `addons-automation` already own the Addon trust model. | Split a new lane only if token/grant/effect intake grows beyond the existing lane. |

## Architecture Direction

Use this lane as a routing module. Its interface is the finding ledger and
status map; its implementation is the set of linked design notes, task ledgers,
and evidence gates in focused execution lanes.

The preferred direction is to deepen modules where an interface can hide real
workflow complexity:

- metadata refresh should hide commit ordering and consistency behavior;
- metadata merge policy should hide field-by-field authority rules;
- Public Client protocol should hide storage implementation details;
- Addon effect intake should hide token, grant, audit, and execution policy;
- playback session identity should hide request profile normalization.

Do not create a generic abstraction when only one adapter exists and no
behavior varies. Prefer workstream-specific seams with concrete tests.

## Finding Routing Table

| ID | Finding | Recommended lane | Status | Notes |
| --- | --- | --- | --- | --- |
| ARF-001 | Metadata refresh, catalog graph, and search projection atomicity | New `metadata-catalog-commit-atomicity` | Proposed | First recommended execution lane. |
| ARF-002 | NFO and provider merge policy duplication | New `metadata-merge-policy-unification` | Proposed | Keep separate from NFO XML preservation. |
| ARF-003 | Broad server app persistence/config interfaces | Reuse or reopen server architecture follow-up | Proposed | Only create a new lane for a concrete workflow. |
| ARF-004 | Media Library config/database source of truth | Expand `multi-library-hardening` into full workstream | Proposed | Current directory has only an old phase note. |
| ARF-005 | Public Client Source Locator leakage | New or reuse `public-api-contract` follow-up | Proposed | Likely needs contract/design discussion first. |
| ARF-006 | Addon token/grant/side-effect seams | Reuse `addons-automation` if active enough, otherwise split | Proposed | Align with ADR 0020. |
| ARF-007 | HLS request identity and Transcode Profile | Reuse `transcode-runtime` or split playback profile lane | Proposed | Avoid changing cache semantics without profile key tests. |
| ARF-008 | Hardware encode viability diagnostics | Reuse `transcode-runtime` follow-up | Deferred | Lower priority until users need operator diagnostics. |
| ARF-009 | Search adapter depth and CJK/pinyin/romaji support | Future search adapter lane | Deferred | Existing ADR accepts basic SQLite fallback. |

## Recommended Execution Order

1. Open `metadata-catalog-commit-atomicity`.
2. Open `metadata-merge-policy-unification`.
3. Promote `multi-library-hardening` into a full workstream.
4. Design Public Client Source Locator redaction.
5. Continue Addon side-effect and token/grant design.
6. Continue playback profile and hardware diagnostics work.

## Closeout Condition

This lane can close when:

- each finding is assigned to a focused lane, explicitly deferred, or closed;
- the first two execution lanes have been opened or deliberately rejected;
- ADR updates needed before implementation are identified;
- `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `WORKSTREAM.json`
  reflect the final routing state.
