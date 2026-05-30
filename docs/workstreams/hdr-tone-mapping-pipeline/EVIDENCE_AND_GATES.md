# HDR Tone Mapping Pipeline - Evidence And Gates

Status: Draft
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

Status: Done with concerns

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

## Residual Risks

- Existing HDR color facts are enough for HDR10/PQ and HLG detection, but may
  not be enough for Dolby Vision dynamic handling, HDR10+ preservation, or
  device-specific passthrough behavior.
- Hardware tone mapping differs across VAAPI, QSV, NVENC, AMF, and CPU paths.
- Tone mapping, audio downmix, and HLS runtime all touch playback/transcode
  seams, so implementation must be serialized unless scopes are narrowed.
