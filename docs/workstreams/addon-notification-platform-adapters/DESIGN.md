# Addon Notification Platform Adapters

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

`nako-notification-bridge` can send generic `http_webhook` notifications, but a
generic webhook is not the same as a named platform integration. Operators need
platform-specific payload shape, configuration, diagnostics, and failure
mapping without moving provider credentials or provider semantics into Nako
core.

## Relevant Authority

- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Related workstreams:
  - `docs/workstreams/addon-notification-bridge/`
  - `docs/workstreams/addon-notification-provider-adapters/`

## Problem

The bridge has only one generic provider, so it cannot prove how named platform
adapters should coexist with the existing ACK path, sidecar-owned credentials,
fixture-backed validation, and redaction-safe diagnostics.

## Target State

The lane closes when one named platform adapter is implemented behind the
existing `library.scanned` event path, disabled by default, fixture-tested, and
documented with no Nako core provider concepts.

The recommended first adapter is `discord_webhook` because it can be validated
with a local HTTP fixture, requires no bot account for the default path, and has
a concrete platform payload shape distinct from generic `http_webhook`.

## In Scope

- Sidecar-owned `discord_webhook` configuration.
- Redaction-safe health and diagnostics facts.
- Fixture-backed send path for `library.scanned`.
- Provider-specific payload shape that excludes raw event payload values.
- Provider failure mapping compatible with existing Addon Event retry.
- Operator docs and default-disabled smoke assertions.

## Out Of Scope

- Telegram, email, Home Assistant, or more than one platform adapter.
- Nako core provider registry, provider credentials, templates, or provider
  retry state.
- Live platform credentials in CI.
- Background sidecar delivery queue.
- Message template controls beyond a fixed safe summary payload.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Discord webhook payloads can be fixture-tested without a live Discord account. | High | `http_webhook` fixture tests already prove outbound HTTP sends. | Pick a different first platform adapter before NPL-020. |
| The existing manifest does not need to change for one more sidecar-owned provider. | High | `http_webhook` did not change host manifest facts. | Add a focused host catalog gate and update Nako docs. |
| Provider credentials should stay in sidecar/operator env. | High | ADRs and closed notification workstreams. | Stop and write an ADR before changing ownership. |

## Architecture Direction

Keep platform adapter code inside `nako-notification-bridge`. Reuse the existing
event route, ACK response envelope, and retry mapping pattern. Add only
redaction-safe provider status to health and diagnostics. Do not add a Nako host
provider abstraction.

## Closeout Condition

This lane can close when `discord_webhook` is implemented, fixture-tested,
documented, disabled by default, and verified with package gates plus any
focused host catalog gate required by manifest drift.

Closeout result: complete on 2026-05-25. `discord_webhook` is implemented and
verified. Additional platform adapters should be split into new lanes.
