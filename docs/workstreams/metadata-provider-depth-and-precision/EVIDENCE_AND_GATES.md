# Metadata Provider Depth And Precision — Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/metadata-provider-depth-and-precision/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/metadata-provider-depth-and-precision/TASKS.jsonl",
    "docs/workstreams/metadata-provider-depth-and-precision/CAMPAIGNS.jsonl",
    "docs/workstreams/metadata-provider-depth-and-precision/CONTEXT.jsonl",
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

- `cargo nextest run -p nako-metadata tmdb_provider metadata_candidate --no-fail-fast`
- `cargo nextest run -p nako-metadata matching refresh --no-fail-fast` when refresh seams change
- focused `cargo nextest run -p nako-metadata ...` for provider capability or matching changes
- focused `cargo nextest run -p nako-api ...` when diagnostics DTOs change
- `cargo check -p nako-server --tests` when API/server integration changes
- `cargo fmt --all -- --check` when Rust changes
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/metadata-provider-breadth/DESIGN.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-apply-repair-actions/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `crates/nako-metadata/src/types.rs`
- `crates/nako-metadata/src/matching.rs`
- `crates/nako-metadata/src/provider_attempt.rs`

## Opening Recon

Local planning recon on 2026-06-02:

- `MetadataProviderCapabilities` already exists and is exposed through
  provider diagnostics.
- `MetadataCandidateMatchingPolicy` already classifies accepted,
  needs-confirmation, and rejected candidates.
- `provider_attempt::resolve_provider_key` already keeps search ambiguity
  non-mutating by turning `needs_confirmation` into a no-match attempt.
- TMDB has explicit movie/series search and season/episode fetch support with
  provider-specific compound keys.
- TMDB currently advertises hierarchy support, but `MetadataCandidateGraph`
  construction only carries a root node; series fetch does not yet expose
  related season Provider Subjects.
- `refresh_existing_with_provider` consumes only `root_provider_subject()`,
  so child graph nodes can remain non-mutating preview evidence.
- Douban and Bangumi expose broader video support, but their adapter notes
  indicate movie/generic subject endpoint limits and should be audited before
  adding UI or mutation semantics.

## MPDP-020 Decision

The first executable slice is TMDB series -> season provider graph preview.

The implementation should:

- parse season summaries already present in TMDB TV details;
- add related season Provider Subjects and parent-child relationships to the
  candidate graph;
- use existing TMDB season compound key semantics;
- preserve root-only refresh and Provider Mapping persistence;
- avoid Media Item hierarchy creation, schema changes, Public Client API
  changes, Web confirmation UI, or Generated Artifact apply behavior.

## Notes

- Treat raw provider responses, provider headers, tokens, proxy URLs, local
  paths, and source locators as forbidden Admin/API data unless an existing raw
  diagnostics endpoint explicitly owns them.
- Do not reopen Generated Artifact apply repair from this lane.
