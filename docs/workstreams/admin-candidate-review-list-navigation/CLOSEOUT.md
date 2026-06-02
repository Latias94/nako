# Admin Candidate Review List Navigation - Closeout

Status: Closed
Closed: 2026-06-02

## Shipped Scope

This lane made durable Metadata Candidate Reviews discoverable from an
item-scoped Admin/Web context and routed operators into the existing
Candidate Review detail/apply governance page.

Shipped behavior:

- `ACRN-020` added
  `GET /admin/v1/metadata/items/{item_id}/candidate-reviews` as a read-only,
  paginated, redaction-safe Admin API list route.
- The list route returns review identity, status, source, root metadata summary,
  related evidence counts, application action, and pagination without exposing
  raw provider payloads or secrets.
- `ACRN-030` added Web Admin item-scoped list/navigation with route-state
  support for `item_id`, `review_id`, `limit`, and `offset`.
- Web row selection preserves item paging state and routes into the existing
  Candidate Review detail/apply page instead of creating a second apply path.
- `ACRN-040` closed the lane and kept global queue/search, provider governance
  bulk review, and related hierarchy application as separate follow-ons.

## Confirmed Boundaries

- No Public Client API route or protocol DTO was added.
- No schema migration was required.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation was added.
- No batch accept/apply or provider governance bulk action was added.
- No raw provider payload, description body, tag body, image URL, local path,
  token, header, proxy URL, source fingerprint, or raw idempotency key is
  rendered through Admin/Web list rows.
- List rows remain navigation/triage summaries; full evidence remains on the
  existing detail/apply route.

## Validation

Fresh implementation gates from `ACRN-020`:

- `cargo nextest` was attempted first and failed because `cargo-nextest` is not
  installed in this environment.
- `cargo test -p nako-server admin_v1_metadata_candidate_review_list_is_item_scoped_redacted_and_read_only -- --nocapture`
  passed: 1 test.
- `cargo test -p nako-api admin_contract -- --nocapture` passed: 5 tests.
- `cargo test -p nako-server candidate_review -- --nocapture` passed: 4 tests.
- `cargo fmt --all -- --check` passed.
- JSON/JSONL validation passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Fresh implementation gates from `ACRN-030`:

- `npm --prefix web run check` passed.
- `npm --prefix web run test` passed: 10 files, 118 tests.
- `npm --prefix web run build:budget` passed with `admin-route-js` 216.11 raw
  KiB / 45.08 gzip KiB and `total-js` 1168.11 raw KiB / 342.05 gzip KiB
  against the 1250/343 KiB budget.
- Edge CDP browser smoke passed for item list to review detail navigation.
- `git diff --check` passed with Git CRLF normalization warnings only.

Fresh closeout gates from `ACRN-040`:

- `python -m json.tool docs/workstreams/admin-candidate-review-list-navigation/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## Follow-Ons

- `docs/workstreams/provider-review-global-queue-search/` (active): cross-item
  Candidate Review queue/search/filtering for operators who are not starting
  from a Media Item.
- `proposed:provider-governance-bulk-review`: batch accept/reject/apply
  governance only after single-review semantics remain stable.
- `proposed:provider-review-related-hierarchy-application`: apply related graph
  nodes, child Provider Subjects, child Provider Mappings, or Media Item
  hierarchy changes only through a new policy and operator workflow.
- `proposed:douban-tv-episode-endpoint-depth`: prove Douban TV/episode endpoint
  semantics before broadening Douban provider capabilities.

## Residual Risks

- Operators still do not have a cross-item Candidate Review queue. They must
  start from item context until a global queue/search follow-on is opened.
- Related graph nodes remain preview-only. That protects Media Item identity,
  but accepted season/episode hierarchy still needs a dedicated governance lane.
- Batch review/apply is intentionally absent; single-review stale guards and
  idempotency should remain the authority before bulk controls are introduced.
