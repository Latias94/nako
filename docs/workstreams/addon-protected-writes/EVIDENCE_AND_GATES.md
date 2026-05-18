# Addon Protected Writes Evidence And Gates

Status: Completed
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

2026-05-18, APW-030 Canonical Metadata apply slice:

- Implemented explicit Addon Side Effect apply outcome state:
  - `crates/taru-core/src/addon.rs` adds `AddonSideEffectApplyStatus` and
    `AddonSideEffectApplyOutcome`.
  - `crates/taru-db/migrations/0023_addon_side_effect_apply_outcome.sql` adds
    `apply_status`, `apply_error_code`, `applied_item_id`, `applied_source`,
    and `applied_at`.
  - `crates/taru-db/src/addons.rs` persists and reloads apply outcomes through
    `set_addon_side_effect_apply_outcome`.
- Implemented first-class Addon metadata attribution:
  - `crates/taru-core/src/media/metadata.rs` adds
    `MetadataSource::Addon(AddonId)`.
  - `crates/taru-db/src/codec.rs` stores it as `source = addon` with
    `source_key = <addon_id>`, avoiding fake provider attribution.
  - `crates/taru-client-protocol/src/catalog.rs` and
    `crates/taru-api/src/public_client.rs` expose a stable public Addon source
    DTO for catalog records that include Addon-sourced genres/tags.
- Implemented the first concrete `metadata_write` apply path:
  - `crates/taru-server/src/app/addons.rs` still performs Addon Token
    authentication, permission/library/target validation, idempotent intake,
    and response redaction at the runtime route boundary.
  - Accepted `metadata_write` side effects are normalized into a minimal
    Canonical Metadata patch with supported fields: title-like fields,
    overview, release date, runtime, tagline, genres, and tags.
  - The patch merges through `MetadataMergePolicy` with
    `MetadataSource::Addon(addon_id)`, persists through
    `MetadataRepository::commit_metadata_item`, and hydrates catalog/search
    through `hydrate_item_catalog`.
  - Unknown payload fields fail apply with safe `invalid_payload` and do not
    echo raw payload/provenance, Source Locators, filesystem paths, provider
    bodies, token hashes, or raw Addon Tokens.
- Tests prove:
  - `crates/taru-db/src/tests.rs` round-trips Addon metadata source attribution
    and records Addon Side Effect apply outcomes.
  - `crates/taru-server/src/http/tests/addons.rs` applies an authorized
    `metadata_write`, updates Canonical Metadata, writes Addon-sourced catalog
    tags, updates search, returns redacted summaries, replays idempotently with
    the known apply outcome, records rejected intake as `skipped`, and records
    bad payload apply failures as `failed`.
- API docs:
  - `docs/api/HTTP_API.md` now documents `metadata_write` apply behavior,
    supported minimal payload fields, safe apply outcome fields, and redaction
    rules.
- Validation run before documentation closeout:
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests`
    exited 0.
  - `cargo nextest run -p taru-db addon --no-fail-fast` exited 0 with 5 tests
    passed.
  - `cargo nextest run -p taru-db sqlite_store_round_trips_metadata_policy_records --no-fail-fast`
    exited 0 with 1 test passed.
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
    exited 0 with 3 tests passed.
  - `cargo nextest run -p taru-catalog --no-fail-fast` exited 0 with 3 tests
    passed.
  - `cargo fmt --all -- --check` exited 0.
  - `git diff --check` exited 0.
  - `Get-Content docs/workstreams/addon-protected-writes/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    exited 0.
- Residual consistency note:
  - APW-030 follows the existing metadata/NFO workflow shape: metadata item
    persistence happens before catalog hydration, while catalog graph and
    search projection commit atomically inside `CatalogRepository`.
  - This proves catalog/search consistency for the successful apply path, not
    a single database transaction spanning media item update plus catalog
    hydration. A future prepared-catalog unit-of-work remains separate design
    scope.

2026-05-18, APW-030 fresh resume verification:

- `cargo fmt --all -- --check` exited 0.
- `git diff --check` exited 0. Git reported Windows LF-to-CRLF working-copy
  warnings only; no whitespace errors were reported.
- `Get-Content docs/workstreams/addon-protected-writes/WORKSTREAM.json |
  ConvertFrom-Json | Out-Null` exited 0.
- `cargo nextest run -p taru-server addon_side_effect --no-fail-fast` exited 0
  with 3 tests passed.
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server
  -p taru-metadata -p taru-catalog --tests` exited 0.
- This fresh evidence verifies the APW-030 completion claim after context
  resume and before closing the Codex goal.

2026-05-18, APW-060 closeout review and split:

- Review result:
  - Workstream compliance had no blocking findings after APW-030 because the
    lane proved a Taru-owned protected-write apply model for Canonical
    Metadata.
  - Code-quality review found one important catalog provenance issue before
    closeout: scalar Addon metadata patches could trigger full catalog
    hydration with Addon source and rewrite existing provider/NFO genre/tag
    graph sources.
- Fix applied before closeout:
  - `crates/taru-core/src/repository/catalog.rs` adds a narrow search
    projection update seam.
  - `crates/taru-db/src/catalog.rs` implements search-only projection update.
  - `crates/taru-catalog/src/lib.rs` adds search-only refresh and selected
    genre/tag hydration helpers.
  - `crates/taru-server/src/app/addons.rs` now refreshes search for scalar
    metadata patches and only replaces touched genre/tag label sets with
    `MetadataSource::Addon(addon_id)`.
  - `crates/taru-server/src/http/tests/addons.rs` proves scalar-only and
    tags-only patches preserve unrelated catalog graph source attribution.
- Follow-on split:
  - `docs/workstreams/addon-managed-artwork-artifacts/` owns `artwork_write`,
    Artwork Candidate, Managed Artwork, Taru-Managed Artifact storage, fetch
    ownership, cache/thumbnail policy, resource budgets, and safe diagnostics.
  - `docs/workstreams/addon-library-file-write-policy/` owns subtitle, NFO,
    sidecar-asset Library File Write target derivation, NFO Round Trip, backup
    policy, VFS write reports, and response redaction.
- Fresh validation after the closeout fix:
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast` exited
    0 with 5 tests passed.
  - `cargo nextest run -p taru-catalog --no-fail-fast` exited 0 with 3 tests
    passed.
  - `cargo nextest run -p taru-db addon --no-fail-fast` exited 0 with 5 tests
    passed.
  - `cargo nextest run -p taru-db sqlite_store_round_trips_metadata_policy_records --no-fail-fast`
    exited 0 with 1 test passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server
    -p taru-metadata -p taru-catalog --tests` exited 0.
- Final closeout gates after APW-060 documentation edits:
  - `cargo fmt --all -- --check` exited 0.
  - `git diff --check` exited 0. Git reported Windows LF-to-CRLF working-copy
    warnings only; no whitespace errors were reported.
  - `Get-Content docs/workstreams/addon-protected-writes/WORKSTREAM.json |
    ConvertFrom-Json | Out-Null` exited 0.
  - `Get-Content docs/workstreams/addon-managed-artwork-artifacts/WORKSTREAM.json |
    ConvertFrom-Json | Out-Null` exited 0.
  - `Get-Content docs/workstreams/addon-library-file-write-policy/WORKSTREAM.json |
    ConvertFrom-Json | Out-Null` exited 0.
- APW status is now completed. Continue in AMAA-020 or ALFW-020 depending on
  the next product priority.
