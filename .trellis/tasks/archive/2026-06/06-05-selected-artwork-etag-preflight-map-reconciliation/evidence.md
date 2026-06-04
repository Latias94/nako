# Evidence

## Changes

- Updated `docs/architecture/CONTROL_PLANE.md` to record metadata-derived
  selected artwork ETag preflight as shipped.
- Updated `docs/architecture/LIBRARY_PIPELINE.md` to remove metadata-only ETag
  preflight from the open artwork delivery cache scope and keep the remaining
  cache follow-ons explicit.
- Added this task's PRD and research evidence.

## Validation

- `git diff --check` passed with only Git LF/CRLF working-copy warnings.
- `python ./.trellis/scripts/task.py validate 06-05-selected-artwork-etag-preflight-map-reconciliation`
  passed.
- Focused grep over `docs/architecture/CONTROL_PLANE.md` and
  `docs/architecture/LIBRARY_PIPELINE.md` confirms the active maps now describe
  metadata-derived selected artwork ETag preflight as shipped while keeping
  weak/wildcard validators, invalidation, derivative generation, and placeholder
  work as follow-ons.
