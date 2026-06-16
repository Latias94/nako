# backend readiness control plane audit

## Problem

`GET /admin/v1/overview` already reports setup, scan, playback, storage, network, and backup posture, but it does not surface durable-job pressure as a first-class operator signal. That leaves the self-hosted control plane with no direct overview indicator for queued or delayed background work even though the repo already maintains redacted job pressure diagnostics.

## Goal

Add a durable-job readiness signal to the Admin overview so operators can see whether background work is clean, queued, or under pressure without exposing job payloads, raw errors, paths, or tokens.

## Scope

In scope:

- Add one new `operator_readiness` area and reason for durable-job pressure.
- Reuse existing durable-job queue pressure facts as the source of truth.
- Keep the summary redaction-safe.
- Update Admin contract generation and overview tests.

Out of scope:

- New job scheduler behavior.
- New job mutation routes.
- Broader durable-job lifecycle redesign.
- Non-admin UI changes.

## Requirements

1. `AdminOverviewResponse.operator_readiness` includes a durable-job check.
2. The check is derived from existing job pressure data or equivalent safe runtime facts.
3. The check stays redaction-safe and does not expose job payloads, raw storage identifiers, or backend errors.
4. Public/Admin contract artifacts stay in sync with the Rust DTOs.
5. Tests prove the new overview readiness state and its JSON shape.

## Acceptance Criteria

- A healthy queue state reports durable-job readiness as ready.
- Queue pressure or delayed retry pressure reports durable-job readiness as degraded.
- The serialized overview response contains the new readiness area and reason.
- The response body still omits unsafe job payload material.

## Notes

- Prefer the existing job pressure summary pipeline instead of inventing a new job-health subsystem.
- If a helper is needed, keep it local to the Admin overview path.
