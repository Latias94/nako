# taru-api Module Split Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M46 is complete. `taru-api` has explicit module boundaries and a thin root
compatibility facade.

## Follow-ons

- Server call sites may later import from `taru_api::public_client`,
  `taru_api::admin`, `taru_api::metadata_diagnostics`, and
  `taru_api::extension` directly, but root-level imports are intentionally kept
  for compatibility.
- No DTO ownership was moved to `taru-client-protocol` in this slice.
- Recommended next goal: NFO Round Trip preservation model before library file
  write/link policy work.

## Validation

Start with:

```powershell
cargo check -p taru-api --tests
cargo nextest run -p taru-api --no-fail-fast
```
