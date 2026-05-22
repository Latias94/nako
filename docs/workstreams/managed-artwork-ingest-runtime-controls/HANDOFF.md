# Managed Artwork Ingest Runtime Controls Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is closed. Runtime code now exposes:

- `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`

The command transactionally resets failed Managed Artwork ingests and failed
durable managed-artwork jobs to queued, clears failure state, is idempotent for
already queued ingests, rejects stored/running/fetching/validating states, and
returns a redacted response. Requeue itself does not fetch, validate, write
artifacts, publish, cleanup, repair, or delete files. The server regression
also proves `process-next` can later retry the same accepted candidate and store
an artifact after the source becomes valid.

## Files Changed

- `crates/nako-core/src/media/artwork.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-db/src/artwork.rs`
- `crates/nako-db/src/lib.rs`
- `crates/nako-db/src/tests.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/artwork.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `crates/nako-server/src/http/tests/mod.rs`
- `docs/api/HTTP_API.md`

## Validation

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
cargo nextest run -p nako-api managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p nako-db managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p nako-server managed_artwork_ingest_requeue --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

## Follow-Ons Outside This Lane

- Active in-process Managed Artwork ingest cancellation.
- Automatic retry scheduling or background worker orchestration.
- Missing-artifact repair or re-ingest.
- Public Client candidate/gallery browsing.
- Artifact cleanup/deletion policy changes.
