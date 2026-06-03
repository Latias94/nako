# Provider Governance Audit Undo Evidence

Date: 2026-06-03
Selected slice: read-only governance audit timeline and undo plan for Admin
metadata candidate review surfaces.

## Selection

Chose the read-only governance slice because it improves operator trust without
crossing into unsafe rollback behavior. The current metadata review flow already
has replay-safe apply semantics, stale-state guards, root-only Provider Mapping
updates, and durable batch status. The smallest useful product-readiness step
was to surface that governance state explicitly on Admin review responses
instead of inventing a mutation undo endpoint without a persisted pre-apply
snapshot.

## Shipped Behavior

- Added `governance` to Admin metadata candidate review detail, list, queue,
  batch-plan, durable batch item, apply, and batch-apply responses.
- `governance.audit_timeline` now reports read-only review, plan, batch-status,
  and application-result events with replay-safe semantics.
- `governance.undo_plan` now exposes root-only rollback boundaries,
  stale-state-guard timestamps, target mapping status, and explicit reasons why
  mutation undo remains deferred.
- Refreshed generated Admin TypeScript contracts for both
  `apps/admin-web/src/adminApi/generated/contract.ts` and
  `web/src/api/admin/generated/contract.ts`.
- Extended server route tests to assert governance fields, replay behavior,
  stale-state guard wiring, and redaction across list/detail/batch-plan/apply/
  batch-apply surfaces.

## Boundaries Preserved

- No Public Client API route or DTO changes.
- No schema migration or persisted rollback snapshot.
- No related hierarchy mutation or hierarchy undo.
- No raw provider payloads, proxy URLs, tokens, local paths, Source
  Fingerprints, or raw backend/provider errors exposed in Admin responses.
- No mutation-capable undo endpoint was added in this slice.

## Validation

- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract_excludes_generated_fetch_runtime_and_raw_sensitive_fields admin_web_generated_contract_matches_generator_output --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_list_is_item_scoped_redacted_and_read_only admin_v1_metadata_candidate_review_batch_plan_is_bounded_redacted_and_read_only admin_v1_metadata_candidate_review_batch_apply_reports_partial_results_redacted admin_v1_metadata_candidate_review_detail_is_redacted_and_read_only admin_v1_metadata_candidate_review_apply_commits_root_mapping_and_replays --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_queue_filters_global_rows_without_writes admin_v1_metadata_candidate_review_batch_durable_create_replays_and_reports_status --no-fail-fast`
  passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-02-04c-provider-governance-audit-undo`
  passed.
- `cargo fmt --all -- --check` passed after formatting the worktree.
- `git diff --check` passed with LF/CRLF normalization warnings only.
- `npm run check --prefix apps/admin-web` and `npm run check --prefix web`
  could not run because `tsc` was unavailable in this worktree. Contract parity
  was still proven by generator refresh plus `nako-api` admin contract tests.

## Follow-ons

- Mutation-capable undo needs an explicit rollback design with persisted
  pre-apply snapshot and stale-state protection semantics.
- Related hierarchy application/undo remains a separate metadata-control-plane
  slice.
- Any future Admin UI over undo guidance should consume the generated contract
  output rather than introducing hand-written route strings or frontend-only
  governance shapes.
