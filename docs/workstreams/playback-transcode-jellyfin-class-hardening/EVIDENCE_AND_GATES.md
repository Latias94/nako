# Playback Transcode Jellyfin-Class Hardening - Evidence And Gates

Status: Closed
Last updated: 2026-06-01

## Source Coverage Audit

| Source | Status | Notes |
| --- | --- | --- |
| User goal | Covered | User wants clear playback/transcode boundaries so multiple Codex terminals can work in parallel. |
| Nako glossary | Covered | `CONTEXT.md` remains the domain language authority. |
| Playback architecture map | Covered | `docs/architecture/PLAYBACK.md` records current shipped and follow-on playback/transcode capabilities. |
| Lane routing | Covered | `docs/architecture/LANES.md` owns long-lived terminal/worktree routing. |
| Workstream index | Covered | `docs/architecture/WORKSTREAM_LINKS.md` links this workstream under Playback And Transcode. |
| ADRs | Covered | ADRs 0038, 0044, 0045, 0046, 0047, 0048, 0049, 0052, and 0053 are the relevant playback/transcode baseline. |
| Related workstreams | Covered | Prior playback/transcode lanes are listed in `CONTEXT.jsonl`. |
| Jellyfin reference | Covered | Local Jellyfin source was inspected for behavior pressure only; do not copy code. |
| Worker prompts | Covered | `WORKER_PROMPTS.md` records first-batch prompts, worktree guidance, and worker note paths. |

## Required Gates

### PTJCH-010 - Interface and lane freeze

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/PLAYBACK.md docs/workstreams/README.md
```

`PTJCH-010` is docs/planning-only. Do not run Rust gates unless code changes are
explicitly added.

### PTJCH-020 - Worker prompt preparation

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening
```

### PTJCH-110 - Playback Capability

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### PTJCH-120 - Transcode Pipeline Capability

```text
cargo nextest run -p nako-transcode pipeline hardware probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### PTJCH-130 - FFmpeg Adapter

```text
cargo nextest run -p nako-transcode ffmpeg hls --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### PTJCH-210 and PTJCH-220 - Coordinated HLS runtime/artifact work

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### PTJCH-310 and PTJCH-390 - Split/closeout

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check
```

If Rust code changed before closeout, rerun the relevant task-specific Rust
gates and record results here.

## Evidence Ledger

### PTJCH-010 - Interface and lane freeze

Status: Done

Evidence collected:

- Initial seam map for Playback Capability, Transcode Pipeline Capability,
  FFmpeg Adapter, HLS Artifact Authority, Playback Runtime, and Artifact I/O
  Policy.
- Architecture lane and workstream index updates.
- Task ledger for first parallel batch and coordinated follow-on batches.

Fresh validation:

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/PLAYBACK.md docs/workstreams/README.md
```

Result: passed on 2026-05-31. `git diff --check` emitted LF/CRLF working-copy
normalization warnings for touched Markdown files and no whitespace errors.

### PTJCH-020 - Worker prompt preparation

Status: Done

Evidence collected:

- First-batch prompts for `PTJCH-110`, `PTJCH-120`, and `PTJCH-130`.
- Suggested per-task worktree/branch names.
- Shared worker rules and stop conditions.
- Task-local worker note directory to reduce shared-doc merge conflicts.

Fresh validation:

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening
```

Result: passed on 2026-05-31. `CONTEXT.jsonl` was also parsed successfully with
29 JSONL entries.

### PTJCH-110 - Playback Capability

Status: Done

Evidence collected:

- Merged commit `0d3bd96f`.
- `evaluate_remux` applies playback output bitrate and resolution constraints.
- Transcode requirement reasons include non-compatible remux evaluation
  blockers.
- Table-driven coverage verifies bitrate cap, resolution cap, and user
  bitrate preference cases fall through from unsupported direct play/remux to
  HLS transcode with explicit reasons.

Fresh validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. `cargo nextest` ran 36 `nako-playback` tests.
`git diff --check` emitted LF/CRLF working-copy warnings for touched Rust files
and no whitespace errors.

### PTJCH-120 - Transcode Pipeline Capability

Status: Done

Evidence collected:

- Merged commit `9f841951`.
- `HardwareAccelerationCapability` exposes available stage-feature lookup.
- Pipeline source compatibility checks requested decode-stage support for
  HEVC/AV1 source inputs.
- QuickSync HEVC tests cover decode-stage present and missing cases.

Fresh validation:

```text
cargo nextest run -p nako-transcode pipeline hardware probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31 before and after rebasing onto main. The focused
nextest gate ran 29 tests after rebase.

### PTJCH-130 - FFmpeg Adapter

Status: Done

Evidence collected:

- Merged commit `bb3835e0`.
- `ffmpeg.rs` is reduced to the builder facade and delegates command planning
  to internal `common`, `remux`, and `hls` modules.
- HLS command planning is split into input, filters, encoders, muxer, seek, and
  sidecar helpers.
- Regression coverage verifies primary HLS output ordering before sidecar
  outputs.

Fresh validation:

```text
cargo nextest run -p nako-transcode ffmpeg hls --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. The full `nako-transcode` nextest gate ran 101
tests after rebasing onto main with `PTJCH-120`.

### First Batch Integration Validation

Status: Done

Fresh validation:

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening docs/architecture/LANES.md docs/workstreams/README.md
cargo nextest run -p nako-playback -p nako-transcode --no-fail-fast
```

Result: passed on 2026-05-31. The combined nextest gate ran 137 tests across
`nako-playback` and `nako-transcode`. `git diff --check` emitted LF/CRLF
working-copy warnings for touched Markdown/JSON files and no whitespace errors.

### PTJCH-210 - HLS Artifact Authority

Status: Done

Evidence collected:

- Merged commit `8ff30ecd`.
- Existing authority flow recorded in `worker-notes/PTJCH-210.md`.
- `HlsRequestVariantPlan` keeps adaptive ladder, media renditions,
  main-output audio shape, and playback generation in request variant
  identity without changing the persisted request-key or artifact path format.
- `HlsArtifactSpec` reconstructs `HlsArtifactManifest` from persisted request
  identity plus the primary playlist path.
- `HlsArtifactManifest::artifact_for_name` now treats the manifest pattern as
  the serveable allow-list for primary playlists, adaptive variant playlists,
  fMP4 init files, main segments, audio sidecars, and subtitle sidecars.
- Legal artifact names outside the manifest return `hls_artifact` not found;
  invalid names still fail validation before path resolution.

Fresh validation:

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. The `nako-transcode` HLS gate ran 59 tests. The
`nako-server` HLS gate ran 71 tests. `cargo fmt --all -- --check` passed.
`git diff --check` emitted LF/CRLF working-copy warnings for touched files and
no whitespace errors.

Notes: an earlier `nako-server` HLS run timed out before producing test
results, and an earlier `nako-transcode` HLS run hit the existing progressive
readiness timing test once; the focused rerun and final full HLS gate both
passed. No PTJCH-220 session lifecycle, admission, cancel, or failure logic was
changed.

### PTJCH-220 - Playback Runtime

Status: Done

Evidence collected:

- HLS supersede candidate discovery and cancellation request ownership now
  lives in the playback runtime control helper instead of being hidden inside
  one HLS reserve branch.
- HLS supersede checks configured resource capacity before cancelling older
  sessions, then waits briefly for a locally cancelled runner to release its
  admission permit before starting the replacement session. Supersede
  candidates include `cancel_requested` sessions because those sessions are
  still active and may still hold local admission permits.
- Browser/renderer HLS playlist paths cancel active playback sessions whose
  linked HLS transcode was superseded, so playback-session state follows the
  runtime cancellation boundary.
- FFmpeg command planning, HLS artifact identity, and manifest allow-lists
  remain owned by `nako-transcode`; this task only changed server runtime
  session/admission/cancellation behavior.
- Added regression coverage for a running HLS playlist session occupying the
  only CPU transcode permit, followed by a seeked HLS request that supersedes
  it without dead-ending on `cpu_transcode` admission.
- Added regression coverage for a `cancel_requested` HLS runner that still
  holds its permit, followed by a seeked HLS request that must signal the
  local cancellation registry and wait for permit release.
- The system playback runtime active-pressure test now uses the same
  platform-aware process-backed HLS readiness timeout as the surrounding
  playback tests, avoiding a Windows full-gate timeout under fake-FFmpeg load.

Fresh validation:

```text
cargo nextest run -p nako-server hls_playlist_playback_seek_supersedes_running_session_without_admission_dead_end --no-fail-fast
cargo nextest run -p nako-server hls_playlist_playback_seek_waits_for_cancel_requested_runner_permit --no-fail-fast
cargo nextest run -p nako-server hls_playlist_playback hls_source_seek_generation_supersedes_active_prior_generation --no-fail-fast
cargo nextest run -p nako-server admin_v1_playback_runtime_reports_active_resource_pressure --no-fail-fast
cargo nextest run -p nako-server hls playback --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
git diff --check
```

Result: passed on 2026-06-01. The focused tracer test failed red first with
`playback resource cpu_transcode is busy`, then passed after bounded supersede
admission and playback-session cancellation were added. The focused HLS
playlist/seek gate ran 4 tests and passed. The active resource-pressure system
test passed focused after its readiness timeout was aligned with existing
Windows playback test helpers. Integration verification also reran the new
seek focused gate with 2 tests passing. The first full `hls playback` attempt
hit a remux cancellation timing failure under parallel test load; that failure
passed on focused rerun, and the second full `hls playback` gate ran 153 tests
and passed. `cargo fmt --all -- --check`, `python -m json.tool`, and
`git diff --check` passed; `git diff --check` emitted LF/CRLF working-copy
warnings only.

### PTJCH-310 - Artifact I/O Decision

Status: Done

Decision:

- HLS artifact I/O pressure is not accepted into this coordination workstream.
- Use the existing `proposed:hls-artifact-io-pressure-enforcement` follow-on
  for disk-sensitive segment read/write pressure, cleanup/throttle policy,
  storage/VFS coordination, and Admin diagnostics.
- Keep FFmpeg command planning and HLS artifact identity in `nako-transcode`;
  keep playback session/admission/supersede/cancel behavior in `nako-server`
  Playback Runtime.

### PTJCH-390 - Closeout

Status: Done

Fresh validation:

```text
python -m json.tool docs/workstreams/playback-transcode-jellyfin-class-hardening/WORKSTREAM.json
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
git diff --check -- docs/workstreams/playback-transcode-jellyfin-class-hardening docs/architecture docs/workstreams/README.md
```

Result: passed on 2026-06-01. No Rust code changed for `PTJCH-310` or
`PTJCH-390`, so no Rust gates were rerun for closeout.
