# nako-api Module Split Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M46 is complete. `nako-api` has explicit module boundaries and a thin root
compatibility facade.

## Follow-ons

- Server call sites may later import from `nako_api::public_client`,
  `nako_api::admin`, `nako_api::metadata_diagnostics`, and
  `nako_api::extension` directly, but root-level imports are intentionally kept
  for compatibility.
- No DTO ownership was moved to `nako-client-protocol` in this slice.
- Recommended next goal: NFO Round Trip preservation model before library file
  write/link policy work.

## Validation

Start with:

```powershell
cargo check -p nako-api --tests
cargo nextest run -p nako-api --no-fail-fast
```
