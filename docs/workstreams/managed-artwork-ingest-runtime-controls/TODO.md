# Managed Artwork Ingest Runtime Controls Task Ledger

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Contract

- [x] MAIRC-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-ingest-runtime-controls,docs/workstreams/README.md]
  Goal: Open the runtime controls lane with explicit requeue, cancellation,
  redaction, and non-goal boundaries.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with failed-ingest requeue.

## M1 - Failed Ingest Requeue

- [x] MAIRC-020 [owner=codex] [deps=MAIRC-010] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,docs/api]
  Goal: Implement `POST /admin/v1/artwork/ingests/{ingest_id}/requeue` for
  failed Managed Artwork ingests and failed durable jobs.
  Validation: focused API/server/db tests plus relevant cargo check.
  Review: requeue must not create duplicate candidates, ingests, jobs, artifact
  rows, or files; it must not fetch bytes directly.
  Evidence: failed ingest/job return to queued; replay against queued returns
  `requeued = false`; stored/running states return conflict; response is
  redacted.
  Handoff: Completed with `MAIRC-030` in the same HTTP regression slice.

## M2 - Retry Execution Regression

- [x] MAIRC-030 [owner=codex] [deps=MAIRC-020] [scope=crates/nako-server,docs/api]
  Goal: Prove a failed ingest can be requeued and then processed by the existing
  `process-next` route into a stored artifact after the source becomes valid.
  Validation: focused server test and redaction inventory.
  Review: `process-next` remains the executor; requeue itself must not perform
  fetch/validation/storage.
  Evidence: first process fails with safe summary, requeue resets state, second
  process stores artifact with redacted response.
  Handoff: Cancellation, automatic scheduling, repair, public gallery browsing,
  and cleanup policy remain follow-ons.

## M3 - Closeout

- [x] MAIRC-040 [owner=codex] [deps=MAIRC-030] [scope=workspace,docs]
  Goal: Close the lane with fresh validation evidence, HTTP docs, and explicit
  follow-ons.
  Validation: focused nextest gates; relevant cargo check; `cargo fmt --all
  -- --check`; `git diff --check`; redaction inventory.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Keep active cancellation, automatic retry scheduling, repair, public
  gallery browsing, and artifact cleanup policy changes as separate lanes.
