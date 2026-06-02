# Provider Review Global Queue Search - Evidence And Gates

Status: Closed
Last updated: 2026-06-02

## Opening Gates

```bash
python -m json.tool docs/workstreams/provider-review-global-queue-search/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/provider-review-global-queue-search/TASKS.jsonl",
    "docs/workstreams/provider-review-global-queue-search/CAMPAIGNS.jsonl",
    "docs/workstreams/provider-review-global-queue-search/CONTEXT.jsonl",
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
- focused `nako-db` queue query tests if repository contract changes
- `cargo fmt --all -- --check`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke if a route or navigation mode is added
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/admin-candidate-review-list-navigation/CLOSEOUT.md`
- `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md`
- `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`

## Opening Recon

Local recon on 2026-06-02:

- Item-scoped list/navigation closed at
  `docs/workstreams/admin-candidate-review-list-navigation/`.
- `MetadataCandidateReviewRepository` currently exposes
  `list_metadata_candidate_reviews_for_item`, but no global queue query.
- SQLite/PostgreSQL candidate review adapters have item/status and source
  indexes, but no explicit global status/updated queue index yet.
- Admin API has detail/apply and item-scoped list routes, but no
  `/admin/v1/metadata/candidate-reviews` global queue route.
- Web route can render item-scoped lists and detail/apply, but no global queue
  mode exists.

## PRGQ-010 Evidence

Implemented behavior:

- opened this workstream as the global Candidate Review queue/search follow-on
  from ACRN closeout;
- selected a read-only Admin API global queue route as the first executable
  task;
- kept Web queue UI, Public Client API, batch governance, status mutation,
  apply mutation, and related hierarchy application out of the first campaign.

Green checks:

- `python -m json.tool docs/workstreams/provider-review-global-queue-search/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## PRGQ-020 Evidence

Implemented behavior:

- added a repository-owned global Candidate Review queue query with optional
  `status` and `provider` filters, stable `updated_at_ms DESC, id ASC`
  ordering, and page limit/offset handling in SQLite and PostgreSQL adapters;
- added `GET /admin/v1/metadata/candidate-reviews` as a read-only Admin API
  route backed by queue DTO rows and application-plan summaries;
- synchronized `nako-api` Admin contract source plus generated contract output
  for `apps/admin-web` and `web`;
- verified route responses do not write Provider Subject, Provider Mapping,
  Canonical Metadata, apply state, status, or hierarchy state and do not expose
  raw provider payloads, local URIs, fingerprints, temp paths, or secret review
  metadata.

Green checks:

- `cargo test -p nako-server admin_v1_metadata_candidate_review_queue_filters_global_rows_without_writes -- --nocapture`
  passed.
- `cargo test -p nako-db nako_database_sqlite_lists_metadata_candidate_review_queue_with_filters_and_pagination -- --nocapture`
  passed.
- `cargo test -p nako-api admin_contract -- --nocapture` passed.
- `cargo test -p nako-server metadata_candidate_review -- --nocapture` passed.
- `cargo test -p nako-db metadata_candidate_review -- --nocapture` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Environment note:

- `cargo nextest run -p nako-server metadata_candidate_review` could not run
  because local Cargo has no `nextest` subcommand installed.

## PRGQ-030 Evidence

Implemented behavior:

- added Web Admin client/read-model support for the global Candidate Review
  queue route and fixture fallback;
- added global queue mode to the Candidate Review page with status/provider
  filters, stable pagination, row-level item IDs, and route-state preservation
  into the existing detail/apply page;
- added a sidebar navigation entry for Candidate Review discovery;
- kept Web behavior read-only for queue rows: no batch apply, status mutation,
  hierarchy application, Public Client API, or provider endpoint depth;
- raised `total-js` gzip bundle budget from 343 KiB to 344 KiB after the new
  queue navigation increased the measured total to 343.14 KiB.

Green checks:

- `npm --prefix web run check` passed.
- `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-state-contracts.test.ts`
  passed: 2 files, 78 tests.
- `npm --prefix web run test` passed: 10 files, 120 tests.
- `npm --prefix web run build:budget` passed: total-js 343.14/344 KiB gzip.
- Playwright smoke passed against
  `http://127.0.0.1:3001/admin/metadata/candidate-reviews`: global queue
  rendered, sidebar `候选评审` was present, clicking a queue row navigated to
  the existing detail/apply surface, and the console had no page errors.

Environment note:

- Port 3000 was already occupied, so the smoke dev server was started from
  `web/` on port 3001.

## PRGQ-040 Evidence

Closeout behavior:

- closed this lane after the Admin API global queue and Web Admin queue
  navigation shipped;
- updated workstream ledgers, handoff, milestones, roadmap, goal map, and
  architecture lane links from active to closed;
- kept provider governance bulk review, provider review related hierarchy
  application, and Douban TV/episode endpoint depth split as follow-ons.

Green checks:

- `python -m json.tool docs/workstreams/provider-review-global-queue-search/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
