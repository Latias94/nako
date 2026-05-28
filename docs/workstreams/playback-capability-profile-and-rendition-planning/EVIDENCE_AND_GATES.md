# Playback Capability Profile And Rendition Planning Evidence And Gates

Status: Completed
Last updated: 2026-05-28

## Required Gates

```text
python3 -m json.tool docs/workstreams/playback-capability-profile-and-rendition-planning/WORKSTREAM.json
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 PCPR-010: Opened the workstream and selected the first
  behavior-preserving fearless refactor slice: replace execution-shaped
  playback output with `PlaybackRenditionPlan` and delete the shallow
  `PlaybackProfile` adapter.
- 2026-05-28 PCPR-020: Replaced `PlaybackDecision.execution` with
  `PlaybackDecision.rendition`, added `PlaybackRenditionPlan` and
  `TranscodeRenditionPlan`, moved transcode requirements under the transcode
  rendition, and preserved Public Client redaction mapping.
- 2026-05-28 PCPR-030: Deleted `PlaybackProfile`. `PlaybackTargetProfile` now
  builds remux and HLS transcode profiles, owns output constraints, and supplies
  the richer profile identity for remux/HLS request keys.
- 2026-05-28 PCPR-040: Focused gates passed:
  `cargo nextest run -p nako-playback --no-fail-fast` (18 passed);
  `cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast`
  (1 passed); `cargo nextest run -p nako-server playback --no-fail-fast` (87
  passed, 296 skipped). Final non-test checks are recorded in `CLOSEOUT.md`.

## Notes

- Public Client DTOs may continue to expose safe `direct_play` and
  `transcode_plan` fields. They must not expose internal rendition payloads,
  transcode requirements, raw locators, command lines, or host paths.
- Request key churn is accepted in this lane because Nako has no compatibility
  users yet and the target-profile identity is the more correct long-term
  shape.
