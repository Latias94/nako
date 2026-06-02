# Provider Review Global Queue Search - Closeout

Status: Closed
Closed: 2026-06-02

## Shipped Scope

This lane added a global Admin-only Metadata Candidate Review queue/filter
surface for operators who are triaging durable review work across items.

Shipped behavior:

- `PRGQ-020` added
  `GET /admin/v1/metadata/candidate-reviews` as a read-only, paginated,
  redaction-safe Admin API queue route.
- The repository contract now exposes global Candidate Review queue reads with
  optional `status` and `provider` filters and deterministic
  `updated_at_ms DESC, id ASC` pagination across SQLite and PostgreSQL
  adapters.
- Admin queue rows expose review identity, item ID, status, source, root
  metadata summary, related evidence counts, application action, and page facts
  without exposing raw provider payloads or local/source secrets.
- `PRGQ-030` added Web Admin global queue navigation with status/provider
  filters, route-state pagination, row-level item IDs, and navigation into the
  existing Candidate Review detail/apply page.
- `PRGQ-040` closed the lane and left bulk governance, related hierarchy
  application, and provider endpoint depth as separate follow-ons.

## Confirmed Boundaries

- No Public Client API route or protocol DTO was added.
- No schema migration was required.
- No Candidate Review status mutation, batch accept/reject/apply, or bulk
  governance was added.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation was added.
- No raw provider payload, description body, tag body, image URL, local path,
  token, header, proxy URL, source fingerprint, temp path, or raw idempotency
  key is rendered through queue rows.
- Queue rows remain discovery/triage summaries; full evidence and explicit
  apply stay on the existing detail/apply route.

## Validation

Fresh implementation gates from `PRGQ-020`:

- `cargo test -p nako-server admin_v1_metadata_candidate_review_queue_filters_global_rows_without_writes -- --nocapture`
  passed.
- `cargo test -p nako-db nako_database_sqlite_lists_metadata_candidate_review_queue_with_filters_and_pagination -- --nocapture`
  passed.
- `cargo test -p nako-api admin_contract -- --nocapture` passed.
- `cargo test -p nako-server metadata_candidate_review -- --nocapture` passed.
- `cargo test -p nako-db metadata_candidate_review -- --nocapture` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `cargo nextest` was attempted and could not run because local Cargo has no
  `nextest` subcommand installed.

Fresh implementation gates from `PRGQ-030`:

- `npm --prefix web run check` passed.
- `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-state-contracts.test.ts`
  passed: 2 files, 78 tests.
- `npm --prefix web run test` passed: 10 files, 120 tests.
- `npm --prefix web run build:budget` passed with `total-js` 343.14 KiB gzip
  against the 344 KiB budget.
- Playwright smoke passed against
  `http://127.0.0.1:3001/admin/metadata/candidate-reviews`: global queue
  rendered, sidebar navigation exposed `候选评审`, row click preserved queue
  route state, and the existing detail/apply surface rendered with no page
  console errors.

Fresh closeout gates from `PRGQ-040`:

- `python -m json.tool docs/workstreams/provider-review-global-queue-search/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## Follow-Ons

- `proposed:provider-governance-bulk-review`: batch accept/reject/apply
  governance only after single-review stale guard and idempotency semantics
  remain the authority.
- `proposed:provider-review-related-hierarchy-application`: apply related graph
  nodes, child Provider Subjects, child Provider Mappings, or Media Item
  hierarchy changes only through a new policy and operator workflow.
- `proposed:douban-tv-episode-endpoint-depth`: prove Douban TV/episode endpoint
  semantics before broadening Douban provider capabilities.

## Residual Risks

- The route supports status/provider filtering and redaction-safe queue rows,
  but not full-text search across review IDs, item IDs, source keys, or root
  summaries yet. Add a search projection/index before claiming broad search.
- Bulk governance remains intentionally absent; operators still apply accepted
  reviews one at a time through the existing detail/apply route.
- Related graph nodes remain preview-only. That protects Media Item identity,
  but accepted season/episode hierarchy still needs a dedicated governance lane.
