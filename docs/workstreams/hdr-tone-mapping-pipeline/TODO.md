# HDR Tone Mapping Pipeline - TODO

Status: Active
Last updated: 2026-05-30

## M0 - Research And Scope Freeze

- [x] HTP-010 [owner=codex] [deps=none] [scope=docs/workstreams/hdr-tone-mapping-pipeline,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Confirm the smallest executable HDR tone-mapping slice, required probe/client facts, shared scopes, and validation before any code changes.
  Validation: `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`; `git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md`
  Review: Planner review before changing status from draft to active.
  Evidence: updated `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: DONE. Research confirmed a playback-first `HTP-020` slice and a later software-first HLS tone-map media slice. Planner activated `HTP-020` after merging accepted `ACDN-020` into this HDR branch.

## M1 - Playback Color Requirement Vocabulary

- [x] HTP-020 [owner=codex] [deps=HTP-010] [scope=crates/nako-playback/src/capability.rs,crates/nako-playback/src/values.rs,crates/nako-playback/src/lib.rs]
  Goal: Add playback-owned **Color Pipeline Requirement** values and typed HDR compatibility reasons that distinguish HDR passthrough, HDR-to-SDR tone-map intent, and unsupported/deferred HDR paths.
  Validation: `cargo nextest run -p nako-playback hdr --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: playback planner/unit tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: DONE. Added playback-owned color pipeline source/requirement values, HDR passthrough vs HDR-to-SDR vs deferred unsupported reasons, and playback tests proving the requirement is carried by transcode decisions. No `nako-transcode`, server HLS, Public Client API DTO, media probe schema, or web player code was edited.

## M2 - Transcode Tone-Mapping Strategy

- [ ] HTP-030 [owner=blocked] [deps=HTP-020] [scope=crates/nako-transcode/src/policy.rs,crates/nako-transcode/src/pipeline.rs,crates/nako-transcode/src/profile.rs,crates/nako-transcode/src/ffmpeg.rs,crates/nako-transcode/src/tests*,crates/nako-server/src/app/playback/mod.rs,crates/nako-server/src/app/playback/hls.rs]
  Goal: Propagate **Color Pipeline Requirement** into transcode policy, profile identity, HLS adaptation, and deterministic software-first HDR-to-SDR FFmpeg command planning.
  Validation: `cargo nextest run -p nako-transcode hdr --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: transcode policy/profile/command-plan tests, server HLS adaptation tests when server code changes, and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/hdr-tone-mapping-pipeline/CONTEXT.jsonl`.
  Handoff: Blocked until `ACDN-040` is accepted or the planner explicitly serializes the shared transcode/HLS scope. Keep hardware tone mapping, device-specific filter chains, Dolby Vision dynamic handling, HDR10+ preservation, and operator smoke matrices as follow-ons unless the planner splits them into a dedicated task.

## M3 - Verification And Closeout

- [ ] HTP-040 [owner=planner] [deps=HTP-030] [scope=docs/workstreams/hdr-tone-mapping-pipeline,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Verify final gates, record evidence, and close or split follow-ons.
  Validation: to be confirmed after implementation tasks are activated.
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split hardware vendor tuning, device profile databases, or UI controls into follow-ons.
