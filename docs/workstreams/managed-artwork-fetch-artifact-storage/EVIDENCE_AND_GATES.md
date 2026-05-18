# Managed Artwork Fetch Artifact Storage Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "ManagedArtworkIngest|managed_artwork_ingest|managed_artwork_artifacts|JobKind::ManagedArtworkIngest|artwork.ingest|storage_uri|ImageAsset|cache_uri|source_uri|thumbnail" crates docs
git diff --check
```

This proves the managed artwork ingest, storage, public image, and thumbnail
seams are freshly inventoried before fetch/artifact behavior is added.

## Gate Set

### Audit Gate

```powershell
rg -n "ManagedArtworkIngest|managed_artwork_ingest|managed_artwork_artifacts|JobKind::ManagedArtworkIngest|artwork.ingest|storage_uri|ImageAsset|cache_uri|source_uri|thumbnail" crates docs
git diff --check
```

### Fetch/Artifact Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests
cargo nextest run -p taru-server artwork --no-fail-fast
cargo nextest run -p taru-db artwork --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Adjust focused filters after MAFA-020 chooses the first concrete worker and
storage target.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace checks if the lane changes shared job runtime, storage
ports, public API contracts, or durable job behavior.

## Evidence Anchors

- `docs/workstreams/managed-artwork-fetch-artifact-storage/DESIGN.md`
- `docs/workstreams/managed-artwork-fetch-artifact-storage/TODO.md`
- `docs/workstreams/managed-artwork-ingest-selection/HANDOFF.md`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/job.rs`
- `crates/taru-db/migrations/0026_managed_artwork_ingest.sql`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/app/job_runtime.rs`
- `crates/taru-vfs`

## Fresh Evidence

2026-05-19, MAFA-010:

- Workstream opened from MAIS-040 closeout as the follow-on for processing
  queued managed artwork ingest jobs into internal managed artifact bytes.
- This is a planning split only; no fetch, validation, byte storage, public
  serving, thumbnail, or selected artwork runtime behavior changed.
- MAIS remains the authority for the queued acceptance boundary:
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- Fresh validation remains required before marking MAFA-020 or later tasks
  complete.
- MAFA-010 validation:
  - `Get-Content -Raw docs/workstreams/managed-artwork-fetch-artifact-storage/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.

2026-05-19, MAFA-020 seam audit and first-target decision:

- Audit inventory command:
  `rg -n "ManagedArtworkIngest|managed_artwork_ingest|managed_artwork_artifacts|JobKind::ManagedArtworkIngest|artwork.ingest|storage_uri|ImageAsset|cache_uri|source_uri|thumbnail" crates docs`.
  Output was redirected to a temp file for review and contained 537 inventory
  lines.
- `crates/taru-server/src/app/job_runtime.rs` can run a known job ID and
  persist success/failure, but it does not claim the next queued job or couple
  job status with `managed_artwork_ingests.status`.
- `crates/taru-db/src/jobs.rs` exposes generic `start_job`, `succeed_job`, and
  `fail_job`, but `start_job` is not conditional on queued status. A
  managed-artwork-specific claim method is needed to avoid duplicate workers
  racing the same ingest.
- `crates/taru-db/src/artwork.rs` can create and load managed ingests, but has
  no artifact insert, ingest status transition, safe failure-code update, or
  artifact commit method yet.
- `crates/taru-db/migrations/0026_managed_artwork_ingest.sql` already has
  `managed_artwork_artifacts` and `managed_artwork_ingests.artifact_id`, but
  there is no worker or byte-storage implementation.
- `crates/taru-server/src/app/staging.rs` and `crates/taru-core/src/staging.rs`
  are probe/FFmpeg input staging with budget, leases, cleanup, and local path
  diagnostics. They are not durable managed artwork authority.
- `crates/taru-vfs/src/cache.rs` is a remote storage fact cache. It should not
  become managed artwork artifact authority.
- `crates/taru-vfs/src/local.rs` is useful reference for path safety and
  atomic local writes, but the backend is library-root oriented and currently
  text-write shaped. Managed artwork needs a purpose-built internal artifact
  storage port.
- `crates/taru-metadata/src/runtime.rs` has a good HTTP runtime pattern for
  timeout, attempts, concurrency, proxy, and user agent, but it is JSON/provider
  shaped. Artwork fetch should use a dedicated byte-stream fetcher port.
- Decision:
  - MAFA-030 should implement a dedicated managed artwork worker/runtime
    boundary with managed-artwork-specific claim/commit repository methods.
  - The first artifact byte store should be a Taru-owned local internal
    artifact root, not a library root, VFS cache, or staging manifest.
  - Persist opaque `managed-artwork://...` storage references and keep raw
    absolute paths out of database authority and DTOs.
  - Fetch only accepted HTTP(S) remote candidates with hard timeout, attempt,
    concurrency, byte-length, media-type, and dimension/decodability limits.
  - Keep public `ImageAsset`, image serving, thumbnails, and selected artwork
    publication split to later lanes.
- MAFA-020 validation:
  - `Get-Content -Raw docs/workstreams/managed-artwork-fetch-artifact-storage/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
