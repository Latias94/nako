# Addon Notification Provider Live Smoke

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Fixture tests prove provider behavior without secrets, but release operators
still need an explicit local workflow for verifying a real provider endpoint.
That workflow must be opt-in, redaction-safe, and excluded from default CI.

## Relevant Authority

- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Related workstreams:
  - `docs/workstreams/addon-notification-provider-adapters/`
  - `docs/workstreams/addon-notification-platform-adapters/`

## Problem

There is no named live smoke path for provider delivery. Operators may invent
ad hoc commands that leak secrets into shell history, logs, or CI.

## Target State

The lane closes when the official notification bridge has an opt-in live smoke
script and documentation that validates real provider delivery only when local
environment variables are provided.

## In Scope

- Opt-in PowerShell live smoke script.
- Explicit required env vars and skip behavior.
- Redaction-safe console output.
- Local sidecar start or externally supplied sidecar URL.
- Documentation that CI must not run live provider smoke by default.

## Out Of Scope

- Storing live secrets in repo, CI, or workstream docs.
- Requiring a live provider account for package gates.
- Provider SDKs.
- Admin Web live smoke UI.
- Expanding provider functionality.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Live smoke should be a script, not a Rust test, to avoid accidental CI execution. | High | Existing local smoke is PowerShell and explicit. | Add stronger skip guards if CI still picks it up. |
| `http_webhook` is the safest first live smoke target. | High | It only needs a URL and optional shared secret. | Add platform-specific live smoke later. |
| The script can avoid printing target URLs and secrets. | High | Existing smoke asserts redacted output. | Fail closed if safe output cannot be guaranteed. |

## Architecture Direction

Keep live smoke outside default cargo tests. Add a dedicated script under
`addons/notification-bridge` that fails unless the operator explicitly opts in
with environment variables. Reuse the existing sidecar event route and assert
safe ACK/diagnostics output.

## Closeout Condition

This lane can close when live smoke is documented, skipped by default, locally
runnable with explicit env vars, and tested for secret-safe script parsing and
default skip behavior.

Closeout result: complete on 2026-05-25. `smoke.live.ps1` exists, parses, skips
by default, and is documented as local-only opt-in validation.
