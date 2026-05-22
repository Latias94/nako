# Selected Artwork Unpublish Delete Policy Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Required Gates

Run with a repo-local or shared target dir when practical:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
cargo nextest run -p nako-api selected_artwork_unpublish --no-fail-fast
cargo nextest run -p nako-db selected_artwork_unpublish --no-fail-fast
cargo nextest run -p nako-server selected_artwork_unpublish --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

If test names land under existing artwork/gallery tests instead of the
`selected_artwork_unpublish` filter, record the exact focused commands used.

## Redaction Inventory

Before closeout, run an inventory over touched public/admin paths:

```powershell
rg -n "unpublish|selection|selected_artwork|storage_uri|source_uri|cache_uri|content_hash|managed-artwork://|artifact_root|local_path" crates/nako-api crates/nako-server/src/http docs/api
```

Expected result:

- route and selected-artwork terms are present;
- forbidden storage/source/cache/hash/path values do not appear in DTO examples
  or response serializers;
- any internal-only occurrence is justified by handler internals or tests that
  assert redaction.

## Evidence Log

| Date | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-19 | `Get-Content docs\workstreams\selected-artwork-unpublish-delete-policy\WORKSTREAM.json \| ConvertFrom-Json \| Select-Object slug,status,current_task` | Pass | Opening docs parsed with `status=active`, `current_task=SAUD-020`. |
| 2026-05-19 | `cargo nextest run -p nako-api selected_artwork_unpublish --no-fail-fast` | Pass | Proves `UnpublishSelectedArtworkResponse` redacts storage URI, `managed-artwork://...`, source/cache URI fields, and content-hash values. |
| 2026-05-19 | `cargo nextest run -p nako-db selected_artwork_unpublish --no-fail-fast` | Pass | Proves repository unpublish removes Selected Artwork, keeps the Managed Artwork Artifact, makes lifecycle report it as cleanup-eligible, and is idempotent on replay. |
| 2026-05-19 | `cargo nextest run -p nako-server selected_artwork_unpublish --no-fail-fast` | Pass | Proves Admin DELETE route, redacted response, Public item image omission, old image `GET`/`HEAD` 404, artifact retention, lifecycle cleanup-candidate visibility, idempotent replay, and invalid-kind 400. |
| 2026-05-19 | `cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests` | Pass | Proves cross-crate trait, DTO, route, and test compilation. |
| 2026-05-19 | `cargo fmt --all -- --check` | Pass | Formatting gate passed after `cargo fmt --all`. |
| 2026-05-19 | `git diff --check` | Pass | No whitespace errors; Git emitted LF-to-CRLF normalization warnings only. |
| 2026-05-19 | `rg -n "unpublish\|selection\|selected_artwork\|storage_uri\|source_uri\|cache_uri\|content_hash\|managed-artwork://\|artifact_root\|local_path" crates/nako-api crates/nako-server/src/http docs/api` | Pass | Inventory shows route/docs/test/internal-redaction assertions only; no new public/Admin response serializer exposes forbidden locators, paths, raw URLs, cache handles, or content-hash values. |

## Review Checklist

- Unpublish is not implemented by artifact ID alone.
- Unpublish does not delete Managed Artwork Artifact rows.
- Unpublish does not remove local artifact files.
- Public image route does not fall back from Selected Artwork ID to artifact ID.
- Responses do not include `storage_uri`, `source_uri`, `cache_uri`,
  `managed-artwork://...`, local paths, or content hashes.
- HTTP docs clearly distinguish unpublish from cleanup.

Status: Completed.
