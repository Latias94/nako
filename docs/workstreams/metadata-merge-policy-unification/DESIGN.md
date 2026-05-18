# Metadata Merge Policy Unification

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

ADR 0007 defines Metadata Source Priority and local authority, while ADR 0008
treats NFO Import as a Local Metadata boundary. The current implementation has
two separate merge implementations:

- `crates/taru-metadata/src/merge.rs` handles provider and hierarchy
  confirmation merges through `MetadataMergePolicy`.
- `crates/taru-nfo/src/import.rs` has a private NFO-specific field merge,
  populated-field detection, and lock filtering path.

Both paths iterate over the same `CanonicalMetadata` fields. They differ in
source semantics, but they should not duplicate field-by-field mechanics. Every
new Canonical Metadata field currently risks being added to one path and missed
by the other.

## Relevant Authority

- ADRs:
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
- Domain glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/architecture-review-followups/`
  - `docs/workstreams/metadata-refresh-seam/`
  - `docs/workstreams/metadata-catalog-commit-atomicity/`
  - `docs/workstreams/nfo-round-trip-preservation/`
  - `docs/workstreams/catalog-hydration-lookup-deepening/`

## Problem

Taru currently has duplicated metadata authority logic:

- provider refresh applies `MetadataRefreshMode` and field locks through
  `MetadataMergePolicy`;
- hierarchy confirmation filters locks from other sources before using the
  same policy;
- NFO import translates `LocalMetadataPolicy` into a separate missing-only
  boolean and protects fields locked by non-NFO sources;
- NFO import separately determines which fields were populated so it can write
  NFO locks for local-authority policies.

This is not just code duplication. It makes the domain model harder to trust.
Metadata Source Priority, NFO local authority, and provider refresh behavior
should be explainable from one policy boundary.

## Target State

- There is one shared merge-policy boundary for Canonical Metadata field
  replacement decisions.
- Provider refresh, hierarchy confirmation, and NFO import all use that boundary
  instead of maintaining separate per-field replacement loops.
- Source-aware lock handling is explicit: a source can protect fields written
  by other sources without accidentally blocking its own local-authority update.
- LocalMetadataPolicy and MetadataRefreshMode are translated into policy inputs
  at workflow edges, not reimplemented as duplicate merge loops.
- Tests prove NFO and provider paths make the same replacement decision for a
  representative set of scalar, optional, list, and external-ID fields.

## In Scope

- Audit existing NFO import, provider refresh, and hierarchy confirmation merge
  behavior.
- Define a shared merge-policy model in the crate boundary that best matches
  existing dependencies.
- Move field replacement, populated-field enumeration, and source-aware lock
  filtering into the shared boundary.
- Update NFO import and metadata refresh/confirmation callers to use the shared
  model.
- Add targeted tests that prove NFO local-first, NFO remote-first, provider
  missing-only, provider full-refresh, and cross-source field locks.
- Update ADR or workstream docs if the implementation sharpens authority
  terminology.

## Out Of Scope

- NFO XML preservation, unknown XML retention, or NFO compatibility profiles.
- New TMDB, Douban, Bangumi, IMDb, or Addon provider breadth.
- Provider priority configuration UI or public API shape changes.
- Database schema migrations unless tests prove existing lock data cannot
  represent the policy.
- Catalog hydration, search projection, or refresh commit atomicity.
- Soft-link or hard-link management.
- Replacing all repository traits or broad server composition.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The shared policy should live where both `taru-metadata` and `taru-nfo` can use it without creating a dependency cycle. | High | `taru-nfo` already depends on `taru-metadata` for hierarchy confirmation; `taru-core` owns `CanonicalMetadata`, locks, and profile enums. | Choose `taru-core` for pure policy data/functions if `taru-metadata` would create coupling. |
| NFO local authority and provider refresh share field replacement mechanics but not workflow side effects. | High | NFO import writes NFO locks and may confirm hierarchy; provider refresh writes raw response, provider subject, mapping, and library confirmation. | Keep workflow side effects in their current crates and share only merge decisions. |
| No schema change is needed for the first slice. | Medium | Existing `MetadataFieldLock` stores field, source, locked flag, and item ID. | If source precedence cannot be expressed, split a schema-focused follow-up. |
| `LocalMetadataPolicy::RemoteFirst` maps to missing-only NFO import behavior for now. | High | Current `merge_nfo_metadata` uses missing-only when policy is RemoteFirst. | If product semantics change, record the decision before code changes. |

## Architecture Direction

Create a small policy boundary that speaks Taru's domain language:

- merge source: Local, NFO, Provider, or User;
- replacement mode: full replacement or missing-only;
- lock scope: protect fields locked by other sources, or respect all locked
  fields;
- field population: whether an incoming Canonical Metadata value is present.

The boundary should produce the merged `CanonicalMetadata` and, if needed,
field-level decisions that callers can use for lock writing or diagnostics.
Workflow crates should translate product context into policy inputs:

- NFO Import translates `LocalMetadataPolicy` and the current source into merge
  inputs, then writes NFO locks only for populated incoming fields when policy
  says local authority should be recorded.
- Provider refresh and hierarchy confirmation translate `MetadataRefreshMode`
  and source into merge inputs, then keep provider mapping/raw-response side
  effects inside metadata workflow code.

Do not move NFO parsing/export into metadata. Do not move provider runtime into
NFO. The shared boundary exists because Canonical Metadata authority is shared,
not because workflows are the same.

## First Slice Decision

The first implementation task should prove one shared merge authority path
without redesigning the world:

1. Add focused tests that document current NFO/provider expectations.
2. Extract shared field replacement and populated-field enumeration.
3. Rewire NFO import and provider/hierarchy merge callers to use the shared
   policy.
4. Remove the duplicated NFO merge loop.

The first slice is allowed to rename or relocate `MetadataMergePolicy` if that
produces the correct dependency direction.

## Closeout Condition

This lane can close when:

- NFO import and provider refresh no longer duplicate Canonical Metadata
  per-field merge loops;
- source-aware field lock behavior is tested for NFO and provider paths;
- targeted `taru-metadata` and `taru-nfo` tests pass;
- docs record any sharpened authority terminology;
- follow-ons for provider priority, diagnostics, or schema changes are split or
  explicitly deferred.

