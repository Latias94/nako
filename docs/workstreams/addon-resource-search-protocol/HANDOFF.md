# Addon Resource Search Protocol - Handoff

Status: active. ARSP-010 and ARSP-020 are complete; ARSP-030 is next.

## Current State

- The official resource-search addon has a hardened addon-side architecture and
  a deferred host protocol proposal.
- Nako protocol now has `AddonResource::ResourceSearch` and
  `AddonScope::AcquisitionSearchRead`.
- Nako protocol now has typed resource-search request/response DTOs, link
  taxonomy, provider execution status/finality, manifest tests, envelope
  validation coverage, and redaction-safe debug coverage for link URLs and
  passwords.
- The temporary official addon manifest uses `AddonResource::Automation` at
  `/resource-search`.
- This workstream freezes the host-side protocol lane and keeps downloader,
  link-check, and candidate-write behavior separate.

## Next Task

ARSP-030: add a typed `nako-addon-client` helper over existing resource calls.

Expected scope:

- `crates/nako-addon-client/src/lib.rs`
- focused tests in the same crate

Expected behavior:

- Call `AddonResource::ResourceSearch` through the existing generic resource
  call path.
- Require `AddonScope::AcquisitionSearchRead` and preserve existing manifest,
  scope, retry, timeout, protocol-version, and safe-error behavior.
- Return the typed resource-search response DTO.

Do not include:

- downloader or cloud-drive save behavior,
- acquisition candidate write scope,
- server routing,
- official-addon manifest migration.

## Suggested First Gate

```bash
cargo nextest run -p nako-addon-client resource_search --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not expose raw provider exception text or credentials.
- Do not treat search result URLs as playback stream URLs.
- Follow ADR 0033 if a protocol-version bump becomes necessary.
- Keep `nako-addon-protocol` permissive and host-neutral.
