# Metadata Provider Depth And Precision — Closeout

Status: Closed
Closed: 2026-06-02
Task: MPDP-050

## Final Status

Closed after `MPDP-020`, `MPDP-030`, and `MPDP-040`.

The lane proved the first safe metadata provider depth slice: TMDB series fetch
can expose season Provider Subjects as candidate graph preview evidence, while
refresh and Provider Mapping persistence remain root-only.

No schema migration, Public Client API change, Admin/Web confirmation UI,
Generated Artifact apply change, automatic Media Item hierarchy creation, or
child Provider Mapping write shipped in this lane.

## Shipped

- TMDB series detail parsing for `seasons` summaries.
- TMDB series -> season `MetadataCandidateGraph` related nodes and `contains`
  relationships.
- A refresh guard proving graph preview nodes do not create child Media Items,
  child Provider Subjects, or child Provider Mappings.
- Follow-on split for TMDB episode depth, Bangumi relations/episodes, Douban
  subject precision, durable candidate review, and Admin/Web provider depth
  governance.

## Gates

Fresh verification for the shipped behavior:

```bash
cargo nextest run -p nako-metadata tmdb_provider_supports_series_season_and_episode_fetches --no-fail-fast
cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast
cargo nextest run -p nako-metadata matching_policy candidate_conflict_review --no-fail-fast
cargo nextest run -p nako-metadata refresh_persists_only_root_provider_mapping_from_provider_graph_preview --no-fail-fast
cargo nextest run -p nako-metadata matching refresh metadata_candidate --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/metadata-provider-depth-and-precision/WORKSTREAM.json
JSONL validation for TASKS.jsonl, CAMPAIGNS.jsonl, and CONTEXT.jsonl
git diff --check
```

Results:

- TMDB provider graph preview tests passed;
- candidate graph, matching policy, and conflict review gates passed;
- refresh persistence guard passed, including root-only Provider Mapping and
  no season Provider Subject insertion;
- workstream JSON and JSONL validation passed;
- `cargo fmt --all -- --check` passed;
- `git diff --check` passed with Windows line-ending warnings only.

## Follow-Ons

- `proposed:tmdb-season-episode-graph-depth`
- `docs/workstreams/bangumi-relations-and-episode-depth/`
- `proposed:douban-subject-kind-precision`
- `docs/workstreams/metadata-candidate-durable-review/`
- `proposed:admin-web-provider-depth-governance`

See `FOLLOW_ONS.md` for scope and non-goals.

## Residual Risks

- TMDB episode depth is not yet represented as season -> episode graph
  preview.
- Bangumi and Douban capability precision is still intentionally deferred.
- Candidate graph previews are not durable review records.
- Admin/Web does not yet render or confirm provider depth evidence.
- Provider graph evidence remains preview-only until a future workstream
  deliberately changes persistence semantics.
