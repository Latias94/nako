# Addon Resource Search Protocol - Handoff

Status: closed. ARSP-010 through ARSP-060 are complete.

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
- Nako intake now has explicit `resource_search_selection` source kinds and a
  host-owned app-service conversion from selected resource-search result/link
  to an acquisition intake candidate.
- The temporary official addon manifest uses `AddonResource::Automation` at
  `/resource-search`.
- This workstream freezes the host-side protocol lane and keeps downloader,
  link-check, and candidate-write behavior separate.

## Follow-Ons

No task remains in this lane. Start a new workstream or issue for one of:

- `nako-official-addons` resource-search manifest migration to
  `resource_search` and `acquisition_search_read`.
- Admin/UI result browsing and explicit select action.
- Link availability/check contract.
- Downloader/external acquisition runner contract.
- Cloud-drive save/transfer authority.
- Secret handling for extraction passwords/codes.

If this closed lane is reopened for audit only, do not add:

- downloader or cloud-drive save behavior,
- link checking,
- resource-search result UI,
- official-addon manifest migration.

## Useful Regression Gate

```bash
cargo nextest run -p nako-server acquisition_intake addon_resource_search --no-fail-fast
```

See `CLOSEOUT.md` for final gates and residual risks.

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not expose raw provider exception text or credentials.
- Do not treat search result URLs as playback stream URLs.
- Follow ADR 0033 if a protocol-version bump becomes necessary.
- Keep `nako-addon-protocol` permissive and host-neutral.
