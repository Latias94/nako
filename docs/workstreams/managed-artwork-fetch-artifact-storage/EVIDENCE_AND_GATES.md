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
