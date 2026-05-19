# Managed Artwork Ingest Selection Evidence And Gates

Status: Completed
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

2026-05-19, MAIS-030:

- Implemented `JobKind::ManagedArtworkIngest` with resource class
  `artwork.ingest`.
- Added `ManagedArtworkIngestId`, `ManagedArtworkArtifactId`, internal
  managed artwork ingest/artifact domain records, and
  `ManagedArtworkRepository`.
- Added migration `0026_managed_artwork_ingest.sql` for
  `managed_artwork_ingests` and `managed_artwork_artifacts`.
- `SqliteStore::accept_managed_artwork_candidate_ingest` commits candidate
  status, durable job, and managed ingest state in one transaction; repeated
  acceptance returns the existing ingest and job.
- Added `ManagedArtworkAppService::accept_candidate` and
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- The Admin accept response uses `AcceptManagedArtworkCandidateResponse`, which
  returns candidate ID/status, managed ingest summary, and a redacted job
  envelope only.
- Added tests:
  `sqlite_store_accepts_artwork_candidate_into_managed_ingest_atomically` and
  `admin_accept_artwork_candidate_queues_managed_ingest_without_public_artwork_or_url_echo`.
- The shipped slice does not fetch remote bytes, create thumbnails, write
  public `ImageAsset` rows, set selected artwork, or expose client-visible
  cache references.
- API docs now describe `managed_artwork_ingest` jobs, Admin candidate accept,
  idempotent accept replay, and redaction guarantees.
- Fresh validation:
  - `cargo nextest run -p taru-db accepts_artwork_candidate --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server admin_accept_artwork_candidate --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db artwork --no-fail-fast` passed.
  - `cargo nextest run -p taru-server artwork --no-fail-fast` passed.
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.

2026-05-19, MAIS-030 pre-commit refresh:

- `Get-Content -Raw docs/workstreams/managed-artwork-ingest-selection/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only LF/CRLF working-copy warnings.
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
  passed.
- `cargo nextest run -p taru-db artwork --no-fail-fast` passed: 3 tests.
- `cargo nextest run -p taru-server artwork --no-fail-fast` passed: 3 tests.
- `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
  passed: 10 tests.

2026-05-19, MAIS-040 closeout review and split:

- MAIS-030 was committed as
  `de72467 feat(artwork): queue managed candidate ingest`.
- Review result:
  - Workstream compliance has no blocking findings. The shipped behavior
    matches the selected first target: Admin candidate acceptance into internal
    managed artwork ingest state.
  - Code-quality review has no blocking findings. The service validates the
    candidate, target item, and library state before delegating the atomic
    candidate/ingest/job commit to the repository boundary.
  - Redaction review has no blocking findings. The Admin response and durable
    job input expose Taru IDs, image kind, status, and job lifecycle only; they
    do not expose candidate `source_uri`, Source Locators, filesystem paths,
    remote storage handles, cache URIs, or unvalidated addon hotlinks.
- Closeout decision:
  - Close this lane after proving the queued candidate-ingest boundary.
  - Split remote fetch/content validation and managed artifact byte storage to
    the next follow-on.
  - Split public image-serving/redacted references, thumbnails, and selected
    artwork publication until a managed artifact exists.
  - Keep artwork sidecar export in `addon-library-file-write-policy`.
- Final closeout gates after MAIS-040 documentation edits:
  - `Get-Content -Raw docs/workstreams/managed-artwork-ingest-selection/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
