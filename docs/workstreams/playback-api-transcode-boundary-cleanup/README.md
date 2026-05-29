# Playback API Transcode Boundary Cleanup

Status: Completed
Last updated: 2026-05-29

This fearless refactor lane removes transcode execution vocabulary from API
crate ownership. `nako-api` should describe stable Admin and Public Client wire
contracts. `nako-server` should adapt runtime/transcode facts into those
contracts because the server composes playback planning, configuration, and
FFmpeg runtime state.

## Why Now

Recent playback lanes deepened HLS, adaptive ladders, sidecar media groups, and
seek/restart behavior. That makes direct `nako-api -> nako-transcode`
dependencies more expensive: every hardware pipeline or FFmpeg planning change
can look like an API-layer dependency change even when the wire contract is
unchanged.

## Target Result

- `nako-api` no longer declares a direct dependency on `nako-transcode`.
- Public Client and Admin wire shapes remain stable.
- Server-side adapters map transcode runtime types into API DTOs at the HTTP and
  app-composition boundary.
- Any remaining `nako-playback -> nako-transcode` dependency is tracked as a
  separate planner/runtime follow-on rather than hidden in API cleanup.

## Completed Result

- `nako-api` no longer has a direct `nako-transcode` dependency.
- Admin hardware and transcode-pipeline readiness fields are API-owned DTOs
  that preserve previous snake_case serialized values.
- `nako-server` owns transcode runtime/config to Admin DTO mapping through a
  local adapter module.
- Public Client playback decision conversion still redacts source locators and
  internal rendition state without naming `nako_transcode` in `nako-api`.
- The remaining `nako-playback -> nako-transcode` surface is split to the
  proposed `playback-planner-transcode-value-vocabulary` follow-on.

## Architecture References

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0053-application-control-plane-boundary.md`
