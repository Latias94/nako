# Web MVP Live Smoke - Evidence And Gates

Status: Active
Last updated: 2026-06-01

## Smallest Current Repro

```bash
npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx
```

This targeted gate proves the dedicated MVP smoke artifact can execute without
requiring a live server or backend/API contract changes.

## Gate Set

### Targeted Iteration Gate

```bash
npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx
```

### Web Product Gate

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
```

### Workstream Metadata Gate

```bash
python -m json.tool docs/workstreams/web-mvp-live-smoke/WORKSTREAM.json
git diff --check -- docs/workstreams/web-mvp-live-smoke web docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md
```

## MVP Gate 3 Coverage

| Requirement | Evidence path | Status |
| --- | --- | --- |
| `/media` loads through Public Client data source | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| Library list and `/media/library?id=<library_id>` item browse | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| `/media/detail?id=<item_id>&type=<media_type>` detail rendering | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| Browser playback ticket creation for a source | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| Native `VideoPlayer` media/subtitle URL rendering | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| Heartbeat via `playback_session_id` | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |
| No console errors or raw secret/path exposure in checked surfaces | `web/src/test/mvp-live-smoke.test.tsx` | Covered by WMLS-020. |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-06-01 | WMLS-010 | Opened `web-mvp-live-smoke` from MVP Campaign B; linked it to `web-product`; recorded authority, scope, gates, and context manifest. | Pending validation. |
| 2026-06-01 | WMLS-020 | Added dedicated Web MVP live smoke coverage for route surfaces, Public Client playback plan, native `VideoPlayer`, heartbeat, and redaction expectations. Ran `npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx`. | Passed: 1 Vitest file / 2 tests passed. |
| 2026-06-01 | WMLS-030 | Ran the required Web gate set and metadata checks: `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build:budget`, `python -m json.tool docs/workstreams/web-mvp-live-smoke/WORKSTREAM.json`, `CONTEXT.jsonl` parse check, and `git diff --check -- docs/workstreams/web-mvp-live-smoke web docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`. | Passed: 10 Vitest files / 98 tests passed; TypeScript passed; bundle budget passed with media route JS 43.68 KiB raw / 12.06 KiB gzip and total JS 1132.92 KiB raw / 331.87 KiB gzip; JSON/JSONL passed; diff check had only LF/CRLF warnings. |

## Notes

- This lane does not prove backend playback runtime closeout; that remains owned
  by `playback-transcode` and `PTJCH-220`.
- This lane does not promote Tauri/native playback into MVP.
- WMLS-030 is ready for planner review through `integrate-lane-results`.
