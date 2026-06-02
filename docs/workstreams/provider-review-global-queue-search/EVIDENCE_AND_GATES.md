# Provider Review Global Queue Search - Evidence And Gates

Status: Active
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
