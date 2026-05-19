# Managed Artwork Ingest Runtime Controls Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open. No runtime code has been changed in this lane yet.

The lane owns Admin runtime controls for Managed Artwork ingest after accepted
Artwork Candidates can become queued ingest jobs and `process-next` can store or
fail them.

## First Executable Task

Start with `MAIRC-020`:

```text
POST /admin/v1/artwork/ingests/{ingest_id}/requeue
```

Implementation expectations:

- only failed ingests with failed `managed_artwork_ingest` jobs are requeued;
- already queued ingests return `requeued = false`;
- stored/running/fetching/validating states return conflict;
- requeue is transactional across `managed_artwork_ingests` and `jobs`;
- response is redacted;
- requeue itself does not fetch, validate, write artifacts, publish, cleanup, or
  delete files.

## Files To Inspect Next

- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-db/src/jobs.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/tests/addons.rs`
- `docs/api/HTTP_API.md`

## Suggested Validation

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-api managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p taru-db managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p taru-server managed_artwork_ingest_requeue --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

## Follow-Ons Outside This Lane

- Active in-process Managed Artwork ingest cancellation.
- Automatic retry scheduling or background worker orchestration.
- Missing-artifact repair or re-ingest.
- Public Client candidate/gallery browsing.
- Artifact cleanup/deletion policy changes.
