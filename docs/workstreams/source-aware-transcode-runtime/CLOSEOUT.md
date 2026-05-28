# Source-Aware Transcode Runtime - Closeout

Status: Completed
Last updated: 2026-05-28

## Final Status

Closed on 2026-05-28. SATR-010 through SATR-100 are complete.

Nako now has a verified source-aware transcode path from ffprobe technical facts
through playback requirements, source-aware hardware pipeline planning, staged
FFmpeg HLS command building, progress metric persistence, Admin/Public evidence
mapping, and progressive HLS segment serving.

## Review Result

No blocking workstream compliance or code-quality findings remain.

Important review notes:

- Public Client playback decisions intentionally expose typed decision/report
  reasons and continue to hide internal `TranscodeRequirement`, host locators,
  raw command lines, and output paths.
- Admin support evidence exposes bounded runtime metrics and source-aware
  readiness reasons; the generated Admin TypeScript contract and mock response
  shape were refreshed.
- HLS running-session serving is file-fact driven: existing segments can be
  streamed, missing running segments return not-ready conflicts, and stale
  sibling `.ts` cleanup keeps the requested segment.

## Verification

Fresh closeout gates:

- `cargo nextest run -p nako-media-probe -p nako-playback -p nako-transcode --no-fail-fast`
  passed: 73 tests.
- `cargo nextest run -p nako-api --no-fail-fast` passed: 61 tests.
- `cargo nextest run -p nako-db transcode_sessions baseline_migration_describes_direct_schema_shape --no-fail-fast`
  passed: 5 tests.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed: 87 tests.
- `cargo nextest run -p nako-server hls --no-fail-fast` passed during SATR-090:
  19 tests.

Final non-test checks:

- `python3 -m json.tool docs/workstreams/source-aware-transcode-runtime/WORKSTREAM.json`
- `cargo fmt --all -- --check`
- `git diff --check`

## Follow-Ons

- Adaptive HLS ladders and bitrate/resolution switching.
- fMP4/CMAF HLS output mode.
- rsmpeg adapter feasibility and typed execution boundary.
- Remote transcode workers.
- Broader per-device codec/profile matrices for hardware decode beyond the
  current first slice.

## Residual Risk

- Hardware compatibility remains intentionally conservative; unsupported
  source facts fall back or fail by policy rather than trying every vendor
  capability combination.
- Progressive HLS currently serves single-variant `.ts` output. It is ready for
  follow-on productization but is not an adaptive streaming stack yet.
