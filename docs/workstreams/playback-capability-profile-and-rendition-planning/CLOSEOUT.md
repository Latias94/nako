# Playback Capability Profile And Rendition Planning - Closeout

Status: Completed
Last updated: 2026-05-28

## Final Status

Closed on 2026-05-28. PCPR-010 through PCPR-040 are complete.

The lane preserved Public/Admin playback behavior while replacing the old
execution-shaped planner payload with a single typed rendition boundary.

## Shipped Boundary Changes

- `PlaybackDecision` now exposes `rendition: PlaybackRenditionPlan` as the only
  selected-output payload.
- `TranscodeRenditionPlan` carries both the safe `TranscodePlan` and the
  source-aware `TranscodeRequirement`.
- Removed duplicate top-level `direct_play`, `transcode_plan`, and
  `transcode_requirement` fields from `PlaybackDecision`.
- Deleted the shallow `PlaybackProfile` adapter.
- Moved remux/HLS transcode profile construction, output constraints, track
  selection, and request-key identity to `PlaybackTargetProfile`.
- Updated ADR 0044 to describe rendition plans instead of execution plans.

## Review Result

No blocking workstream compliance or code-quality findings remain.

Important review notes:

- Public Client DTOs still expose only safe `direct_play` and `transcode_plan`
  fields. Internal rendition details, source locators, host paths, command
  lines, and transcode requirements remain hidden.
- Remux/HLS request keys now use the richer target-profile identity. This is
  intentional request-key churn and acceptable before user compatibility
  constraints exist.
- `nako-server` renderer playback, remux, and HLS flows now consume the
  rendition boundary through focused helpers.

## Verification

Fresh gates:

- `cargo nextest run -p nako-playback --no-fail-fast` passed: 18 tests.
- `cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast`
  passed: 1 test.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed: 87 tests,
  296 skipped; nextest run `5e5baedf-e079-4d7b-947e-e3017cc43a22`.
- `python3 -m json.tool docs/workstreams/playback-capability-profile-and-rendition-planning/WORKSTREAM.json`
- `cargo fmt --all -- --check`
- `git diff --check`

## Follow-Ons

- Adaptive HLS ladder and bitrate/resolution switching.
- fMP4/CMAF output mode.
- DLNA device profile design.
- Subtitle/audio/HDR transcode maturity.
- rsmpeg adapter feasibility.
- Remote transcode workers.

## Residual Risk

- The Public Client capability DTO is still shallow. That is acceptable because
  this lane removed duplicate internal planning shapes first. Richer profile
  input should be added as a separate product/API lane.
