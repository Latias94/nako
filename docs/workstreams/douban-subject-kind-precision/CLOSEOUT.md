# Douban Subject Kind Precision - Closeout

Status: Closed
Closed: 2026-06-02
Task: DSKP-030

## Final Status

Closed after `DSKP-010`, `DSKP-020`, and `DSKP-030`.

The lane made Douban provider capability claims truthful for the current
adapter. Douban search and fetch remain movie endpoint-backed, and Series,
Season, and Episode requests now fail before provider HTTP calls.

No schema migration, Public Client API change, Admin/Web confirmation UI,
Generated Artifact apply change, hierarchy graph preview, automatic Media Item
creation, or child Provider Mapping write shipped in this lane.

## Shipped

- Narrowed Douban provider capabilities to `Movie` and `Unknown` media kinds.
- Narrowed Douban Provider Subject capabilities to `Movie` and `Subject`.
- Explicit `Unsupported` responses for Series, Season, and Episode search and
  fetch before endpoint-backed support exists.
- Regression coverage proving unsupported requests do not reach Douban movie
  endpoints.
- Existing Douban movie search/fetch, metadata mapping, and candidate graph
  behavior remained compatible.

## Gates

Fresh verification for the shipped behavior:

```bash
cargo nextest run -p nako-metadata douban_provider built_in_provider_capabilities --no-fail-fast
cargo nextest run -p nako-metadata douban_provider metadata_candidate --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/douban-subject-kind-precision/WORKSTREAM.json
JSONL validation for TASKS.jsonl, CAMPAIGNS.jsonl, and CONTEXT.jsonl
git diff --check
```

Results:

- capability and unsupported-kind guard tests passed;
- Douban movie search/fetch mapping tests passed;
- candidate graph raw-payload projection guard still passed;
- workstream JSON and JSONL validation passed;
- `cargo fmt --all -- --check` passed;
- `git diff --check` passed with Windows line-ending warnings only.

## Follow-Ons

- `proposed:douban-tv-episode-endpoint-depth`
- `proposed:metadata-candidate-durable-review`
- `proposed:admin-web-provider-depth-governance`

## Residual Risks

- Douban Series, Season, and Episode support remains unsupported until a future
  endpoint-backed lane proves source contract and mapping semantics.
- Candidate graph previews are not durable review records.
- Admin/Web does not yet render provider capability governance or confirm
  provider graph depth evidence.
- Provider graph evidence remains preview-only until a future workstream
  deliberately changes persistence semantics.
