# HDR Tone Mapping Pipeline - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This workstream is closed. `HTP-030` was reviewed, verified, merged into
`main`, and accepted as the first software-first HLS HDR-to-SDR media-output
slice.

## Follow-ons

Open separate workstreams for:

- hardware tone mapping and vendor-specific filter chains;
- Dolby Vision/HDR10+ dynamic handling or preservation;
- device profile databases and richer display capability inputs;
- operator hardware smoke matrices and release diagnostics;
- UI/client controls for HDR behavior.

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

Completed `HTP-030` result:

- added transcode-owned color pipeline requirement values without introducing a
  `nako-transcode -> nako-playback` dependency;
- carried HDR-to-SDR tone mapping intent through HLS runtime planning,
  execution policy, profile identity, and request identity;
- made HDR-to-SDR HLS command planning emit a deterministic software video
  filter before H.264 encoding;
- rejected deferred dynamic HDR tone mapping as outside this slice;
- kept server code as playback-to-transcode mapping and HLS composition around
  the transcode-owned runtime/execution planner Interfaces.

Owned docs scope for any continuation:

- `docs/workstreams/hdr-tone-mapping-pipeline/`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Closeout docs validation:

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

Completed `HTP-030` implementation validation:

```text
cargo nextest run -p nako-transcode hdr --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
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

HTP-030 result:

- `TranscodeColorPipelineRequirement` mirrors playback color intent as a
  transcode-owned execution value;
- HLS runtime and profile identity include color pipeline identity for
  tone-mapped outputs;
- FFmpeg command planning uses software `zscale,tonemap,zscale,format` for
  HDR-to-SDR HLS output;
- an app-level HLS test proves HDR probe facts plus an SDR client reach the
  FFmpeg tone-map command through the real playback service path.

## Stop Conditions

Return to planner coordination if:

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
