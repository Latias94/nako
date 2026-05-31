# HDR Tone Mapping Pipeline - Evidence And Gates

Status: Closed
Last updated: 2026-05-31

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

Reviewer finding follow-up:

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

Planner verification on 2026-05-31:

- `cargo nextest run -p nako-playback hdr --no-fail-fast` passed with 4 tests
  run and 30 skipped by filter.
- `cargo fmt --all -- --check` passed.
- `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`
  passed.
- `git diff --check` passed.

### HTP-030 - Software-first transcode tone-mapping strategy

Status: Done and accepted.

Evidence:

- `crates/nako-transcode/src/policy.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/tests/mod.rs`
- `crates/nako-server/src/app/tests/playback.rs`

Findings:

- Added transcode-owned color pipeline requirement values and reason bits so
  `nako-transcode` does not depend on `nako-playback`.
- HLS runtime planning carries HDR-to-SDR color intent into
  `TranscodeExecutionPolicy`, `TranscodeProfileIdentity`, and request identity.
- HDR-to-SDR HLS command planning emits a deterministic software
  `zscale,tonemap,zscale,format` video filter before H.264 encoding.
- Hardware tone mapping remains deferred: when HDR-to-SDR tone mapping is
  required and hardware was requested, the runtime plan selects the software
  pipeline through the existing CPU fallback path.
- Deferred dynamic HDR tone mapping remains unsupported in this slice and
  returns a typed unsupported error before command execution.
- Server changes stay as thin composition/mapping around the transcode-owned
  runtime and execution planner Interfaces; raw `HlsRequest` and FFmpeg builder
  logic remain inside `nako-transcode`.

Verification on 2026-05-31:

- `cargo nextest run -p nako-transcode hdr --no-fail-fast` passed with 4 tests
  run and 90 skipped by filter.
- First `cargo nextest run -p nako-server hls --no-fail-fast` run passed 63 of
  65 tests; two existing progressive HLS readiness tests timed out under load.
  Both timed-out tests passed when rerun individually.
- Fresh rerun of `cargo nextest run -p nako-server hls --no-fail-fast` passed
  with 65 tests run and 434 skipped by filter.
- `python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`
  passed and proved the updated workstream manifest remains valid JSON.
- `git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md`
  passed and found no whitespace errors in the scoped docs diff.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

Planner acceptance on 2026-05-31:

- `HTP-030` review found no blocking workstream or code-quality findings.
- Commit `39a9dd2f feat(transcode): add software hdr tone mapping plan` was
  merged into `main`.

### HTP-040 - Verification and closeout

Status: Done

Evidence:

- `docs/workstreams/hdr-tone-mapping-pipeline/CLOSEOUT.md`
- `docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Findings:

- The shipped lane covers playback color pipeline vocabulary and software-first
  HLS HDR-to-SDR FFmpeg planning.
- The lane intentionally does not cover hardware tone mapping, vendor filter
  chains, Dolby Vision/HDR10+ dynamic handling, device profile databases,
  operator smoke matrices, UI controls, Public Client API DTO changes, or media
  probe schema expansion.
- Remaining HDR depth should be opened as separate workstreams so the closed
  lane does not absorb unrelated hardware, device-profile, or UI work.

## Residual Risks

- Existing HDR color facts are enough for HDR10/PQ and HLG detection, but may
  not be enough for Dolby Vision dynamic handling, HDR10+ preservation, or
  device-specific passthrough behavior.
- Hardware tone mapping differs across VAAPI, QSV, NVENC, AMF, and CPU paths.
- The HLS progressive readiness tests can be sensitive to host load when the
  full filtered suite runs concurrently; reruns passed in this verification.
- Tone mapping, audio downmix, and HLS runtime all touch playback/transcode
  seams, so follow-ons should stay explicitly scoped.
