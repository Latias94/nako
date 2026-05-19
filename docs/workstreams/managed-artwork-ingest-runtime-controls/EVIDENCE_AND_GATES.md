# Managed Artwork Ingest Runtime Controls Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Required Gates

Run with a shared target dir when practical:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-api managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p taru-db managed_artwork_ingest_requeue --no-fail-fast
cargo nextest run -p taru-server managed_artwork_ingest_requeue --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

## Redaction Inventory

Before closeout, run:

```powershell
rg -n "requeue|retry|cancel|managed_artwork_ingest|source_uri|storage_uri|cache_uri|content_hash|managed-artwork://|payload_json|provenance_json|artifact_root|local_path" crates/taru-api crates/taru-server/src/http crates/taru-server/src/app/artwork.rs docs/api
```

Expected result:

- requeue route/docs/tests are present;
- cancellation appears only as explicit follow-on wording unless implemented;
- forbidden source/storage/cache/path/payload/hash values are not serialized in
  Admin responses;
- tests may contain forbidden terms only as redaction assertions or internal
  setup.

## Evidence Log

| Date | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-19 | `WORKSTREAM.json` parse | Pending | Run after opening docs. |
| 2026-05-19 | `git diff --check` | Pending | Run after opening docs. |

## Review Checklist

- Requeue is keyed by ingest ID, not raw candidate source URL.
- Requeue does not fetch, validate, store, publish, cleanup, or delete.
- Failed job and ingest state are reset atomically.
- Already queued replay is idempotent.
- Stored/running/fetching/validating states are rejected.
- Responses do not expose raw candidate source, addon payload/provenance,
  provider query strings, storage handles, local paths, cache URIs, raw
  validation errors, or content hashes.
