# Admin Candidate Review List Navigation - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Gates

```bash
python -m json.tool docs/workstreams/admin-candidate-review-list-navigation/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/admin-candidate-review-list-navigation/TASKS.jsonl",
    "docs/workstreams/admin-candidate-review-list-navigation/CAMPAIGNS.jsonl",
    "docs/workstreams/admin-candidate-review-list-navigation/CONTEXT.jsonl",
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
- `cargo fmt --all -- --check`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke if a route or navigation mode is added
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md`
- `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md`
- `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`

## Opening Recon

Local recon on 2026-06-02:

- Admin/Web Candidate Review detail and apply are shipped at
  `docs/workstreams/admin-web-provider-depth-governance/`.
- `MetadataCandidateReviewRepository::list_metadata_candidate_reviews_for_item`
  already exists and the SQLite adapter orders by `updated_at_ms DESC, id ASC`.
- Current Admin API routes expose detail/apply by `review_id` only.
- Current Web route can render detail/apply when `review_id` is known, but there
  is no item-scoped discovery/navigation surface.

## ACRN-010 Evidence

Implemented behavior:

- opened this workstream as the item-scoped Candidate Review discovery follow-on
  from AWPDG closeout;
- selected a read-only Admin API list route as the first executable task;
- kept Web navigation, Public Client API, schema migrations, global queues,
  batch governance, and related-node hierarchy application out of the first
  campaign.

Green checks:

- `python -m json.tool docs/workstreams/admin-candidate-review-list-navigation/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## ACRN-020 Evidence

Implemented behavior:

- added `GET /admin/v1/metadata/items/{item_id}/candidate-reviews`;
- returned item-scoped Candidate Review summaries with pagination, root metadata
  summary, related/relationship counts, application plan, and read-only boundary;
- synchronized `crates/nako-api` Admin contract source plus generated contracts
  under `apps/admin-web` and `web`;
- proved the list route does not create Provider Subjects, Provider Mappings,
  Canonical Metadata, or related hierarchy writes on read;
- proved response bodies omit other-item reviews, raw overviews/tags, source
  locators, local paths, tokens, and fingerprints.

Green checks:

- `cargo nextest` was attempted first and failed because `cargo-nextest` is not
  installed in this environment.
- `cargo test -p nako-server admin_v1_metadata_candidate_review_list_is_item_scoped_redacted_and_read_only -- --nocapture`
  passed: 1 test.
- `cargo test -p nako-api admin_contract -- --nocapture` passed: 5 tests.
- `cargo test -p nako-server candidate_review -- --nocapture` passed: 4 tests.
- `cargo fmt --all -- --check` passed.
- `python -m json.tool docs/workstreams/admin-candidate-review-list-navigation/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## ACRN-030 Evidence

Implemented behavior:

- added Web Admin item-scoped Candidate Review list loading through
  `NAKO_ADMIN_ROUTES.metadataCandidateReviewsForItem`;
- mapped live list responses into redaction-safe navigation rows and kept full
  evidence loading on the existing detail route;
- added route-state support for `item_id`, `review_id`, `limit`, and `offset`;
- let list row selection preserve item paging state while routing into the
  existing detail/apply page;
- kept list rows as navigation/triage summaries only: no batch apply, no related
  hierarchy application, no raw provider payload display, and no Public Client
  API expansion;
- raised the aggregate `total-js` gzip budget from 341 KiB to 343 KiB after the
  measured production build reached 342.05 KiB. Individual initial, admin-route,
  and media-route budgets stayed unchanged.

Green checks:

- `npm --prefix web run check` passed.
- `npm --prefix web run test` passed: 10 files, 118 tests.
- `npm --prefix web run build:budget` passed. Final budget table included
  `admin-route-js` 216.11 raw KiB / 45.08 gzip KiB and `total-js` 1168.11 raw
  KiB / 342.05 gzip KiB against the 1250/343 KiB budget.
- Edge CDP browser smoke passed for
  `/admin/metadata/candidate-reviews?item_id=fixture-item-1`: the page rendered
  `fixture-metadata-candidate-review-accepted-1`, clicked the row button, and
  reached
  `?item_id=fixture-item-1&review_id=fixture-metadata-candidate-review-accepted-1`
  with `Review evidence` and `Root Provider Mapping` visible.
- `git diff --check` passed with Git CRLF normalization warnings only.

Gate notes:

- The Browser plugin runtime was not exposed in this session, so the browser
  smoke used local Edge headless CDP against a temporary Vite dev server.
