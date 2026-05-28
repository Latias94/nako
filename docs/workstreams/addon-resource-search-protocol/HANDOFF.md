# Addon Resource Search Protocol - Handoff

Status: active. ARSP-010 through ARSP-040 are complete; ARSP-050 is next.

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
- Nako server/API now has a dedicated admin resource-search diagnostic route
  that applies host limits and returns redaction-safe counts/provider summaries
  without raw result payloads.
- The temporary official addon manifest uses `AddonResource::Automation` at
  `/resource-search`.
- This workstream freezes the host-side protocol lane and keeps downloader,
  link-check, and candidate-write behavior separate.

## Next Task

ARSP-050: record or implement selected-result conversion into acquisition
intake candidates.

Expected scope:

- `crates/nako-core/src/acquisition_intake.rs`
- `crates/nako-server/src/app`
- focused acquisition-intake/server tests

Expected behavior:

- Convert a selected resource-search result/link into a host-owned
  `AcquisitionIntakeCandidate` only after explicit selection.
- Preserve redacted source references and diagnostics.
- Keep downloader execution, link checking, and cloud-drive save outside this
  lane unless a new task explicitly scopes them.

Do not include:

- downloader or cloud-drive save behavior,
- acquisition candidate write scope,
- server routing,
- official-addon manifest migration.

## Suggested First Gate

```bash
cargo nextest run -p nako-server acquisition_intake addon_resource_search --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not expose raw provider exception text or credentials.
- Do not treat search result URLs as playback stream URLs.
- Follow ADR 0033 if a protocol-version bump becomes necessary.
- Keep `nako-addon-protocol` permissive and host-neutral.
