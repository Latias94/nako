# Admin Web Provider Depth Governance - Closeout

Status: Closed
Closed: 2026-06-02

## Shipped Scope

This lane exposed durable Metadata Candidate Review evidence and accepted-review
application through explicit Admin API/Web governance without broadening review
acceptance into hidden catalog mutation.

Shipped behavior:

- `AWPDG-020` added redaction-safe read-only Admin API detail for durable
  Candidate Review evidence and accepted-review application plan facts.
- `AWPDG-030` added an explicit Admin API apply mutation that calls
  `MetadataCandidateReviewApplicationService` with `item_id`,
  `expected_updated_at_ms`, and an operator idempotency key.
- The apply mutation exposes applied/noop/conflict/replay facts, returns only an
  idempotency-key fingerprint, and applies only root Provider Subject /
  Provider Mapping state.
- `AWPDG-040` added the Web Admin route for inspecting durable Candidate Review
  evidence, separating preview related graph evidence from accepted root
  Provider Mapping facts, and requiring a two-step apply confirmation.
- Web Admin fixture mutation stays disabled, live data-source mapping is
  contract-tested, route-state tests cover the confirmation flow, and the bundle
  budget stays under the committed threshold.
- `AWPDG-050` closes the lane and keeps related-node hierarchy application,
  provider endpoint depth, and Candidate Review navigation/listing as separate
  follow-ons.

## Confirmed Boundaries

- No Public Client API route or protocol DTO was added.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation was added.
- No raw provider payload, description body, tag body, image URL, local path,
  token, header, proxy URL, or operator idempotency key is rendered through
  Admin/Web.
- Candidate Review application does not reuse Generated Artifact apply outcome
  tables or create another metadata apply executor.
- Existing Admin Catalog Governance Provider Mapping review routes remain
  separate from durable Candidate Review governance.
- Provider endpoint breadth, including Douban TV/episode support, remains split.

## Validation

Fresh implementation gates from `AWPDG-020`:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_detail --no-fail-fast`
  passed: 1 test run, 1 passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 5
  tests run, 5 passed.
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
  passed: 108 tests run, 108 passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Fresh implementation gates from `AWPDG-030`:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_apply_commits_root_mapping_and_replays --no-fail-fast`
  passed: 1 test run, 1 passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 5
  tests run, 5 passed.
- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 6 tests run, 6 passed.
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
  passed: 109 tests run, 109 passed.

Fresh implementation gates from `AWPDG-040`:

- `npm --prefix web run check` passed.
- `npm --prefix web run test` passed: 10 test files, 116 tests.
- `npm --prefix web run build:budget` passed: `total-js` 1162.97 KiB raw /
  340.92 KiB gzip, under the 1250/341 KiB budget.
- Browser smoke passed against
  `http://127.0.0.1:3001/admin/metadata/candidate-reviews?review_id=smoke-review`:
  HTTP 200, key route text present, and no console errors.

Fresh closeout gates:

- `python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## Follow-Ons

- `proposed:provider-review-related-hierarchy-application`: apply related graph
  nodes, child Provider Subjects, child Provider Mappings, or Media Item
  hierarchy changes only through a new policy and operator workflow.
- `proposed:douban-tv-episode-endpoint-depth`: prove Douban TV/episode endpoint
  semantics before broadening Douban provider capabilities.
- `docs/workstreams/admin-candidate-review-list-navigation/` (active): add an
  Admin queue/list and navigation entry for durable Candidate Reviews instead
  of requiring a direct `review_id` URL.
- `proposed:provider-governance-bulk-review`: handle batch or cross-provider
  governance only after single-review semantics remain stable.

## Residual Risks

- Web Admin currently has a direct detail route, not a discoverable Candidate
  Review queue. Operators still need another surface to find review IDs.
- Related graph nodes remain preview-only. That protects Media Item identity,
  but accepted season/episode hierarchy still needs a dedicated governance lane.
- Douban TV/episode support is intentionally unsupported until a provider-depth
  lane proves endpoint-backed semantics.
