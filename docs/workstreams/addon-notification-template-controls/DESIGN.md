# Addon Notification Template Controls

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Provider adapters currently send fixed safe summaries. Operators will need some
control over notification text, but raw Addon Event payload values may contain
secret-like or sensitive strings. Template controls must be explicit, small,
and safe by construction.

## Relevant Authority

- ADRs:
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Related workstreams:
  - `docs/workstreams/addon-notification-provider-adapters/`
  - `docs/workstreams/addon-notification-platform-adapters/`

## Problem

Without template controls, all provider messages are fixed. With unsafe template
controls, the bridge could leak raw payload values, provider secrets, URLs, or
implementation details.

## Target State

The lane closes when `nako-notification-bridge` has a minimal safe template
renderer and provider configuration contract that can format notifications from
whitelisted fields only.

## In Scope

- A small renderer for whitelisted tokens.
- Safe fields: event kind, event id, subject kind, subject id, occurred_at,
  attempt, payload key list, and provider id/status.
- Default templates that preserve current behavior.
- Config validation and redaction-safe diagnostics.
- Fixture-backed tests proving raw payload values are unavailable.

## Out Of Scope

- Full templating languages.
- Raw payload value interpolation.
- Nako-managed templates.
- Per-user templates in Admin Web.
- Localization, Markdown rendering, or provider-specific rich layout builders.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| A small token renderer is safer than embedding a general template engine. | High | Current payload redaction requirements. | Stop and write a design decision before adding a general engine. |
| Templates can be sidecar env/config only for the first slice. | High | Existing provider config is env-based. | Split Admin UI/API controls into a later lane. |
| Whitelisted event facts are enough for the first configurable message. | Medium | Current fixed summaries use these facts. | Expand whitelist deliberately with tests. |

## Architecture Direction

Add a small sidecar-owned renderer that substitutes known tokens and rejects
unknown tokens. Provider senders receive rendered safe summaries, not the raw
event payload. Diagnostics expose only template status, not template text when
it may contain operator-authored sensitive literals.

## Closeout Condition

This lane can close when templates are configurable, validated, tested against
leakage, documented, and disabled/default-safe by default.

Closeout result: complete on 2026-05-25. Safe summary templates are implemented
with whitelisted event fact tokens only.
