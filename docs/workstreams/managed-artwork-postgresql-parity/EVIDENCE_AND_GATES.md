# Managed Artwork PostgreSQL Parity — Evidence And Gates

Status: Proposed
Last updated: 2026-05-20

## Baseline Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-db --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db artwork --no-fail-fast
cargo nextest run -p taru-server artwork --no-fail-fast
git diff --check
```

When PostgreSQL contracts are added:

```bash
TARU_TEST_POSTGRES_URL=<url> cargo nextest run -p taru-db managed_artwork --run-ignored ignored-only --no-fail-fast
```

## Redaction Inventory Gate

```bash
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|artifact_root|local_path|selected_artwork|managed_artwork" crates/taru-api crates/taru-server/src/http docs/api
```

The inventory must prove that public/Admin DTOs remain redacted before runtime
support is claimed.

## PGR-090 Split Evidence

M62 split Managed Artwork parity because the subsystem spans Addon Artwork
Candidates, Managed Artwork Ingest, artifacts, Selected Artwork, galleries,
lifecycle cleanup, drift diagnostics, remediation, thumbnails, artifact-store
files, and redaction-sensitive public/Admin serving. Partial PostgreSQL support
would be worse than an explicit unsupported boundary.
