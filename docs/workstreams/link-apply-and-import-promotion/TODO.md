# Link Apply And Import Promotion — TODO

Status: Active
Last updated: 2026-05-21

Task IDs use the `LAIP` prefix.

## M0 — Lane Open

- [x] LAIP-010 [owner=planner] [deps=managed-import-staging MIS-050,MIS-060] [scope=docs/workstreams/link-apply-and-import-promotion]
  Goal: Open the follow-on apply lane with mutation boundaries, non-goals,
  task order, and gates.
  Validation: workstream docs agree and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: Execute LAIP-020.

## M1 — Durable Acceptance And Audit Domain

- [x] LAIP-020 [owner=codex] [deps=LAIP-010,MIS-040] [scope=crates/taru-core,crates/taru-db]
  Goal: Add promotion apply IDs, operation/state enums, accepted plan snapshot,
  audit outcome records, repository traits, migrations, and backend-neutral
  contract tests.
  Validation: `cargo nextest run -p taru-db promotion_apply --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: core domain records, repository trait, SQLite/PostgreSQL migrations,
  facade forwarding, and DB contract tests. Completed with
  `ManagedImportPromotionApplyId`,
  `ManagedImportPromotionApplyState`, durable idempotency-keyed apply records,
  SQLite/PostgreSQL schema parity, repository adapters, and backend-neutral
  acceptance/audit contract coverage.
  Handoff: Wire app-service acceptance/replay in LAIP-030.

## M2 — App Service Acceptance And Idempotent Replay

- [ ] LAIP-030 [owner=codex] [deps=LAIP-020] [scope=crates/taru-server]
  Goal: Add app service command that explicitly accepts a promotion plan,
  records a durable apply attempt, replays matching idempotency keys, and rejects
  mismatched stale or blocked requests without storage mutation.
  Validation: focused server tests prove operator confirmation fields,
  redacted diagnostics, idempotent replay, stale-plan rejection, and no library
  file or Media Source write before mutation tasks.
  Evidence: `taru-server` app tests and service boundary.
  Handoff: Add VFS mutation primitives in LAIP-040.

## M3 — VFS Copy/Link Apply Primitive

- [ ] LAIP-040 [owner=codex] [deps=LAIP-020,LAIP-030,LNA-020] [scope=crates/taru-vfs]
  Goal: Add storage-mediated copy/hardlink/symlink apply primitives that reuse
  planning safety checks, never expose OS path mutation to server code, and
  return typed redacted outcomes.
  Validation: `cargo nextest run -p taru-vfs link --no-fail-fast`; focused copy
  apply tests; `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: VFS storage apply types, local backend tests, unsupported backend
  behavior.
  Handoff: Compose promotion apply orchestration in LAIP-050.

## M4 — Promotion Apply Orchestration

- [ ] LAIP-050 [owner=codex] [deps=LAIP-030,LAIP-040] [scope=crates/taru-server]
  Goal: Revalidate plan facts, execute selected storage operation, commit Media
  Source / duplicate relationship state only after target durability, and record
  terminal audit outcomes.
  Validation: server tests prove successful apply, blocked/stale apply,
  duplicate evidence behavior, no direct OS path mutation, and catalog writes
  after target creation only.
  Evidence: app service orchestration and tests.
  Handoff: Add partial-failure cleanup/rollback gates in LAIP-060.

## M5 — Partial Failure Rollback And Cleanup

- [ ] LAIP-060 [owner=codex] [deps=LAIP-050] [scope=crates/taru-server,crates/taru-vfs]
  Goal: Inject failures after storage creation and prove rollback or
  cleanup-pending audit behavior without marking artifacts promoted.
  Validation: focused tests with failing repository/storage doubles prove
  cleanup-complete and cleanup-pending outcomes.
  Evidence: rollback/cleanup tests and audit outcomes.
  Handoff: Decide NFO sidecar mutation split in LAIP-070.

## M6 — NFO Sidecar Mutation Split Decision

- [ ] LAIP-070 [owner=planner] [deps=LAIP-050,LAIP-060] [scope=docs/workstreams/link-apply-and-import-promotion]
  Goal: Decide whether NFO import/export apply belongs in this lane or must
  split to a dedicated sidecar-promotion lane.
  Validation: DESIGN/HANDOFF record backup, authority, rollback, and audit
  requirements.
  Evidence: updated split decision.
  Handoff: Implement NFO sidecar apply or split follow-on.

## M7 — Closeout

- [ ] LAIP-080 [owner=planner] [deps=LAIP-070] [scope=docs/workstreams/link-apply-and-import-promotion]
  Goal: Close or split Link Apply And Import Promotion.
  Validation: evidence gates are fresh; parent umbrella points to the next lane.
  Evidence: closeout journal and parent handoff.
  Handoff: Return to `post-rpd-product-hardening` for next lane scoring.
