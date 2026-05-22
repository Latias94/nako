# Addon Managed Artwork Artifacts Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Nako-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs
git diff --check
```

This proves the artwork/artifact inventory is fresh before `artwork_write`
behavior is added.

## Gate Set

### Audit Gate

```powershell
rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Nako-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs
git diff --check
```

### Artwork Apply Gate

```powershell
cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-vfs --tests
cargo nextest run -p nako-server addon_side_effect --no-fail-fast
cargo nextest run -p nako-server artwork --no-fail-fast
cargo nextest run -p nako-db artwork --no-fail-fast
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
- `crates/nako-core/src/media/catalog.rs`
- `crates/nako-core/src/media/artwork.rs`
- `crates/nako-db/src/artwork.rs`
- `crates/nako-server/src/app/addons.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-18, AMAA-010:

- Workstream opened from APW-060 closeout as the follow-on for
  `artwork_write`, Artwork Candidate, Managed Artwork, and Nako-Managed
  Artifact behavior.
- This is a planning split only; no artwork runtime behavior changed.
- Fresh validation remains required before marking AMAA-020 or later tasks
  complete.

2026-05-18, core-architecture-deepening CAD-070 alignment:

- AMAA remains proposed; no `artwork_write` runtime behavior exists yet.
- Added explicit guidance that future artwork writes must use or introduce a
  Nako-owned artwork/catalog commit boundary if they need multi-step durable
  state.
- Artwork sidecar export is classified as Library File Write work owned by
  `addon-library-file-write-policy`, not AMAA.

2026-05-19, AMAA-020 seam audit and first-target decision:

- Audit inputs:
  - `crates/nako-core/src/addon.rs` already defines the `artwork_write`
    permission and grant/access-check tests cover it, but runtime apply still
    skips unsupported protected-write permissions.
  - `crates/nako-core/src/media/catalog.rs` defines `ImageAsset` with
    `source_uri`, `provider`, optional `cache_uri`, dimensions, language,
    selected state, hash, and etag.
  - `crates/nako-db/migrations/0007_catalog_ingestion.sql` and
    `crates/nako-db/src/catalog.rs` persist `image_assets` and enforce
    uniqueness by owner, kind, and `source_uri`.
  - `crates/nako-api/src/public_client.rs` maps `ImageAsset` to Public Client
    DTOs that include `source_uri` and `cache_uri`.
  - `crates/nako-core/src/media/artwork.rs` and
    `crates/nako-db/src/artwork.rs` define and persist `ArtworkTask` with
    resource classes for fetch, resize, preview, and cleanup, but worker
    execution, thumbnail generation, and cache eviction are future work per ADR
    0013.
  - `crates/nako-core/src/staging.rs` currently has staging purposes for probe
    and FFmpeg input only; no artwork artifact or image-fetch staging purpose
    exists.
  - `crates/nako-core/src/automation.rs` has Automation Artifacts for
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
  - Nako-Managed Artifact intake is deferred because there is no dedicated
    Addon artifact store, upload policy, or image content validation boundary.
  - Direct `ImageAsset` selection is deferred because it would be public catalog
    state and needs managed cache/redaction guarantees first.
- Validation:
  - `rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Nako-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs`
    completed successfully; output was redirected to a temp file for review and
    contained 561 inventory lines.
  - `Get-Content -Raw docs\workstreams\addon-managed-artwork-artifacts\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    the edited workstream docs.

2026-05-19, AMAA-030 MediaItem-targeted Addon Artwork Candidate proposal:

- Implementation:
  - Added `ArtworkCandidateId`, `NewArtworkCandidate`,
    `ArtworkCandidateRecord`, `ArtworkCandidateSourceKind`, and
    `ArtworkCandidateStatus` in `nako-core`.
  - Added `ArtworkCandidateRepository` and a SQLite-backed
    `addon_artwork_candidates` table. The table stores internal source details
    with constraints for remote URL source kind, dimensions, language length,
    and candidate status.
  - Added `artwork_write` apply handling in the Addon Side Effect service. The
    first slice accepts only MediaItem-targeted `propose_artwork` payloads with
    `poster`, `backdrop`, `logo`, `banner`, or `thumbnail` kinds and HTTP(S)
    remote URL sources.
  - `artwork_write` MediaSource targets are rejected during intake validation
    as invalid targets. `library_file_write` MediaItem targets are also
    rejected during target validation so permission-specific target authority is
    explicit.
  - Apply reports use `applied_source: "artwork_candidate"` and include only
    candidate ID, image kind, status, and candidate-created/existing counters.
- Redaction and boundary guarantees:
  - The response and replay response do not include raw payloads, provenance,
    remote URLs, Source Locators, filesystem paths, cache URIs, raw token
    material, or `source_uri`/`cache_uri` DTO fields.
  - The apply path does not write public `ImageAsset` rows, selected artwork,
    managed cache artifacts, thumbnails, or sidecar files.
  - Unsafe payloads with `cache_uri`, `selected`, non-HTTP(S) URLs, `file:`,
    `local:`, and data URI sources are rejected as `invalid_payload` without
    echoing unsafe details.
- API/docs:
  - `docs/api/HTTP_API.md` now documents the `artwork_write` candidate payload,
    target kind, source restrictions, and redacted apply report.
  - Workstream docs advance AMAA to AMAA-040 for closeout or follow-on split.
- Validation during implementation:
  - `cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-vfs --tests`
    passed.
  - `cargo nextest run -p nako-db artwork --no-fail-fast` passed: 2 tests run,
    2 passed.
  - `cargo nextest run -p nako-server artwork --no-fail-fast` passed: 2 tests
    run, 2 passed.
  - `cargo nextest run -p nako-server addon_side_effect --no-fail-fast` passed:
    10 tests run, 10 passed.
- Final validation before commit:
  - `cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-vfs --tests`
    passed.
  - `cargo nextest run -p nako-db artwork --no-fail-fast` passed: 2 tests run,
    2 passed.
  - `cargo nextest run -p nako-server artwork --no-fail-fast` passed: 2 tests
    run, 2 passed.
  - `cargo nextest run -p nako-server addon_side_effect --no-fail-fast` passed:
    10 tests run, 10 passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
  - `Get-Content -Raw docs\workstreams\addon-managed-artwork-artifacts\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.

2026-05-19, AMAA-040 closeout review and split:

- AMAA-030 was committed as `8c2e74d feat(addons): propose artwork candidates
  from side effects`.
- Review result:
  - Workstream compliance has no blocking findings. The shipped behavior
    matches the selected first target: MediaItem-targeted Addon Artwork
    Candidate proposals through `artwork_write`.
  - Code-quality review has no blocking findings. The Addon handler
    authenticates, validates, records, normalizes a small candidate command,
    and delegates candidate persistence to repository seams; it does not own
    remote fetch, image validation, cache/artifact storage, thumbnailing,
    selected artwork, or public catalog-image publication.
  - Redaction review has no blocking findings. Responses and stored
    `apply_report` values expose only safe IDs/statuses/counters, not raw
    payloads, provenance, Source Locators, filesystem paths, remote handles,
    cache URIs, `source_uri`, or `cache_uri` public DTO fields.
- Closeout decision:
  - Close this lane after proving one safe `artwork_write` path.
  - Split Candidate acceptance, remote fetch, image validation, cache URI
    assignment, thumbnail generation, selected artwork, and public `ImageAsset`
    publication to `docs/workstreams/managed-artwork-ingest-selection/`.
  - Keep artwork sidecar export in `addon-library-file-write-policy` because it
    is Library File Write behavior, not Managed Artwork behavior.
- Final closeout gates after AMAA-040 documentation edits:
  - `Get-Content -Raw docs\workstreams\addon-managed-artwork-artifacts\WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
