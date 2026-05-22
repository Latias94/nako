# Architecture Review Follow-Ups

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

The 2026-05-18 architecture review found several cross-cutting design risks in
Nako's current implementation. Most of them touch more than one crate, ADR, or
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

Nako already has strong domain language and many completed architecture
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
| Metadata refresh atomicity is the highest-risk follow-up. | High | `crates/nako-metadata/src/strategy.rs`, `crates/nako-catalog/src/lib.rs`, `crates/nako-db/src/search.rs` | Reprioritize if another active feature will hit Public Client or Addon contracts first. |
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
| ARF-001 | Metadata refresh, catalog graph, and search projection atomicity | `metadata-catalog-commit-atomicity` | Closed | First execution lane was opened and closed. Catalog graph/search commits and metadata refresh persistence commits now have SQLite transaction tests. |
| ARF-002 | NFO and provider merge policy duplication | `metadata-merge-policy-unification` | Closed | Execution lane completed. Shared `MetadataMergePolicy` now lives in `nako-core`; provider refresh, hierarchy confirmation, and NFO import use the shared boundary. |
| ARF-003 | Broad server app persistence/config interfaces | `server-architecture-hardening` plus `repository-seam-deepening` | Assigned | Reuse existing lanes for concrete workflows only; do not open a generic app-service rewrite lane. |
| ARF-004 | Media Library config/database source of truth | `multi-library-hardening` | Closed | Execution lane completed. Startup reconciliation now establishes persisted Library authority, workflows use reconciled libraries, and configured backend roots are validated at startup. |
| ARF-005 | Public Client Source Locator leakage | `public-client-source-locator-redaction` | Closed | Execution lane completed. Public Client DTOs, OpenAPI schema, generated SDK output, route tests, and HTTP API docs now redact raw Source Locators. |
| ARF-006 | Addon token/grant/side-effect seams | `addons-automation` | Assigned | Routed to Post-M5 follow-up for Addon Token issuance, rotation, library-scoped grants, and Nako-mediated Addon Side Effect intake. |
| ARF-007 | HLS request identity and Transcode Profile | `transcode-runtime` plus `playback-source-selection-deepening` | Assigned | Routed to Post-M25/Post-M43 follow-ups. Avoid changing HLS reuse/cache semantics without request/profile key tests. |
| ARF-008 | Hardware encode viability diagnostics | Reuse `transcode-runtime` follow-up | Deferred | Lower priority until users need operator diagnostics. |
| ARF-009 | Search adapter depth and CJK/pinyin/romaji support | Future search adapter lane | Deferred | Existing ADR accepts basic SQLite fallback. |

## Recommended Execution Order

1. Closed: `metadata-catalog-commit-atomicity`.
2. Closed: `metadata-merge-policy-unification`.
3. Closed: `multi-library-hardening`.
4. Closed: `public-client-source-locator-redaction`.
5. Next execution lane: continue Addon side-effect and token/grant design through the Post-M5
   follow-up.
6. Continue playback profile work through the Post-M25/Post-M43 follow-ups;
   defer hardware diagnostics until operator
   needs make it urgent.
7. Defer deeper search adapter work until search requirements exceed SQLite.

## Closeout Condition

This lane can close when:

- each finding is assigned to a focused lane, explicitly deferred, or closed;
- the first two execution lanes have been opened or deliberately rejected;
- ADR updates needed before implementation are identified;
- `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `WORKSTREAM.json`
  reflect the final routing state.

## Closeout Summary

ARF closes after ARF-060. The first two execution lanes,
`metadata-catalog-commit-atomicity` and `metadata-merge-policy-unification`,
were opened and completed. `multi-library-hardening` and
`public-client-source-locator-redaction` have also completed after this routing
lane closed. Remaining assigned findings have target lanes:
`addons-automation`, `transcode-runtime`, and
`playback-source-selection-deepening`. Hardware encode viability diagnostics
and deeper search adapter work remain deferred.
