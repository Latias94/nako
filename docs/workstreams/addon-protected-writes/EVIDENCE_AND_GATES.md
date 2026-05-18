# Addon Protected Writes Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs
git diff --check
```

This proves the current protected-write inventory is fresh before concrete
metadata, artwork, subtitle, NFO, or Library File Write handlers are added.

## Gate Set

### Audit Gate

```powershell
rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs
git diff --check
```

Proves APW-020 has current file anchors for the existing intake and write
boundaries.

### Canonical Metadata Apply Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Add focused metadata/catalog tests selected by APW-020. The gate must prove
that a valid `metadata_write` side effect applies through Taru-owned metadata
and catalog seams, and that denied or replayed requests remain safe.

### Artwork And Artifact Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests
git diff --check
```

Add focused artwork/addon tests after APW-020 identifies the concrete model.
The gate must prove artwork/artifact outputs do not become raw provider hotlinks
or path leaks.

### Library File Write Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests
cargo fmt --all -- --check
git diff --check
```

Add focused NFO/storage/addon tests after APW-020 identifies the concrete
write path. The gate must prove Library File Write behavior remains Taru-owned,
redacted, and compatible with NFO Round Trip and backup policy.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to `cargo check --workspace --tests` and `cargo nextest run --workspace
--no-fail-fast` if protected-write changes affect shared repository, catalog,
metadata, storage, or API boundaries across the workspace.

### Review Gate

Run `review-workstream` before accepting APW-020, before accepting any concrete
protected-write apply task, and before lane closeout. Record blocking findings,
missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/addon-protected-writes/DESIGN.md`
- `docs/workstreams/addon-protected-writes/TODO.md`
- `docs/workstreams/addon-protected-writes/MILESTONES.md`
- `docs/workstreams/addon-token-grants-side-effects/EVIDENCE_AND_GATES.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `crates/taru-core/src/addon.rs`
- `crates/taru-core/src/repository/addon.rs`
- `crates/taru-db/src/addons.rs`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-api/src/extension.rs`
- code/test paths proving concrete protected-write behavior after
  implementation

## Fresh Evidence

2026-05-18, APW-010:

- Workstream opened from ATGSE-060 closeout as the follow-on for concrete
  Addon protected writes.
- Scope is intentionally after Addon Token, accepted grant, addon-principal,
  and Addon Side Effect intake proof.
- First executable task set to APW-020 protected write seam audit before
  changing metadata, artwork, subtitle, NFO, or Library File Write behavior.
- Workstream index and ATGSE handoff point to this lane.
- Validation is recorded in ATGSE-060 closeout evidence.

2026-05-18, APW-020 protected write seam audit:

- Audit command:
  - `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`
- Addon Side Effect intake anchors:
  - `crates/taru-core/src/addon.rs` defines `AddonPermission`,
    `AddonSideEffectTargetKind`, `AddonSideEffectValidationStatus`,
    `NewAddonSideEffect`, `AddonSideEffectRecord`, and `AddonPrincipal`.
  - `crates/taru-db/migrations/0022_addon_side_effects.sql` stores actor,
    token, permission, concrete Media Library, target, idempotency key,
    provenance JSON, payload JSON, validation status, safe error code, and
    creation time.
  - `crates/taru-db/src/addons.rs` persists intake idempotently by
    `(addon_id, idempotency_key)`.
  - `crates/taru-api/src/extension.rs` exposes generic JSON payload/provenance
    on request, but response summaries exclude raw payload/provenance.
  - `crates/taru-server/src/app/addons.rs` authenticates the Addon Token,
    enforces accepted permission and concrete library target, records rejected
    intake after a trustworthy Addon principal is resolved, and returns only
    safe summaries.
- Intake gap:
  - `validation_status` has only `accepted` and `rejected`. It proves
    authorization/target validation, not domain apply.
  - There is no apply status, applied-at timestamp, apply error code, applied
    item/source summary, or post-apply replay result.
  - APW-030 must not overload `AddonSideEffectValidationStatus` to mean
    "metadata write applied".
- Canonical Metadata seam anchors:
  - `crates/taru-core/src/media/item.rs` owns `CanonicalMetadata`.
  - `crates/taru-core/src/media/merge.rs` owns `MetadataMergePolicy`,
    `MetadataMergeMode`, source-aware lock handling, and populated-field
    discovery.
  - `crates/taru-metadata/src/strategy.rs` uses `MetadataMergePolicy`,
    `MetadataRefreshPort`, `MetadataAttemptPort`, and `hydrate_item_catalog`
    for provider refresh.
  - `crates/taru-nfo/src/import.rs` uses
    `MetadataMergePolicy::for_nfo_import`, writes NFO field locks when policy
    requires it, and hydrates catalog/search after changes.
  - `crates/taru-db/src/metadata.rs` exposes
    `MetadataRepository::commit_metadata_refresh`, which atomically persists
    media item metadata, provider raw response, provider subject, provider
    mapping, and library item confirmation.
- Metadata gap:
  - Existing metadata commit is provider-shaped because it requires
    `ProviderRawResponse` and provider mapping. Addon metadata writes need a
    Taru-owned commit path that does not fabricate provider subjects or raw
    provider responses.
  - `crates/taru-core/src/media/metadata.rs::MetadataSource` has `Local`,
    `Nfo`, `Provider`, and `User`, but no Addon variant.
  - `crates/taru-db/src/codec.rs::metadata_source_from_parts` maps unknown
    source strings to `Provider(ExternalProvider::Other(source))`; using that
    fallback for Addon writes would blur Addon provenance into provider
    provenance.
  - APW-030 should add first-class Addon metadata source attribution before it
    writes locks, mappings, catalog source facts, or event payloads.
- Catalog/search seam anchors:
  - `crates/taru-catalog/src/lib.rs::hydrate_item_catalog` loads the current
    item, builds graph replacement from Canonical Metadata, builds search
    projection, and commits both through `CatalogRepository`.
  - `crates/taru-core/src/repository/catalog.rs::commit_item_projection`
    declares the graph-plus-search commit boundary.
  - `crates/taru-db/src/catalog.rs::commit_item_projection` commits graph
    replacement and search projection in one SQLite transaction.
- Event/audit gap:
  - `crates/taru-core/src/event.rs::DomainEventKind` has
    `ItemMetadataRefreshed`, but no addon-specific metadata-applied event.
  - APW-030 can either add a safe event kind or emit a clearly scoped metadata
    event payload, but it must not include raw payload/provenance, Source
    Locators, filesystem paths, token hashes, or raw Addon Tokens.
- Artwork seam anchors:
  - `crates/taru-core/src/media/catalog.rs::ImageAsset` stores artwork-like
    asset records with source URI, optional cache URI, dimensions, language,
    selected state, hash, and etag.
  - `crates/taru-core/src/media/artwork.rs::ArtworkTask` and
    `crates/taru-db/src/artwork.rs` provide task persistence for fetch, resize,
    preview, and cleanup.
- Artwork gap:
  - There is not yet a cohesive Managed Artwork apply service that turns Addon
    output into fetched/cached/selected artwork with artifact budgets and safe
    diagnostics.
  - Artwork should remain APW-040 or split into a dedicated artwork/artifact
    lane if image processing or cache policy dominates.
- NFO/VFS seam anchors:
  - `crates/taru-nfo/src/export.rs` exports via
    `StorageWriteRequest::atomic_replace` and requests same-directory backup
    when overwriting an existing sidecar.
  - `crates/taru-vfs/src/lib.rs` defines `StorageWriteMode`,
    `StorageBackupPolicy`, `StorageWriteRequest`, and `StorageWriteReport`.
  - `crates/taru-vfs/src/local.rs` implements local direct and atomic replace
    writes plus backup handling.
- NFO/VFS gap:
  - Addon-driven Library File Write requires target/path derivation, sidecar
    policy, backup semantics, and redacted report shaping before implementation.
  - NFO/subtitle/sidecar writes are too broad for the first concrete apply
    slice and remain APW-050 or follow-on lanes.
- First apply target decision:
  - Choose Canonical Metadata for APW-030.
  - Required APW-030 preconditions: explicit side-effect apply outcome state,
    Addon metadata source attribution, minimal metadata payload normalization,
    merge through `MetadataMergePolicy`, Taru-owned persistence, catalog/search
    hydration, redacted response, and idempotent replay after apply.
- ADR impact:
  - No ADR amendment is required if APW-030 preserves ADR 0020 and Taru-owned
    APIs, permissions, library grants, audit, and resource boundaries.
  - Split an ADR if the implementation chooses direct storage authority, Public
    Client write APIs, Admin API reuse, OAuth-first authorization, or fake
    provider attribution for Addon writes.
- Task status: DONE. This was a docs/audit task only; no Rust behavior changed.
- Validation passed:
  - `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`
  - `git diff --check`
- Fresh closeout verification passed:
  - `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`
    exited 0 and returned 1125 matches.
  - `git diff --check` exited 0.
  - `Get-Content docs/workstreams/addon-protected-writes/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    exited 0.
  - `cargo fmt --all -- --check` exited 0.
- Broader Rust tests were not run because APW-020 changed only workstream
  documentation and made no Rust behavior, schema, or public API changes.

Fresh verification is required before marking any later task, Codex goal, or
lane complete.
