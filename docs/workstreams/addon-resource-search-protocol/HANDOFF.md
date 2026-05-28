# Addon Resource Search Protocol - Handoff

Status: active. ARSP-010 is complete; ARSP-020 is next.

## Current State

- The official resource-search addon has a hardened addon-side architecture and
  a deferred host protocol proposal.
- Nako core currently lacks `AddonResource::ResourceSearch` and
  `AddonScope::AcquisitionSearchRead`.
- The temporary official addon manifest uses `AddonResource::Automation` at
  `/resource-search`.
- This workstream freezes the host-side protocol lane and keeps downloader,
  link-check, and candidate-write behavior separate.

## Next Task

ARSP-020: implement the smallest `nako-addon-protocol` slice.

Expected scope:

- `crates/nako-addon-protocol/src/lib.rs`
- optional docs/tests in the same crate

Expected behavior:

- Add `AddonResource::ResourceSearch` with wire name `resource_search`.
- Add `AddonScope::AcquisitionSearchRead` with wire name
  `acquisition_search_read`.
- Add request/response DTOs for typed resource search.
- Add link type, provider execution status, and provider finality enums.
- Add serde/manifest validation tests with a `resource_search` filter.

Do not include:

- downloader or cloud-drive save behavior,
- acquisition candidate write scope,
- server routing,
- official-addon manifest migration.

## Suggested First Gate

```bash
cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not expose raw provider exception text or credentials.
- Do not treat search result URLs as playback stream URLs.
- Follow ADR 0033 if a protocol-version bump becomes necessary.
- Keep `nako-addon-protocol` permissive and host-neutral.
