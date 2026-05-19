# Managed Artwork Ingest Runtime Controls Evidence And Gates

Status: Completed
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
| 2026-05-19 | `WORKSTREAM.json` parse | Pass | `Get-Content ... WORKSTREAM.json \| ConvertFrom-Json` succeeded after opening docs. |
| 2026-05-19 | `git diff --check` | Pass | Opening-doc diff was whitespace-clean. |
| 2026-05-19 | `cargo nextest run -p taru-api managed_artwork_ingest_requeue --no-fail-fast` | Pass | Redacted requeue DTO does not serialize raw job input, summary, error, source URI, or token values. |
| 2026-05-19 | `cargo nextest run -p taru-db managed_artwork_ingest_requeue --no-fail-fast` | Pass | Failed ingest/job requeue to queued, queued replay is idempotent, retry claim works, running/stored states conflict. |
| 2026-05-19 | `cargo nextest run -p taru-server managed_artwork_ingest_requeue --no-fail-fast` | Pass | HTTP route requeues failed ingest, redacts response, supports queued replay, and `process-next` later stores artifact after source becomes valid. |
| 2026-05-19 | `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests` | Pass | Cross-crate trait/API changes compile across core, db, api, and server tests. |
| 2026-05-19 | `cargo fmt --all -- --check` | Pass | Formatting clean after applying `cargo fmt --all` to this lane's Rust edits. |
| 2026-05-19 | `git diff --check` | Pass | Diff has no whitespace errors; Git reports only line-ending normalization warnings. |
| 2026-05-19 | Redaction inventory | Pass | Hits are expected implementation internals, docs policy text, and tests asserting forbidden values are absent from responses. |

## Review Checklist

- Requeue is keyed by ingest ID, not raw candidate source URL.
- Requeue does not fetch, validate, store, publish, cleanup, or delete.
- Failed job and ingest state are reset atomically.
- Already queued replay is idempotent.
- Stored/running/fetching/validating states are rejected.
- Responses do not expose raw candidate source, addon payload/provenance,
  provider query strings, storage handles, local paths, cache URIs, raw
  validation errors, or content hashes.
