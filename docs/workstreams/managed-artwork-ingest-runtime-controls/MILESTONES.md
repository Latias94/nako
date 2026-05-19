# Managed Artwork Ingest Runtime Controls Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Contract

Exit criteria:

- Workstream docs exist and agree on terminology.
- Requeue is separated from process execution, publication, repair, cleanup, and
  cancellation.
- `WORKSTREAM.json` parses.

## M1 - Failed Ingest Requeue

Exit criteria:

- `POST /admin/v1/artwork/ingests/{ingest_id}/requeue` exists.
- Failed ingests and failed managed-artwork jobs return to queued.
- Requeue is idempotent for already queued ingests.
- Stored/running/fetching/validating states do not requeue.
- Responses are redacted.

## M2 - Retry Execution Regression

Exit criteria:

- A failed ingest can be requeued and later processed through `process-next`.
- Requeue does not fetch, validate, store, publish, cleanup, or delete anything.
- No raw source URL, storage URI, local path, addon payload, token, validation
  detail, or content hash leaks through Admin responses.

## M3 - Closeout

Exit criteria:

- HTTP docs describe requeue and cancellation follow-ons.
- Focused tests and cargo check evidence are recorded.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Follow-ons are explicit and outside this lane.
