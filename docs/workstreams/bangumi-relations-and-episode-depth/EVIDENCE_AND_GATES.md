# Bangumi Relations And Episode Depth - Evidence And Gates

Status: Active
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

## Notes

- Do not change persistence semantics from this lane.
- Do not expose raw provider payloads or secrets.
