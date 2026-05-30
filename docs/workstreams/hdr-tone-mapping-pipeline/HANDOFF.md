# HDR Tone Mapping Pipeline - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

`HTP-020` is complete, planner-verified, and committed on the HDR branch. The
task stayed playback-only after the planner merged the accepted `ACDN-020`
audio output baseline into this HDR branch.

## Next Task

Return to planner coordination before starting `HTP-030`. The HDR playback
vocabulary slice is accepted, but the transcode/HLS implementation scope should
remain blocked until the planner resolves the overlapping audio `ACDN-030`
HLS gate concern or explicitly serializes those shared files.

Completed `HTP-020` implementation scope:

- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/lib.rs`

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
- tests cover HDR passthrough, HDR-to-SDR tone-map intent, and Dolby Vision
  deferred unsupported intent.

## Stop Conditions

Return to planner coordination if:

- `HTP-030` is requested before `HTP-020` review is complete;
- follow-up work would expand beyond playback-owned color requirement
  vocabulary without planner approval;
- the lane needs a new ADR before code can start;
- hardware-specific behavior cannot be represented by existing ADRs.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include implementation files changed, validation evidence, residual risks, and
whether `HTP-020` is ready for planner review.
