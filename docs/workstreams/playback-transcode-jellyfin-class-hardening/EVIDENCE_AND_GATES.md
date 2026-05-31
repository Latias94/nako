# Playback Transcode Jellyfin-Class Hardening - Evidence And Gates

Status: Active
Last updated: 2026-05-31

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
