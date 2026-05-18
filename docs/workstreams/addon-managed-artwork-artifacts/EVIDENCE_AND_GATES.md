# Addon Managed Artwork Artifacts Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

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
