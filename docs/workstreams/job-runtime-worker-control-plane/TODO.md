# Job Runtime Worker Control Plane Task Ledger

Status: Active
Last updated: 2026-05-19

## M0 - Runtime Inventory And Contract

- [x] JRWCP-010 [owner=codex] [deps=none] [scope=docs/workstreams/job-runtime-worker-control-plane,docs/adr,crates/taru-core,crates/taru-db,crates/taru-server]
  Goal: Inventory existing durable job execution paths, ADR constraints, and
  worker/runtime supervisor surfaces; choose the first shared contract shape.
  Validation: design notes identify each current execution mode and name the
  first code slice.
  Review: do not propose a generic untyped scheduler; keep job-kind execution
  typed.
  Evidence: updated `DESIGN.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
  Result: DONE. Inventory shows existing `RuntimeSupervisor::spawn_job` and
  `DurableJobRuntime` cover immediate one-job execution, while Managed Artwork
  has typed claim/commit/fail/requeue but no supervised worker loop. First code
  slice should use a concrete Managed Artwork worker registered through
  `RuntimeSupervisor`, keeping claim/commit/fail in the Managed Artwork
  repository and postponing generic lease schema.
  Handoff: Continue with `JRWCP-020`.

## M1 - Managed Artwork Worker Tracer Bullet

- [x] JRWCP-020 [owner=codex] [deps=JRWCP-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server]
  Goal: Add a supervised Managed Artwork ingest worker loop that claims and
  processes queued jobs through the existing safe artifact pipeline.
  Validation: focused server/runtime test proves queued ingest is processed by
  the worker without calling Admin `process-next`.
  Review: worker loop must use bounded resource permits and must not expose raw
  source/storage/error values.
  Evidence: success path stores artifact and marks job succeeded.
  Result: DONE. Added opt-in `[artwork].ingest_worker_enabled` and
  `ingest_worker_idle_ms`, registered a concrete
  `managed_artwork_ingest_worker` through `RuntimeSupervisor`, and shared the
  same `process_next_unit` pipeline between the worker and Admin
  `process-next`. Focused HTTP/runtime coverage proves a queued ingest is
  stored by the worker without calling Admin `process-next`, with no public
  artwork publication or locator leaks.
  Handoff: Continue with `JRWCP-030` failure/recovery semantics.

- [ ] JRWCP-030 [owner=codex] [deps=JRWCP-020] [scope=crates/taru-db,crates/taru-server]
  Goal: Prove worker failure handling and restart recovery for Managed Artwork
  ingest.
  Validation: focused tests for safe failed summary and stale running recovery
  policy.
  Review: recovery must not duplicate artifacts or claim running jobs that are
  still owned.
  Evidence: failed job remains requeueable; stale claim policy is explicit.
  Handoff: Continue with cancellation or split it.

## M2 - Control Plane Semantics

- [ ] JRWCP-040 [owner=codex] [deps=JRWCP-030] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server,docs/api]
  Goal: Decide and implement the first cancellable worker boundary only if the
  worker has an ownership token/checkpoint that can observe cancellation.
  Validation: cancellation test proves request is observed by worker or rejected
  as not cancellable.
  Review: no unsafe task killing; no false cancellation claims after restart.
  Evidence: Admin response is redacted and state-machine behavior is documented.
  Handoff: Split generic cancellation if it grows beyond Managed Artwork.

## M3 - Closeout

- [ ] JRWCP-050 [owner=codex] [deps=JRWCP-020] [scope=workspace,docs]
  Goal: Close or split the lane after Managed Artwork worker semantics are
  proven and follow-ons are explicit.
  Validation: focused tests, `cargo check`, `cargo fmt --all -- --check`, and
  `git diff --check`.
  Review: no half-migrated job kind is marked complete.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and `WORKSTREAM.json`.
  Handoff: Split metadata/webhook/NFO/automation workers as follow-on lanes if
  needed.
