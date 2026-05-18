# Managed Artwork Ingest Selection Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs
git diff --check
```

This proves the candidate/artwork/cache inventory is fresh before candidate
acceptance behavior is added.

## Gate Set

### Audit Gate

```powershell
rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs
git diff --check
```

### Managed Ingest Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests
cargo nextest run -p taru-server artwork --no-fail-fast
cargo nextest run -p taru-db artwork --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Adjust focused filters after MAIS-020 chooses the first concrete acceptance
target.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace checks if managed artwork changes shared storage, catalog,
search, public API contracts, or durable job behavior.

## Evidence Anchors

- `docs/workstreams/managed-artwork-ingest-selection/DESIGN.md`
- `docs/workstreams/managed-artwork-ingest-selection/TODO.md`
- `docs/workstreams/addon-managed-artwork-artifacts/HANDOFF.md`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/media/catalog.rs`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-db/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-server/src/app/addons.rs`

## Fresh Evidence

2026-05-19, MAIS-010:

- Workstream opened from AMAA-040 closeout as the follow-on for accepting
  internal Addon Artwork Candidates into Taru-managed artwork.
- This is a planning split only; no managed artwork runtime behavior changed.
- AMAA-030 remains the only shipped `artwork_write` behavior: candidate
  proposal. It does not fetch/cache/thumbnail/select/publish artwork.
- Fresh validation remains required before marking MAIS-020 or later tasks
  complete.

2026-05-19, MAIS-020:

- Audit inventory command:
  `rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs`.
- `crates/taru-core/src/media/artwork.rs` shows Addon Artwork Candidates are
  proposal records with internal `source_uri` and proposed/accepted/rejected
  status. There is no accept/update repository method yet.
- `ArtworkTask` is keyed by `ImageAssetId`, so it cannot represent candidate
  fetch/validate/cache before a safe public asset exists.
- `crates/taru-core/src/media/catalog.rs`,
  `crates/taru-db/src/catalog.rs`, and
  `crates/taru-api/src/public_client.rs` show `ImageAsset` currently stores
  and exposes `source_uri`, `cache_uri`, `selected`, `content_hash`, and `etag`.
  Direct candidate publication would leak raw addon/provider details.
- `crates/taru-catalog/src/lib.rs` hydrates metadata image refs into
  `ImageAsset` rows and selects the first image kind when no existing selected
  row exists. This is not an acceptance workflow for untrusted addon
  candidates.
- `crates/taru-core/src/staging.rs` and `crates/taru-db/src/staging.rs` define
  probe/FFmpeg input staging with cleanup-oriented local paths, leases, and
  budget state. It is not durable Managed Artwork authority.
- `crates/taru-server/src/app/addons.rs` correctly keeps `artwork_write` at
  candidate proposal and returns only candidate ID, image kind, status, and
  counters. Fetch/cache/thumbnail/selection remain outside the Addon Side
  Effect handler.
- Generic jobs are suitable for lifecycle visibility, but `JobResponse` exposes
  parsed `input` and `summary`, so managed artwork jobs must persist redacted
  Taru IDs and outcome counters only.
- Decision: MAIS-030 should implement a queued candidate-ingest boundary that
  creates internal Managed Artwork state. Do not create selected public
  `ImageAsset` rows as the first acceptance target.
