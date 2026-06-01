# TMDB Season Episode Graph Depth — Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/tmdb-season-episode-graph-depth/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/tmdb-season-episode-graph-depth/TASKS.jsonl",
    "docs/workstreams/tmdb-season-episode-graph-depth/CAMPAIGNS.jsonl",
    "docs/workstreams/tmdb-season-episode-graph-depth/CONTEXT.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check
```

## Expected Gates

- `cargo nextest run -p nako-metadata tmdb_provider_supports_series_season_and_episode_fetches --no-fail-fast`
- `cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast`
- `cargo nextest run -p nako-metadata refresh season metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-provider-depth-and-precision/CLOSEOUT.md`
- `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-metadata/src/providers/tmdb.rs`
- `crates/nako-metadata/src/mapping/tmdb.rs`
- `crates/nako-metadata/src/tests.rs`

## Opening Recon

Local planning recon on 2026-06-02:

- TMDB season fetch already uses compound keys `{series_id}/{season_number}`.
- TMDB episode fetch already uses compound keys
  `{series_id}/{season_number}/{episode_number}`.
- `TmdbSeasonDetails` currently parses season root facts but not episode
  summary arrays.
- `tmdb_episode_details_to_metadata` already maps episode detail facts and can
  guide episode summary metadata shape.
- Prior MPDP refresh guard proves related graph nodes remain non-mutating.

## TSEG-020 Evidence

Completed on 2026-06-02.

Implementation:

- `TmdbSeasonDetails` now parses TMDB `episodes` summaries.
- `TmdbMetadataProvider::fetch(Season)` projects each episode summary into a
  related `MetadataCandidateNode`.
- The graph records a `contains` relationship from the TMDB season Provider
  Subject to each TMDB episode Provider Subject.
- Episode subject keys use the existing
  `{series_id}/{season_number}/{episode_number}` compound key. Episode TMDB IDs
  remain metadata external IDs.
- No schema, API, Web, Generated Artifact apply, Media Item hierarchy, or
  Provider Mapping behavior changed.

Validation:

```bash
cargo nextest run -p nako-metadata tmdb_provider_supports_series_season_and_episode_fetches --no-fail-fast
```

Result: passed, 1 test.

```bash
cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast
```

Result: passed, 3 tests.

```bash
cargo fmt --all -- --check
```

Result: passed.

## TSEG-030 Evidence

Completed on 2026-06-02.

Implementation:

- Added a season refresh guard that feeds a TMDB season root graph with a
  related episode preview node into `MetadataRefreshService`.
- The refresh persists only the root season Provider Subject and accepted root
  Provider Mapping.
- The related episode preview node remains evidence only: no episode Media
  Item, episode Provider Subject, or child Provider Mapping is created.
- Raw provider response caching still records the season payload under the root
  season provider key.

Validation:

```bash
cargo nextest run -p nako-metadata refresh_persists_only_root_provider_mapping_from_season_episode_graph_preview --no-fail-fast
```

Result: passed, 1 test.

```bash
cargo nextest run -p nako-metadata refresh season metadata_candidate --no-fail-fast
```

Result: passed, 12 tests.

```bash
cargo fmt --all -- --check
```

Result: passed.

## Notes

- Do not change persistence semantics from this lane.
- Do not expose raw provider payloads or secrets.
