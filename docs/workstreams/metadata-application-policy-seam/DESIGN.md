# Metadata Application Policy Seam

Status: Completed
Last updated: 2026-05-26

## Why This Lane Exists

Nako already has a shared `MetadataMergePolicy` in `nako-core`, catalog graph
projection in `nako-catalog`, provider refresh orchestration in
`nako-metadata`, and protected Addon `metadata_write` Side Effects in
`nako-server`. The missing Module is the host-owned application seam that turns
incoming Canonical Metadata facts into a committed Media Item plus catalog
projection and report.

Today `crates/nako-server/src/app/addons/metadata_write.rs` parses Addon
payloads, normalizes protocol fields, chooses merge policy, loads field locks,
plans catalog projection, and builds the Addon persistence commit. That makes
the Addon adapter shallow: callers and future maintainers must understand too
much host authority logic in a side-effect-specific file.

## Target State

When this lane closes:

- `MetadataApplication` is an explicit server app Module with a small
  Interface.
- Addon `metadata_write` is an Adapter: protocol payload validation,
  protocol-to-Canonical Metadata mapping, target resolution, and a call into
  `MetadataApplication`.
- Host policy decides application mode from request/provenance/library profile,
  not from the Addon Sidecar.
- Field locks, source-aware merge behavior, catalog graph/search projection,
  and apply report generation are local to `MetadataApplication`.
- Addon writeback honors `MetadataRefreshMode::MissingOnly` when scan-time
  library policy asks for missing-only application.
- Same-source Addon refresh behavior is covered by tests instead of inferred
  from implementation.
- Existing provider refresh and hierarchy confirmation seams are audited for
  reuse without creating a `nako-metadata -> nako-server` dependency cycle.

## In Scope

- Workstream docs and closeout evidence.
- Characterization tests for Addon metadata writeback policy behavior.
- `crates/nako-server/src/app/metadata_application.rs`.
- Refactoring Addon `metadata_write` into a thin Adapter.
- Host policy selection for scan-triggered Addon writeback.
- Focused server/core gates and whitespace/format checks.

## Out Of Scope

- Provider breadth or provider-specific matching behavior.
- Official Addon engine cleanup in `nako-official-addons`.
- Scan Addon bulk continuation and cursor persistence.
- Addon install, distribution, health, or task scheduler changes.
- Frontend UI.
- Database schema changes unless a report field proves unavoidable.
- Public API changes beyond existing Addon Side Effect payload semantics.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Host owns metadata authority and Addon Sidecars should remain fact submitters. | High | ADR 0007 and ADR 0035. | Reopen Addon writeback architecture before changing Sidecar semantics. |
| `MetadataMergePolicy` should stay in `nako-core`. | High | `metadata-merge-policy-unification` completed this direction. | If application mode logic becomes pure and cross-crate, add only pure types to core. |
| First `MetadataApplication` belongs in server app. | High | It needs repository access and catalog projection. | Split pure command/result later if provider or NFO paths need it. |
| Scan-time Addon writeback should use library profile refresh mode. | Medium | Library profiles already own `MetadataRefreshMode`. | Add an explicit application mode field if library profile is too coarse. |
| Baseline Addon writeback payload shape remains valid. | High | ADR 0035 accepts canonical metadata-shaped patches. | Add protocol versioning only if a payload incompatibility appears. |

## Architecture Direction

The accepted shape is a server app Module:

```rust
MetadataApplicationCommand {
    item,
    source,
    incoming,
    mode,
    provenance,
}
```

`MetadataApplication` owns:

- loading field locks;
- selecting `MetadataMergePolicy`;
- applying source-aware merge rules;
- building the updated `MediaItem`;
- planning catalog graph/search projection;
- returning a persistence-ready result and a safe apply report.

Addon `metadata_write` remains responsible for:

- parsing and validating `AddonMetadataPatch`;
- mapping protocol payloads into `CanonicalMetadata`;
- resolving `AddonSideEffectTarget` to a `MediaItem`;
- deriving Addon provenance;
- delegating to `MetadataApplication`.

Provider refresh and hierarchy confirmation should not be forced through server
app code during the first slice. They already live in `nako-metadata` and rely
on ports. The closeout audit should decide whether a future pure
application-decision type belongs in `nako-core`, while keeping repository and
catalog side effects out of `nako-core`.

## Policy Notes

- Addon Sidecars do not choose merge policy.
- Addon Side Effects may carry provenance and target information, but not host
  authority.
- Field locks from other sources protect fields from Addon writes.
- Same-source Addon writes may refresh fields locked by that same Addon source;
  this behavior must be explicit in tests.
- Scan-time Addon writeback starts with the Media Library profile refresh mode.

## Closeout Condition

This lane can close when:

- characterization tests prove Addon writeback missing-only, lock, same-source,
  and catalog projection behavior;
- Addon `metadata_write` no longer owns merge policy or catalog projection;
- `MetadataApplication` is the single host-owned Module for Addon metadata
  application;
- scan-triggered Addon writeback uses host policy instead of hard-coded full
  refresh;
- focused gates pass and evidence is recorded;
- official Addon adapter cleanup and bulk continuation are split as follow-ons.

## Closeout Result

Completed on 2026-05-26. `crates/nako-server/src/app/metadata_application.rs`
is now the host-owned application seam for Addon Canonical Metadata writes. It
accepts a small command containing the target item, source, incoming metadata,
application mode, and provenance; then it resolves library profile policy,
loads field locks, applies `MetadataMergePolicy`, plans catalog graph/search
projection, and returns a safe report.

`crates/nako-server/src/app/addons/metadata_write.rs` no longer chooses
`MetadataRefreshMode::FullRefresh`, reads field locks, invokes
`MetadataMergePolicy`, or calls `plan_item_catalog_projection` directly. It is
now a protocol Adapter that parses and validates `AddonMetadataPatch`, maps it
to `CanonicalMetadata`, resolves the Side Effect target, and delegates to
`MetadataApplication`.

Provider refresh and hierarchy confirmation were audited but intentionally not
forced through the server app Module. They already live in `nako-metadata`
behind ports and still use the shared `nako-core::MetadataMergePolicy`. A
future lane can extract pure application-decision types into `nako-core` if
provider, NFO, and Addon paths need one common command/result without a
`nako-metadata -> nako-server` dependency.
