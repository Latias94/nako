# Admin Web Provider Depth Governance - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Gates

```bash
python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/admin-web-provider-depth-governance/TASKS.jsonl",
    "docs/workstreams/admin-web-provider-depth-governance/CAMPAIGNS.jsonl",
    "docs/workstreams/admin-web-provider-depth-governance/CONTEXT.jsonl",
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

- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
- `cargo fmt --all -- --check`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke if a route is added
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md`
- `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md`
- `docs/workstreams/admin-catalog-governance-read-model/README.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`

## Opening Recon

Local recon on 2026-06-02:

- Backend accepted-review application exists in
  `MetadataCandidateReviewApplicationService`.
- Existing Admin Catalog Governance Provider Mapping review routes mutate
  already persisted Provider Mapping rows; they do not expose durable Candidate
  Review snapshots or application plans.
- Metadata diagnostics Candidate Review DTOs expose matching/review facts but
  are not the durable application surface.
- Generated Artifact Provider Mapping apply proves plan/apply/result patterns,
  but its outcome tables remain out of scope for Candidate Review state.

## AWPDG-010 Evidence

Implemented behavior:

- opened this workstream as the Admin/Web provider depth governance follow-on
  from ARPMA closeout;
- selected a read-only Admin API Candidate Review detail/application-plan slice
  as the first executable task;
- kept apply mutation, Web, Public Client API, and related-node hierarchy
  application out of the first campaign.

Green checks:

- `python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
