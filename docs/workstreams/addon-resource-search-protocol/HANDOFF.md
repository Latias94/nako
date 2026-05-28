# Addon Resource Search Protocol - Handoff

Status: active. ARSP-010, ARSP-020, and ARSP-030 are complete; ARSP-040 is next.

## Current State

- The official resource-search addon has a hardened addon-side architecture and
  a deferred host protocol proposal.
- Nako protocol now has `AddonResource::ResourceSearch` and
  `AddonScope::AcquisitionSearchRead`.
- Nako protocol now has typed resource-search request/response DTOs, link
  taxonomy, provider execution status/finality, manifest tests, envelope
  validation coverage, and redaction-safe debug coverage for link URLs and
  passwords.
- Nako addon client now has typed `call_addon_resource_search` helpers that
  reuse the generic resource-call path and enforce read scope plus payload
  schema constants.
- The temporary official addon manifest uses `AddonResource::Automation` at
  `/resource-search`.
- This workstream freezes the host-side protocol lane and keeps downloader,
  link-check, and candidate-write behavior separate.

## Next Task

ARSP-040: define the host service/admin diagnostic seam for calling a
resource-search addon.

Expected scope:

- `crates/nako-server/src/app/addons.rs`
- `crates/nako-api/src/extension.rs`
- focused server/API tests

Expected behavior:

- Expose a host-owned call boundary for resource-search addons.
- Keep limits, granted scopes, addon selection, timeout, retry, and diagnostics
  under Nako policy.
- Return typed/sanitized API DTOs without raw provider exception text.

Do not include:

- downloader or cloud-drive save behavior,
- acquisition candidate write scope,
- server routing,
- official-addon manifest migration.

## Suggested First Gate

```bash
cargo nextest run -p nako-server addon_resource_search --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not expose raw provider exception text or credentials.
- Do not treat search result URLs as playback stream URLs.
- Follow ADR 0033 if a protocol-version bump becomes necessary.
- Keep `nako-addon-protocol` permissive and host-neutral.
