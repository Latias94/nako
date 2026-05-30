# HDR Tone Mapping Pipeline - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## HTP-010 Gates

```text
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

`HTP-010` is docs/research-only. Do not run or modify Rust implementation code
for this task.

## Confirmed Future Gates

### HTP-020 - Playback Color Requirement Vocabulary

```text
cargo nextest run -p nako-playback hdr --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run only after `ACDN-020` is complete, merged, or explicitly serialized by the
planner.

### HTP-030 - Software-First Transcode Tone-Mapping Strategy

```text
cargo nextest run -p nako-transcode hdr --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

The first media-output gate should be command-plan and HLS adaptation evidence.
Live FFmpeg hardware smoke tests are not required for the first CI slice and
should be split into an operations/release or hardware-matrix follow-on.

## Evidence Ledger

### HTP-010 - Research and scope freeze

Status: Done

Evidence:

- `docs/workstreams/hdr-tone-mapping-pipeline/DESIGN.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/TODO.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/MILESTONES.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/EVIDENCE_AND_GATES.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`

Findings:

- Existing media probe facts cover the first HDR planning slice: pixel format,
  bit depth, color range/space/transfer/primaries/chroma location, and HDR
  dynamic-range metadata.
- Existing client capability input is sufficient for the first slice through
  `supports_hdr`, but richer display capabilities and device profile databases
  are deferred.
- The first implementation task is playback-only `HTP-020`, adding a
  playback-owned **Color Pipeline Requirement** and typed reasons.
- The first real media-output slice is a later software-first HLS HDR-to-SDR
  tone-map path for HDR10/PQ or HLG sources.
- The workstream should remain draft until the planner serializes it after
  active `ACDN-020` playback vocabulary changes.

Verification on 2026-05-30:

- `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`
  passed and proved the workstream manifest remains valid JSON.
- `git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md`
  passed and found no whitespace errors in the scoped docs diff.
- Rust gates were intentionally not run because `HTP-010` is docs/research-only
  and did not edit implementation code.

### HTP-020 - Playback color requirement vocabulary

Status: Done

Evidence:

- `crates/nako-playback/src/values.rs`
- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/lib.rs`
- playback tests for HDR passthrough, HDR-to-SDR tone mapping intent, and
  deferred unsupported dynamic HDR paths

Findings:

- Added playback-owned color pipeline source and requirement values.
- `TranscodeRequirement` now carries the color pipeline requirement beside
  output constraints and audio output requirements.
- SDR-only clients receiving HDR10/PQ sources get `tone_mapping=required`.
- HDR-capable clients preserve source color when transcode is explicitly
  requested.
- Dolby Vision/HDR10+ style dynamic HDR is marked `deferred_unsupported` for
  this playback-only slice.
- Remux now rejects HDR sources for SDR-only clients so container remux cannot
  bypass HDR-to-SDR tone mapping intent.
- The task stayed inside `nako-playback`; transcode, server HLS, Public Client
  API DTOs, media probe schema, and web player code were not edited.

Verification on 2026-05-30:

- `cargo nextest run -p nako-playback hdr --no-fail-fast` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Reviewer finding follow-up on 2026-05-30:

- Added regression coverage for an SDR-only client playing an HDR source from
  an unsupported container. The planner now selects transcode, not remux, so
  `TranscodeRequirement.color_pipeline` carries the HDR-to-SDR intent.
- `cargo nextest run -p nako-playback hdr --no-fail-fast` passed with 4 HDR
  tests.
- `cargo fmt --all -- --check` passed.

Planner verification on 2026-05-30:

- `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`
  passed.
- `cargo nextest run -p nako-playback hdr --no-fail-fast` passed with 4 HDR
  tests run and 30 skipped by filter.
- `cargo nextest run -p nako-playback --no-fail-fast` passed with 34 tests
  run.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

## Residual Risks

- Existing HDR color facts are enough for HDR10/PQ and HLG detection, but may
  not be enough for Dolby Vision dynamic handling, HDR10+ preservation, or
  device-specific passthrough behavior.
- Hardware tone mapping differs across VAAPI, QSV, NVENC, AMF, and CPU paths.
- Tone mapping, audio downmix, and HLS runtime all touch playback/transcode
  seams, so implementation must be serialized unless scopes are narrowed.
