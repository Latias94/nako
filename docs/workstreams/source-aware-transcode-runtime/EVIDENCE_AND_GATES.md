# Source-Aware Transcode Runtime - Evidence And Gates

Status: Completed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-media-probe --no-fail-fast
```

This first gate proves source media technical facts can be parsed from ffprobe
payloads before playback and transcode planning consume them.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-media-probe --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### API And Storage Gates

```bash
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-db --no-fail-fast
```

Run these when a task changes wire DTOs, generated clients, media probe
persistence, or transcode session schema.

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
```

Use focused package gates above for closeout unless this lane changes shared
workspace contracts broadly enough to require `cargo nextest run --workspace`.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in the task
handoff.

## Evidence Anchors

- `docs/workstreams/source-aware-transcode-runtime/DESIGN.md`
- `docs/workstreams/source-aware-transcode-runtime/TODO.md`
- `docs/workstreams/source-aware-transcode-runtime/MILESTONES.md`
- `docs/workstreams/source-aware-transcode-runtime/HANDOFF.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`

## Evidence Log

- 2026-05-27 SATR-010: Opened the workstream and proposed ADR 0049. Validation
  passed with `python3 -m json.tool docs/workstreams/source-aware-transcode-runtime/WORKSTREAM.json`.
- 2026-05-27 SATR-020: Added source-aware stream technical facts to
  `MediaStreamInfo.technical` and mapped ffprobe profile, level, pixel format,
  bit depth, frame rate, color/HDR, disposition, rotation, and channel layout
  fields. Validation passed with `cargo nextest run -p nako-media-probe
  --no-fail-fast`, `cargo nextest run -p nako-core --no-fail-fast`, and
  `cargo nextest run -p nako-library --no-fail-fast`.
- 2026-05-27 SATR-030: Persisted stream technical facts through
  `media_streams.technical_json` for SQLite and PostgreSQL baselines.
  Validation passed with `cargo nextest run -p nako-db
  nako_database_sqlite_round_trips_media_probe_results
  sqlite_scan_commit_contract_writes_full_source_unit_and_resolves_failure
  --no-fail-fast` and `cargo nextest run -p nako-db
  baseline_migration_describes_direct_schema_shape --no-fail-fast`.
- 2026-05-27 SATR-040: Added internal source-aware `TranscodeRequirement` to
  playback decisions with selected streams, output constraints, subtitle
  strategy, source technical facts, and transcode reasons. Validation passed
  with `cargo nextest run -p nako-playback --no-fail-fast`,
  `cargo nextest run -p nako-api --no-fail-fast`, and `cargo nextest run -p
  nako-server playback --no-fail-fast`.
- 2026-05-28 SATR-050: Kept Public playback decisions redaction-safe while
  exposing typed decision/report reasons, and mapped Admin source-aware
  readiness/support evidence to include hardware fallback reasons and
  redaction-safe runtime metrics. Refreshed the Admin TypeScript contract and
  admin-web mock response shape. Validation passed with `cargo nextest run -p
  nako-api --no-fail-fast`.
- 2026-05-27 SATR-060: Added source facts to `TranscodePipelineRequest` and
  connected HLS execution policy planning to stored ffprobe streams. The
  planner now emits typed fallback/failure reasons when requested VAAPI, QSV, or
  VideoToolbox hardware decode is incompatible with source video codec or bit
  depth. Validation passed with `cargo nextest run -p nako-transcode pipeline
  --no-fail-fast`, `cargo nextest run -p nako-api --no-fail-fast`, and `cargo
  nextest run -p nako-server playback --no-fail-fast`.
- 2026-05-27 SATR-070: Split HLS FFmpeg command construction into global,
  device/input, stream map, filter graph, video encoder, audio encoder,
  subtitle, and muxer parts. Added command-plan coverage for QSV argument
  ordering, omitted subtitle strategy, and minimum HLS segment time. Validation
  passed with `cargo nextest run -p nako-transcode ffmpeg --no-fail-fast`,
  `cargo nextest run -p nako-transcode --no-fail-fast`, and `cargo nextest run
  -p nako-server playback --no-fail-fast`.
- 2026-05-27 SATR-080: Added redaction-safe transcode runtime metrics,
  FFmpeg `-progress pipe:1` parsing, HLS runner metric capture, SQLite/Postgres
  baseline persistence via `runtime_metrics_json`, Admin support evidence
  exposure, and server HLS persistence assertions. Validation passed with
  `cargo nextest run -p nako-transcode --no-fail-fast`, `cargo nextest run -p
  nako-api --no-fail-fast`, `cargo nextest run -p nako-server playback
  --no-fail-fast`, `cargo nextest run -p nako-db transcode_sessions
  baseline_migration_describes_direct_schema_shape --no-fail-fast`, and `cargo
  nextest run -p nako-core --no-fail-fast`.
- 2026-05-28 SATR-090: Prepared HLS serving for progressive output by allowing
  running HLS sessions to serve already-written segments, returning conflict
  for still-missing running segments, waiting once when transcode throttling is
  enabled, and pruning stale sibling `.ts` segments without deleting the
  requested segment or playlist. Validation passed with `cargo nextest run -p
  nako-server hls --no-fail-fast`.
- 2026-05-28 SATR-100: Closeout review found no blocking workstream compliance
  or code-quality issues. Fresh verification passed with `cargo nextest run -p
  nako-media-probe -p nako-playback -p nako-transcode --no-fail-fast`, `cargo
  nextest run -p nako-api --no-fail-fast`, `cargo nextest run -p nako-db
  transcode_sessions baseline_migration_describes_direct_schema_shape
  --no-fail-fast`, and `cargo nextest run -p nako-server playback
  --no-fail-fast`. Final formatting and diff checks are recorded in
  `CLOSEOUT.md`.

## Notes

- Do not claim hardware runtime support from static inventory alone; each task
  must show what the gate proves.
- Do not record raw host paths or full FFmpeg commands in Admin/Public evidence.
- Fresh verification is required before marking a task, Codex goal, or lane
  complete.
