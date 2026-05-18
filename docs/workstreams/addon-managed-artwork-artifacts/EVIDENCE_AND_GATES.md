# Addon Managed Artwork Artifacts Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs
git diff --check
```

This proves the artwork/artifact inventory is fresh before `artwork_write`
behavior is added.

## Gate Set

### Audit Gate

```powershell
rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs
git diff --check
```

### Artwork Apply Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests
cargo nextest run -p taru-server artwork --no-fail-fast
cargo nextest run -p taru-db artwork --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Adjust focused filters after AMAA-020 chooses the first concrete target.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace checks if artwork changes shared storage, catalog, search,
or public API contracts.

## Evidence Anchors

- `docs/workstreams/addon-managed-artwork-artifacts/DESIGN.md`
- `docs/workstreams/addon-managed-artwork-artifacts/TODO.md`
- `docs/workstreams/addon-protected-writes/HANDOFF.md`
- `crates/taru-core/src/media/catalog.rs`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-server/src/app/addons.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-18, AMAA-010:

- Workstream opened from APW-060 closeout as the follow-on for
  `artwork_write`, Artwork Candidate, Managed Artwork, and Taru-Managed
  Artifact behavior.
- This is a planning split only; no artwork runtime behavior changed.
- Fresh validation remains required before marking AMAA-020 or later tasks
  complete.

2026-05-18, core-architecture-deepening CAD-070 alignment:

- AMAA remains proposed; no `artwork_write` runtime behavior exists yet.
- Added explicit guidance that future artwork writes must use or introduce a
  Taru-owned artwork/catalog commit boundary if they need multi-step durable
  state.
- Artwork sidecar export is classified as Library File Write work owned by
  `addon-library-file-write-policy`, not AMAA.

2026-05-19, AMAA-020 seam audit and first-target decision:

- Audit inputs:
  - `crates/taru-core/src/addon.rs` already defines the `artwork_write`
    permission and grant/access-check tests cover it, but runtime apply still
    skips unsupported protected-write permissions.
  - `crates/taru-core/src/media/catalog.rs` defines `ImageAsset` with
    `source_uri`, `provider`, optional `cache_uri`, dimensions, language,
    selected state, hash, and etag.
  - `crates/taru-db/migrations/0007_catalog_ingestion.sql` and
    `crates/taru-db/src/catalog.rs` persist `image_assets` and enforce
    uniqueness by owner, kind, and `source_uri`.
  - `crates/taru-api/src/public_client.rs` maps `ImageAsset` to Public Client
    DTOs that include `source_uri` and `cache_uri`.
  - `crates/taru-core/src/media/artwork.rs` and
    `crates/taru-db/src/artwork.rs` define and persist `ArtworkTask` with
    resource classes for fetch, resize, preview, and cleanup, but worker
    execution, thumbnail generation, and cache eviction are future work per ADR
    0013.
  - `crates/taru-core/src/staging.rs` currently has staging purposes for probe
    and FFmpeg input only; no artwork artifact or image-fetch staging purpose
    exists.
  - `crates/taru-core/src/automation.rs` has Automation Artifacts for
    automation jobs, but that model is provider/job-shaped rather than Addon
    Side Effect-shaped.
- Decision: AMAA-030 should implement MediaItem-targeted Addon Artwork
  Candidate proposal as the first `artwork_write` apply path.
- Target semantics:
  - The candidate target is `media_item` first. Person/collection/studio
    artwork and source-derived thumbnails are deferred until owner and
    selection semantics are explicit.
  - The first payload should be typed around candidate intent, image kind,
    HTTP(S) remote URL source metadata, optional dimensions, and optional
    language.
  - The payload must reject filesystem paths, Source Locators, remote storage
    handles, raw image bytes, data URIs, `cache_uri`, `selected`, and sidecar
    export fields.
  - The apply response/report must expose only redacted IDs, status, image kind,
    and aggregate candidate counters. It must not echo raw URLs, cache URIs,
    paths, Source Locators, remote handles, or raw payload content.
- Core architecture alignment:
  - Do not write the current public `ImageAsset` table directly for the first
    slice. Public catalog DTOs expose `source_uri` and `cache_uri`; direct
    insertion would turn unverified addon output into client-visible hotlinks.
  - Do not implement fetch/cache/thumbnail/selection inside the Addon handler.
    Later slices should use or introduce first-party artwork/artifact services,
    task workers, storage policy, and catalog commit boundaries.
  - Artwork sidecar export remains Library File Write scope, not AMAA.
- Deferred alternatives:
  - Managed Artwork import is deferred because artifact storage, content
    validation, cache URI assignment, and fetch/resize workers are not yet
    cohesive enough for a first slice.
  - Taru-Managed Artifact intake is deferred because there is no dedicated
    Addon artifact store, upload policy, or image content validation boundary.
  - Direct `ImageAsset` selection is deferred because it would be public catalog
    state and needs managed cache/redaction guarantees first.
- Validation:
  - `rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs`
    completed successfully; output was redirected to a temp file for review and
    contained 561 inventory lines.
  - `Get-Content -Raw docs\workstreams\addon-managed-artwork-artifacts\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    the edited workstream docs.
