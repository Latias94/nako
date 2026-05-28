# Subtitle Import Plan Preview

Status: Complete
Last updated: 2026-05-28

## Problem

Nako can search subtitle providers and record short-lived selected subtitle
references, but the host still lacks an explicit import planning boundary. If
the next write slice accepted browser-submitted subtitle content, provider URLs,
target paths, or Source Locators, it would bypass ADR 0051's host-owned import
chain.

## Target State

- Admin can preview a `SubtitleImportPlan` from an existing subtitle selected
  reference and a target media source.
- The plan input names only host-owned IDs and policies: media item/source,
  language, format, sidecar role, conflict policy, and backup policy.
- The plan output is redaction-safe: ids, fingerprints, file names, safe policy
  terms, status/reasons, and idempotency key; no Source Locator, absolute path,
  remote handle, provider URL, subtitle text, artifact id, or backup URI.
- The plan does not download subtitles, persist plan records, write files, or
  call Library File Write apply.

## Scope

- `docs/workstreams/subtitle-import-plan-preview`
- `crates/nako-api`
- `crates/nako-server`

## Non-Goals

- No subtitle file writes.
- No subtitle download execution.
- No durable import/apply record.
- No Library File Write apply endpoint.
- No library subtitle fact refresh.
- No frontend route work; current web/Tauri changes are user-owned and out of
  scope.

## Architecture Direction

Add an import-plan preview endpoint under the existing selected subtitle
resource:

`POST /admin/v1/addons/{addon_id}/subtitle-search/{search_id}/selections/{selection_id}/import-plan`

The endpoint retrieves the raw provider candidate from the host-owned
short-lived selection store, validates the requested media source belongs to
the requested media item, normalizes language/format/role/policies, derives a
sidecar file name from the media source file name, and returns a stable
idempotency key plus redaction-safe plan facts.

The preview intentionally stays in `AddonAppService` for this slice because
the selected candidate is currently owned by that service. A future durable
subtitle import/apply service should move the shared planning code into a
first-party subtitle module once selected references become durable.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Browser replays provider payloads into planning. | High | Endpoint reads candidate from server session only. |
| Plan exposes target path or Source Locator. | High | Response includes only file name and source fingerprint. |
| Plan promises write behavior too early. | Medium | Status/reasons are preview-only and no write/apply endpoint is added. |
| Media item/source mismatch creates unsafe target. | High | Validate source.item_id equals request.media_item_id. |

## Validation Strategy

- `cargo nextest run -p nako-api subtitle_import_plan --no-fail-fast`
- `cargo nextest run -p nako-server addon_subtitle_import_plan --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --cached --check`
