# Architecture Map Related Hierarchy Closeout Reconciliation

## Goal

Update current architecture navigation after the Admin-only related hierarchy
application surface shipped, so future planning no longer treats
`provider-review-related-hierarchy-application` as an unstarted proposed lane.

## Requirements

- Mark the shipped related hierarchy application/admin surface as completed in
  current architecture maps.
- Remove `proposed:provider-review-related-hierarchy-application` from current
  follow-on queues where it now points at completed work.
- Preserve historical legacy workstream closeout wording; do not rewrite old
  evidence documents.
- Keep remaining true follow-ons visible: Douban TV/episode endpoint depth,
  Public Client governance, mutation-capable undo, Admin Web UX, and durable
  bulk related hierarchy execution where appropriate.
- Do not modify Rust, generated API contracts, schema, or runtime behavior.

## Acceptance Criteria

- [x] `docs/GOALS.md`, `docs/ROADMAP.md`,
      `docs/architecture/LIBRARY_PIPELINE.md`, and
      `docs/architecture/WORKSTREAM_LINKS.md` no longer list the shipped related
      hierarchy application as an active proposed lane.
- [x] `docs/architecture/LANES.md` points library-metadata-control-plane at
      true remaining candidates instead of the already-shipped audit/read-only
      or related hierarchy lanes.
- [x] Current maps link to the archived Trellis task that shipped the Admin
      plan/apply surface.
- [x] Remaining follow-ons are named narrowly and do not imply the completed
      backend/Admin surface is still missing.
- [x] Trellis task context validates.
- [x] `git diff --check` passes.

## Out Of Scope

- No code changes.
- No legacy workstream evidence rewrite.
- No new ADR unless a durable boundary conflict is discovered.
- No implementation task creation for the next lane in this task.

## Research Notes

- Shipped task:
  `.trellis/tasks/archive/2026-06/06-05-provider-hierarchy-application-admin/`.
- Backend-only predecessor:
  `.trellis/tasks/archive/2026-06/06-02-01c-provider-review-related-hierarchy-application/`.
- Current `docs/architecture/LANES.md` already treats the lane as idle after
  accepted-review related hierarchy application landed.
