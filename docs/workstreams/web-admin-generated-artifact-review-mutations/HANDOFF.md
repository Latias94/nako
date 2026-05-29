# Web Admin Generated Artifact Review Mutations - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

The lane is closed. The previous `web-admin-generated-artifacts-automation`
lane owns the proposal queue, and this lane added the guarded review-plan and
accept/reject mutation path for the new `web/` Admin frontend.

Important contract fact: backend `review-plan` is `POST
/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan` with
request body `{ decision }`.

## Active Task

None.

## Next Action

Recommended follow-ons:

- Metadata Authority apply after accepting a Generated Artifact.
- Bulk review only after per-artifact permission/readiness semantics harden.
- Automation Provider adapter breadth and local runtime integration.
- Addon task/event diagnostics for Automation Provider execution visibility.
