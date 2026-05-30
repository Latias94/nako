# HDR Tone Mapping Pipeline - TODO

Status: Draft
Last updated: 2026-05-30

## M0 - Research And Scope Freeze

- [ ] HTP-010 [owner=unassigned] [deps=none] [scope=docs/workstreams/hdr-tone-mapping-pipeline,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Confirm the smallest executable HDR tone-mapping slice, required probe/client facts, shared scopes, and validation before any code changes.
  Validation: `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md`
  Review: Planner review before changing status from draft to active.
  Evidence: updated `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT. Do not edit Rust code in this task.

## M1 - Playback Color Requirement Vocabulary

- [ ] HTP-020 [owner=blocked] [deps=HTP-010] [scope=to-be-confirmed]
  Goal: Model playback-owned color compatibility and tone-mapping requirements after HTP-010 freezes the executable seam.
  Validation: to be confirmed by HTP-010.
  Review: Use `review-workstream` before accepting completion.
  Evidence: to be confirmed.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: Blocked until HTP-010 approves implementation scope.

## M2 - Transcode Tone-Mapping Strategy

- [ ] HTP-030 [owner=blocked] [deps=HTP-020] [scope=to-be-confirmed]
  Goal: Propagate color requirements into transcode strategy and FFmpeg command planning.
  Validation: to be confirmed by HTP-010.
  Review: Use `review-workstream` before accepting completion.
  Evidence: to be confirmed.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: Blocked until HTP-010 approves implementation scope.

## M3 - Verification And Closeout

- [ ] HTP-040 [owner=planner] [deps=HTP-030] [scope=docs/workstreams/hdr-tone-mapping-pipeline,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Verify final gates, record evidence, and close or split follow-ons.
  Validation: to be confirmed after implementation tasks are activated.
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split hardware vendor tuning, device profile databases, or UI controls into follow-ons.
