# Addon Notification Bridge

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Addon Event Scheduler And Replay is complete. Nako can now discover due Addon
Event work, retry safely, avoid duplicate in-flight delivery, force replay with
operator intent, and evaluate simple event fact filters before sidecar calls.

The next useful proof is a notification bridge because it is the simplest
event-driven Addon that exercises the new scheduler end to end without needing
new Nako core authority. It turns `library.scanned` and later events into
sidecar-owned notification attempts, while Nako remains responsible for event
facts, delivery scheduling, grants, attempts, and redaction-safe diagnostics.

## Relevant Authority

- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/guides/ADDON_AUTHOR_GUIDE.md`
  - `docs/workstreams/addon-event-scheduler-and-replay/`
  - `F:\SourceCodes\Rust\nako-official-addons\README.md`
- Related workstreams:
  - `docs/workstreams/addon-ecosystem-foundation/`
  - `docs/workstreams/addon-event-scheduler-and-replay/`
  - `docs/workstreams/addon-runtime-and-distribution/`

## Problem

Nako has durable event scheduling, but no real notification-style Addon uses it.
Without a concrete official bridge, the event scheduler remains technically
proven but product-light: operators cannot yet wire scan or library events to
Telegram, Discord, Home Assistant, or another notification sink through the
official Addon path.

The risky mistake would be to put provider matrices, provider credentials, and
message-template behavior into Nako core. Notification delivery should prove
the opposite boundary: Nako emits and schedules events; an Addon sidecar owns
provider-specific formatting and outbound provider calls.

## Target State

When this workstream closes:

- Nako has a named notification bridge follow-on lane with explicit boundaries.
- The first official notification Addon proof subscribes to a durable event and
  receives scheduled deliveries through the Addon Event scheduler.
- Provider-specific notification credentials stay in the Addon sidecar, not
  Nako core.
- Nako-side diagnostics remain redaction-safe and show event delivery evidence
  without notification message bodies or provider secrets.
- The official addon repository has a clear path for adding provider adapters
  behind one Addon/Suite deployment shape.
- Telegram, Discord, Home Assistant, email, or webhook fan-out breadth is split
  into `docs/workstreams/addon-notification-provider-adapters/`.

## In Scope

- Workstream planning and evidence gates for notification bridge.
- A minimal official Addon sidecar proof in `nako-official-addons`.
- Manifest-declared Addon Event Subscription for `library.scanned`.
- Redaction-safe ACK behavior and smoke tests.
- Optional first provider adapter only if it stays narrow and sidecar-owned.
- Nako host tests only when the Addon Event scheduler contract needs a new host
  behavior to support notification bridge.

## Out Of Scope

- Moving notification provider credentials or templates into Nako core.
- Built-in Nako notification provider matrix.
- Watch-state cloud sync.
- MCP media steward behavior.
- Arr-stack integration.
- DLNA/UPnP/WebDAV compatibility.
- Network tunnel provider behavior.
- Addon Manager process supervision, package signing, marketplace hosting, or
  Docker socket control.
- Changing the Addon Protocol unless the proof exposes a real contract gap.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing Addon Event scheduler is enough for first notification proof. | High | AESR-060 closeout gates. | Reopen host scheduler work before provider fan-out. |
| Notification provider calls belong in official addon sidecars. | High | ADR 0003, ADR 0015, ADR 0034. | Core would absorb credentials and provider churn, weakening the sidecar model. |
| `library.scanned` is a sufficient first event. | Medium | Existing metadata scraper event proof and scheduler tests. | Add another event producer only after proving why scan events are insufficient. |
| First slice can ACK without sending to a real third-party provider. | Medium | AEF/AESR used ACK proofs to harden contracts before breadth. | If product value requires a real provider immediately, pick one provider and keep it sidecar-owned. |
| Official addon suite packaging should remain possible. | High | ADR 0034. | A one-off notification sidecar could fragment operator deployment. |

## Architecture Direction

Treat notification bridge as an Addon capability, not as a Nako core subsystem.
Nako owns durable event production, scheduler execution, Addon grants, routing
plans, delivery attempts, forced replay, filters, and safe diagnostics. The
official notification Addon owns message templates, provider credentials,
provider API calls, and provider-specific retries beyond Nako's delivery retry.

The first proof reused the Addon Event Subscription path that AESR closed. It
stays small enough to validate with focused tests in the official addon
repository and a narrow Nako host gate that proves the scheduler can call the
new manifest path. ANB-040 split provider breadth into
`docs/workstreams/addon-notification-provider-adapters/` now that the ACK path
shows the host/addon contract is solid.

## Closeout Condition

This lane can close when:

- the first notification bridge proof receives scheduled Addon Event delivery;
- provider credential ownership remains sidecar-only;
- redaction-safe Nako-side evidence is recorded;
- official addon docs and smoke commands explain the deployment shape;
- focused host and official-addon gates pass;
- provider breadth is split into
  `docs/workstreams/addon-notification-provider-adapters/`.
