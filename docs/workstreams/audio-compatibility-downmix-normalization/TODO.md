# Audio Compatibility Downmix Normalization - TODO

Status: Active
Last updated: 2026-05-30

## M0 - Scope And Evidence Freeze

- [x] ACDN-010 [owner=planner] [deps=none] [scope=docs/workstreams/audio-compatibility-downmix-normalization,docs/architecture]
  Goal: Freeze problem, target state, non-goals, lane ownership, and first validation gates.
  Validation: `python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json`; `git diff --check -- docs/workstreams/audio-compatibility-downmix-normalization docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md CONTEXT.md`
  Evidence: `docs/workstreams/audio-compatibility-downmix-normalization/DESIGN.md`
  Context: `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`
  Handoff: DONE. Planner opened the lane and made `ACDN-020` the first executable task.

## M1 - Playback Requirement Vocabulary

- [x] ACDN-020 [owner=codex] [deps=ACDN-010] [scope=crates/nako-playback/src/capability.rs,crates/nako-playback/src/values.rs,crates/nako-playback/src/lib.rs]
  Goal: Add playback-owned **Audio Output Requirement** values and compatibility reasons for channel support, downmix, and normalization intent.
  Validation: `cargo nextest run -p nako-playback audio --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: playback planner/unit tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`.
  Handoff: DONE. Added playback-owned audio output requirement values, downmix/normalization intent reasons, and playback tests proving channel-limited audio selects transcode instead of remux.

## M2 - Transcode Policy Propagation

- [x] ACDN-030 [owner=codex] [deps=ACDN-020] [scope=crates/nako-transcode/src/policy.rs,crates/nako-transcode/src/pipeline.rs,crates/nako-transcode/src/profile.rs,crates/nako-server/src/app/playback/mod.rs,crates/nako-server/src/app/playback/hls.rs]
  Goal: Propagate audio output requirements into transcode profile/pipeline planning and server playback adaptation without moving playback policy into transcode.
  Validation: `cargo nextest run -p nako-transcode audio --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`
  Review: Use `review-workstream` before accepting completion.
  Evidence: transcode policy and HLS adaptation tests.
  Context: `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`.
  Handoff: DONE. Implemented audio output requirement propagation through transcode policy/pipeline/profile and server HLS adaptation. Follow-up diagnosis showed the server HLS failure was an existing running-playlist concurrency-sensitive test issue; the gate was stabilized by widening the focused test timeout and waiting for the first segment route readiness. Full ACDN-030 validation now passes.

## M3 - FFmpeg Audio Filter Planning

- [ ] ACDN-040 [owner=unassigned] [deps=ACDN-030] [scope=crates/nako-transcode/src/ffmpeg.rs,crates/nako-transcode/src/lib.rs,crates/nako-transcode/src/tests*]
  Goal: Make FFmpeg command planning emit deterministic audio downmix and normalization filters when requested by the transcode policy.
  Validation: `cargo nextest run -p nako-transcode hls audio --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`
  Review: Use `review-workstream` before accepting completion.
  Evidence: FFmpeg command-plan tests and HLS regression tests.
  Context: `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`.
  Handoff: Keep filter choices deterministic and explainable. Do not add UI preference storage in this task.

## M4 - Diagnostics, Verification, And Closeout

- [ ] ACDN-050 [owner=planner] [deps=ACDN-040] [scope=docs/workstreams/audio-compatibility-downmix-normalization,docs/architecture/PLAYBACK.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Record final evidence, update capability maps, and close or split follow-ons.
  Validation: `cargo nextest run -p nako-playback audio --no-fail-fast`; `cargo nextest run -p nako-transcode hls audio --no-fail-fast`; `cargo nextest run -p nako-server hls --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split persisted preferences, client UI controls, device profile database, or dialogue clarity into follow-ons.
