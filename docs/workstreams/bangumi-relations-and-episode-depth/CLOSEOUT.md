# Bangumi Relations And Episode Depth - Closeout

Status: Closed
Closed: 2026-06-02
Task: BRED-050

## Final Status

Closed after `BRED-020`, `BRED-030`, and `BRED-040`.

The lane made Bangumi provider depth truthful and endpoint-backed. Bangumi no
longer advertises Season/Episode direct support before dedicated endpoints
exist, while Series fetches now expose episode preview evidence from
`/v0/episodes`.

Refresh and Provider Mapping persistence remain root-only. No schema
migration, Public Client API change, Admin/Web confirmation UI, Generated
Artifact apply change, automatic episode Media Item creation, or child Provider
Mapping write shipped in this lane.

## Shipped

- Narrowed Bangumi provider capabilities to endpoint-backed subject-level
  `Movie`, `Series`, and `Unknown` behavior.
- Explicit `Unsupported` responses for Season/Episode search and fetch before
  direct endpoint-backed support exists.
- Bangumi `/v0/episodes` parsing for Series fetches.
- Bangumi Series -> Episode Candidate Graph related nodes and `contains`
  relationships.
- Episode summary mapping for title, original title, airdate, runtime minutes,
  overview, and Bangumi external ID.
- A refresh guard proving episode preview nodes do not create episode Media
  Items, episode Provider Subjects, or child Provider Mappings.

## Gates

Fresh verification for the shipped behavior:

```bash
cargo nextest run -p nako-metadata bangumi_provider built_in_provider_capabilities --no-fail-fast
cargo nextest run -p nako-metadata bangumi_provider_uses_runtime_and_maps_http_response --no-fail-fast
cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast
cargo nextest run -p nako-metadata refresh_persists_only_root_provider_mapping_from_bangumi_episode_graph_preview --no-fail-fast
cargo nextest run -p nako-metadata bangumi refresh metadata_candidate --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/bangumi-relations-and-episode-depth/WORKSTREAM.json
JSONL validation for TASKS.jsonl, CAMPAIGNS.jsonl, and CONTEXT.jsonl
git diff --check
```

Results:

- capability and unsupported-kind guard tests passed;
- Bangumi endpoint-backed episode graph preview tests passed;
- candidate graph and refresh guard tests passed;
- refresh persists only the root Bangumi Provider Subject and accepted root
  Provider Mapping;
- related episode preview nodes remain non-mutating;
- workstream JSON and JSONL validation passed;
- `cargo fmt --all -- --check` passed;
- `git diff --check` passed with Windows line-ending warnings only.

## Follow-Ons

- `proposed:douban-subject-kind-precision`
- `proposed:metadata-candidate-durable-review`
- `proposed:admin-web-provider-depth-governance`

## Residual Risks

- Bangumi direct Episode fetch remains unsupported; the shipped work exposes
  episode graph preview under Series fetch only.
- Candidate graph previews are not durable review records.
- Admin/Web does not yet render or confirm provider graph depth evidence.
- Douban subject-kind precision remains intentionally deferred to a focused
  provider lane.
- Provider graph evidence remains preview-only until a future workstream
  deliberately changes persistence semantics.
