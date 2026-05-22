# 2026-05-20 — MAPG Closeout

## Summary

Closed Managed Artwork PostgreSQL parity.

The implementation adds PostgreSQL schema and repository parity for Addon
Artwork Candidates, legacy Artwork Tasks, Managed Artwork Ingest, Managed
Artwork Artifacts, Selected Artwork, gallery snapshots, lifecycle cleanup, and
runtime capability reporting.

## Code Changes

- Added PostgreSQL migration `0002_managed_artwork.sql`.
- Implemented PostgreSQL `ArtworkTaskRepository`,
  `ArtworkCandidateRepository`, and `ManagedArtworkRepository`.
- Added backend-neutral Managed Artwork contract tests that run for SQLite and
  ignored PostgreSQL contract backends.
- Enabled PostgreSQL `managed_artwork` capability and changed server startup
  gating to rely on backend capability rather than a PostgreSQL-specific block.

## Verification

- `cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast`
- `cargo nextest run -p nako-db postgres_managed_artwork_contract --run-ignored ignored-only --no-fail-fast`
- `cargo check -p nako-db --tests`
- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api managed_artwork --no-fail-fast`
- `cargo nextest run -p nako-server managed_artwork --no-fail-fast`
- `cargo nextest run -p nako-db managed_artwork --no-fail-fast`

PostgreSQL verification used an ephemeral local PostgreSQL 17 cluster under
`target/nako-pg-contract`; it was stopped and removed after the ignored
contract run.

## Residual Follow-ons

- Add permanent PostgreSQL CI service orchestration for ignored contract gates.
- Open a separate lane for image-processing or artifact-store policy changes if
  requirements expand beyond the existing local artifact-store model.
