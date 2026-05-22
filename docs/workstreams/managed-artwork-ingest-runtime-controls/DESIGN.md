# Managed Artwork Ingest Runtime Controls Design

Status: Completed
Last updated: 2026-05-19

## Problem

Managed Artwork ingest failures are currently terminal at the runtime-control
surface. `process-next` can mark one ingest and durable job as failed, but an
administrator has no safe command to retry after fixing a transient provider,
network, or configuration issue.

Retry must not be modeled as "accept the candidate again" because acceptance is
idempotent and returns the existing ingest/job. It also must not be modeled as
"publish a fallback artifact" because publication is a separate Selected
Artwork decision. Runtime control needs its own boundary.

## Target State

- Admin users can requeue a failed Managed Artwork ingest by ingest ID.
- Requeue only works for failed ingests with failed `managed_artwork_ingest`
  jobs.
- Requeue sets ingest status back to `queued`.
- Requeue sets the existing durable job back to `queued`.
- Requeue clears `failure_code`, job `error`, job `summary_json`, `started_at`,
  and `completed_at`.
- Requeue does not create a new candidate, side effect, or ingest row.
- Requeue does not delete artifacts or files.
- Requeue does not publish Selected Artwork.
- `process-next` remains the only executor that fetches, validates, and stores
  Managed Artwork bytes.
- Admin responses stay redacted.

## Route Direction

Preferred first command:

```text
POST /admin/v1/artwork/ingests/{ingest_id}/requeue
```

Expected behavior:

- failed ingest + failed managed-artwork job: `200` with `requeued = true`;
- already queued ingest: `200` with `requeued = false`;
- running/fetching/validating/stored ingest: `409`;
- missing ingest: `404`;
- mismatched or missing durable job: `409`.

The command is idempotent for an already queued ingest so Admin UI retries do
not create duplicate jobs.

## Cancellation Boundary

Cancellation is intentionally split. Existing durable job state has only
`queued`, `running`, `succeeded`, and `failed`, and `process-next` is a short
Admin-triggered executor rather than a registered long-running worker with a
per-ingest cancellation token.

Adding cancellation correctly would require at least one of these decisions:

- introduce global job cancellation states;
- add process-local cancellation token registration for Managed Artwork fetch;
- define what happens to `fetching`/`validating` rows after process restart;
- decide whether cancellation is terminal or requeueable.

That is not needed to make failed ingest retry safe, so it remains a follow-on.

## Redaction Policy

The requeue response may include:

- ingest ID;
- candidate ID;
- job ID;
- library ID;
- item ID;
- image kind;
- current status;
- `requeued`;
- `had_failure`.

It must not include:

- raw candidate `source_uri`;
- addon side-effect payload/provenance JSON;
- provider query strings;
- addon tokens;
- `storage_uri`;
- `managed-artwork://...`;
- local artifact paths or artifact root paths;
- cache URIs;
- Source Locators;
- raw validation/fetch error messages;
- file contents;
- content hash values.

## Architecture Direction

- `nako-core` owns a `ManagedArtworkIngestRequeueRecord` and repository method.
- `nako-db` implements requeue in one transaction against
  `managed_artwork_ingests` and `jobs`.
- `nako-api` owns an explicit redacted Admin response DTO.
- `nako-server::app::artwork` validates command intent and maps repository
  output to Admin DTOs.
- `nako-server::http::admin` only parses the ingest ID and maps errors.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Requeue should reuse the existing job ID. | Medium | Existing accept is idempotent and current process-next claims by ingest/job state. | Keep response explicit; split new-job retry policy only if operational evidence requires it. |
| Failed ingests are safe to retry without deleting old artifacts. | High | Failed ingests have no committed artifact in the current state machine. | Repository should reject stored ingests and any inconsistent failed row with an artifact ID. |
| Cancellation needs a separate state-machine lane. | High | Global `JobStatus` lacks cancelled states and playback cancellation is process-local. | Document follow-on before adding ad hoc cancel behavior. |

## Closeout Condition

This lane can close when failed Managed Artwork ingests can be requeued through a
redacted Admin command, `process-next` can process the requeued work, invalid
states are rejected or idempotent as specified, HTTP docs describe the retry
boundary, and fresh validation evidence proves no raw source, storage, path,
payload, or content-hash leakage.

Closeout result: implemented and verified on 2026-05-19. Cancellation,
automatic retry scheduling, repair, Public Client gallery browsing, and cleanup
policy changes remain separate lanes.
