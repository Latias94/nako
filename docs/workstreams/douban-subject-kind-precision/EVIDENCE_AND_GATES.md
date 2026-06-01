# Douban Subject Kind Precision - Evidence And Gates

Status: Closed
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/douban-subject-kind-precision/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/douban-subject-kind-precision/TASKS.jsonl",
    "docs/workstreams/douban-subject-kind-precision/CAMPAIGNS.jsonl",
    "docs/workstreams/douban-subject-kind-precision/CONTEXT.jsonl",
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

- `cargo nextest run -p nako-metadata douban_provider built_in_provider_capabilities --no-fail-fast`
- `cargo nextest run -p nako-metadata douban_provider metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md`
- `docs/workstreams/bangumi-relations-and-episode-depth/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-metadata/src/providers/douban.rs`
- `crates/nako-metadata/src/mapping/douban.rs`
- `crates/nako-metadata/src/tests.rs`

## Opening Recon

Local recon on 2026-06-02:

- Existing Douban search uses `movie/search`.
- Existing Douban fetch uses `movie/subject/{id}`.
- Current capabilities include `Series`, `Season`, and `Episode` even though
  the adapter does not call non-movie endpoints.
- Existing refresh coverage proves Douban movie Provider Subject and Provider
  Mapping persistence.
- The first implementation slice should remove false capability claims before
  any future Douban TV or episode endpoint work.

## DSKP-020 Evidence

Red checks:

- `cargo nextest run -p nako-metadata built_in_provider_capabilities_are_diagnostics_safe --no-fail-fast`
  failed before implementation because Douban still advertised
  `MediaKind::Series`.
- `cargo nextest run -p nako-metadata douban_provider_rejects_series_season_episode_until_endpoint_backed --no-fail-fast`
  failed before implementation because a Series search reached `/movie/search`
  and returned a candidate graph.

Implemented behavior:

- Douban capabilities now advertise only `Movie` and `Unknown` media kinds and
  only `Movie` and `Subject` Provider Subject kinds.
- Douban search/fetch rejects `Series`, `Season`, and `Episode` before HTTP.
- Unsupported Douban subject kinds no longer map to Series/Season/Episode
  Provider Subject kinds through the movie endpoint.

Green checks:

- `cargo nextest run -p nako-metadata built_in_provider_capabilities_are_diagnostics_safe --no-fail-fast`
- `cargo nextest run -p nako-metadata douban_provider_rejects_series_season_episode_until_endpoint_backed --no-fail-fast`
- `cargo nextest run -p nako-metadata douban_provider built_in_provider_capabilities --no-fail-fast`
- `cargo nextest run -p nako-metadata douban_provider metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## DSKP-030 Closeout Evidence

Closeout checks:

- `cargo nextest run -p nako-metadata douban_provider built_in_provider_capabilities --no-fail-fast`
- `cargo nextest run -p nako-metadata douban_provider metadata_candidate --no-fail-fast`
- `cargo fmt --all -- --check`
- `python -m json.tool docs/workstreams/douban-subject-kind-precision/WORKSTREAM.json`
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
- `git diff --check`

Closeout result:

- Douban precision target state is complete.
- Future Douban TV/episode endpoint depth is split to
  `proposed:douban-tv-episode-endpoint-depth`.
- Durable candidate review and Admin/Web provider governance remain separate
  follow-ons.

## Notes

- Do not change persistence semantics from this lane.
- Do not expose raw provider payloads or secrets.
