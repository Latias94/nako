# Bangumi Relations And Episode Depth - Evidence And Gates

Status: Closed
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/bangumi-relations-and-episode-depth/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/bangumi-relations-and-episode-depth/TASKS.jsonl",
    "docs/workstreams/bangumi-relations-and-episode-depth/CAMPAIGNS.jsonl",
    "docs/workstreams/bangumi-relations-and-episode-depth/CONTEXT.jsonl",
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

- `cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast`
- `cargo nextest run -p nako-metadata bangumi refresh metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md`
- `docs/workstreams/tmdb-season-episode-graph-depth/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-metadata/src/providers/bangumi.rs`
- `crates/nako-metadata/src/mapping/bangumi.rs`
- `crates/nako-metadata/src/tests.rs`

## Opening Recon

Local and official-spec recon on 2026-06-02:

- Existing Bangumi search uses `/v0/search/subjects`.
- Existing Bangumi fetch uses `/v0/subjects/{id}` and returns a root-only
  Candidate Graph.
- Current capabilities include `Season` and `Episode` even though the adapter
  does not call Bangumi episode endpoints.
- Official Bangumi OpenAPI at
  `https://raw.githubusercontent.com/bangumi/api/master/open-api/v0.yaml`
  includes `/v0/subjects/{subject_id}/subjects`, `/v0/episodes`, and
  `/v0/episodes/{episode_id}` endpoint entries.
- The first implementation slice should remove false capability claims before
  adding relation or episode graph depth.

## BRED-020 Evidence

Completed on 2026-06-02.

Implementation:

- Bangumi `MetadataProviderCapabilities` now advertises only endpoint-backed
  subject-level `Movie`, `Series`, and `Unknown` media kinds.
- Bangumi supported Provider Subject kinds no longer include `Season` or
  `Episode`.
- Bangumi `search` and `fetch` reject Season/Episode requests before provider
  HTTP calls until dedicated relation or episode endpoints are implemented.
- Existing Bangumi subject search/fetch and refresh mapping behavior remains
  compatible.

Validation:

```bash
cargo nextest run -p nako-metadata bangumi_provider built_in_provider_capabilities --no-fail-fast
```

Result: initially failed as expected before implementation, then passed with 4
tests after capability narrowing.

```bash
cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast
```

Result: passed, 4 tests.

```bash
cargo fmt --all -- --check
```

Result: passed.

## BRED-030 Evidence

Completed on 2026-06-02.

Implementation:

- Bangumi `fetch(Series)` now reads `/v0/episodes` with `subject_id`, `type=0`,
  and bounded pagination.
- Main episode summaries are projected into related Candidate Graph nodes with
  Bangumi Episode Provider Subjects.
- The graph records `contains` relationships from the Bangumi series Provider
  Subject to each related episode Provider Subject.
- Bangumi raw fetch output records both subject and episode endpoint payloads
  for the root fetch result.
- Direct Episode search/fetch capability remains unsupported until a dedicated
  endpoint-backed task implements it.

Validation:

```bash
cargo nextest run -p nako-metadata bangumi_provider_uses_runtime_and_maps_http_response --no-fail-fast
```

Result: initially failed as expected before implementation, then passed with 1
test after episode graph preview.

```bash
cargo nextest run -p nako-metadata bangumi_provider metadata_candidate --no-fail-fast
```

Result: passed, 4 tests.

```bash
cargo fmt --all -- --check
```

Result: passed.

## BRED-040 Evidence

Completed on 2026-06-02.

Implementation:

- Added a Bangumi series refresh guard that feeds a root series graph with a
  related episode preview node into `MetadataRefreshService`.
- Refresh persists only the root Bangumi Series Provider Subject and accepted
  root Provider Mapping.
- The related Bangumi Episode preview node remains evidence only: no episode
  Media Item, episode Provider Subject, or child Provider Mapping is created.
- Raw provider response caching still records the root series payload that
  includes episode preview evidence.

Validation:

```bash
cargo nextest run -p nako-metadata refresh_persists_only_root_provider_mapping_from_bangumi_episode_graph_preview --no-fail-fast
```

Result: passed, 1 test.

```bash
cargo nextest run -p nako-metadata bangumi refresh metadata_candidate --no-fail-fast
```

Result: passed, 16 tests.

```bash
cargo fmt --all -- --check
```

Result: passed.

## Notes

- Do not change persistence semantics from this lane.
- Do not expose raw provider payloads or secrets.
- Lane closed at `BRED-050`; see `CLOSEOUT.md`.
