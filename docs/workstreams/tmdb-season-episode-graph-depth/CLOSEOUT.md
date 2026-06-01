# TMDB Season Episode Graph Depth — Closeout

Status: Closed
Closed: 2026-06-02
Task: TSEG-040

## Final Status

Closed after `TSEG-020`, `TSEG-030`, and `TSEG-040`.

The lane extended TMDB provider graph preview from series -> season into
season -> episode depth. TMDB season fetches now expose related episode
Provider Subjects and `contains` relationships as candidate graph evidence,
while refresh and Provider Mapping persistence remain root-only.

No schema migration, Public Client API change, Admin/Web confirmation UI,
Generated Artifact apply change, automatic episode Media Item creation, or
child Provider Mapping write shipped in this lane.

## Shipped

- TMDB season detail parsing for endpoint-backed `episodes` summaries.
- TMDB season -> episode `MetadataCandidateGraph` related nodes and
  `contains` relationships.
- Existing TMDB episode compound key semantics:
  `{series_id}/{season_number}/{episode_number}`.
- Provider fetch tests proving the season graph exposes episode nodes without
  changing root season metadata behavior.
- A season refresh guard proving episode preview nodes do not create episode
  Media Items, episode Provider Subjects, or child Provider Mappings.

## Gates

Fresh verification for the shipped behavior:

```bash
cargo nextest run -p nako-metadata tmdb_provider_supports_series_season_and_episode_fetches --no-fail-fast
cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast
cargo nextest run -p nako-metadata refresh_persists_only_root_provider_mapping_from_season_episode_graph_preview --no-fail-fast
cargo nextest run -p nako-metadata refresh season metadata_candidate --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/tmdb-season-episode-graph-depth/WORKSTREAM.json
JSONL validation for TASKS.jsonl, CAMPAIGNS.jsonl, and CONTEXT.jsonl
git diff --check
```

Results:

- TMDB provider graph preview tests passed;
- candidate graph and refresh guard tests passed;
- season refresh persists only the root season Provider Subject and accepted
  root Provider Mapping;
- related episode preview nodes remain non-mutating;
- workstream JSON and JSONL validation passed;
- `cargo fmt --all -- --check` passed;
- `git diff --check` passed with Windows line-ending warnings only.

## Follow-Ons

- `docs/workstreams/bangumi-relations-and-episode-depth/`
- `proposed:douban-subject-kind-precision`
- `docs/workstreams/metadata-candidate-durable-review/`
- `proposed:admin-web-provider-depth-governance`

## Residual Risks

- Candidate graph previews are not durable review records.
- Admin/Web does not yet render or confirm provider graph depth evidence.
- Bangumi relation/episode depth and Douban subject-kind precision remain
  intentionally deferred to focused provider lanes.
- Provider graph evidence remains preview-only until a future workstream
  deliberately changes persistence semantics.
