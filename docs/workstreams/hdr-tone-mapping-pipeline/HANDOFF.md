# HDR Tone Mapping Pipeline - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`HTP-020` is complete, planner-verified, and committed on the HDR branch. The
task stayed playback-only after the planner merged the accepted `ACDN-020`
audio output baseline into this HDR branch.

## Next Task

`HTP-030` is ready for planner assignment after
`transcode-interface-and-runtime-plan-deepening` closeout. The HDR playback
vocabulary slice is accepted, including the reviewer finding that remux cannot
satisfy HDR-to-SDR tone mapping for SDR-only clients.

Before starting implementation, sync or recreate the HDR worktree from current
`main`. The task should use the transcode-owned runtime and execution planner
Interfaces introduced by `TIRP-020` and `TIRP-030`, not server-side raw FFmpeg
request assembly.

Completed `HTP-020` implementation scope:

- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/lib.rs`

Planned `HTP-030` implementation scope:

- `crates/nako-transcode/src/policy.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/ffmpeg.rs`
- focused `nako-transcode` tests
- server HLS adaptation files only if the transcode-owned Interface requires a
  composition update

Owned docs scope for any continuation:

- `docs/workstreams/hdr-tone-mapping-pipeline/`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Required docs validation:

```text
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

Completed `HTP-020` implementation validation:

```text
cargo nextest run -p nako-playback hdr --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Research conclusion:

- current probe facts are sufficient for the first playback planning slice;
- current `supports_hdr` client capability is sufficient for the first slice;
- first implementation is playback-owned **Color Pipeline Requirement** and
  typed reasons, not FFmpeg command planning;
- first media-output behavior is a later software-first HLS HDR-to-SDR path.

HTP-020 result:

- added playback color pipeline source and requirement values;
- added typed reasons for HDR source detection, HDR passthrough, client HDR
  unsupported, tone mapping required, and deferred unsupported dynamic HDR;
- `TranscodeRequirement` carries the color pipeline requirement beside existing
  output constraints and audio output requirement;
- tests cover HDR passthrough, HDR-to-SDR tone-map intent, remux denial when
  tone mapping is required, and Dolby Vision deferred unsupported intent.

## Stop Conditions

Return to planner coordination if:

- implementation starts from an old branch that predates the accepted
  `TIRP-030` Interface shape;
- follow-up work would reintroduce server-side raw FFmpeg request assembly;
- the lane needs a new ADR before code can start;
- hardware-specific behavior cannot be represented by existing ADRs.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include implementation files changed, validation evidence, residual risks, and
whether the software-first HDR-to-SDR slice stayed inside the planned scope.
