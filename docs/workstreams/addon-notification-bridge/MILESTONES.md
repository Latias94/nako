# Addon Notification Bridge — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Scope And Evidence Freeze

Status: Complete.

Exit criteria:

- Notification bridge is split from scheduler/replay closeout.
- Provider credentials and provider fan-out are explicitly outside Nako core.
- First executable task is a small official addon ACK proof.

Primary evidence:

- `docs/workstreams/addon-notification-bridge/DESIGN.md`
- `docs/workstreams/addon-notification-bridge/TODO.md`

## M1 — Official Addon ACK Proof

Status: Complete.

Exit criteria:

- Official notification bridge Addon manifest exists.
- The Addon declares a `library.scanned` event subscription.
- The event route accepts an Addon Event envelope and returns redaction-safe ACK
  output.
- No third-party provider credentials are needed.

Primary gate:

- focused `cargo nextest` gate in `F:\SourceCodes\Rust\nako-official-addons`.

Result: ANB-020 added `nako-notification-bridge` with manifest, health,
diagnostics, `library.scanned` ACK route, checked-in manifest drift test,
sidecar smoke script, Dockerfile, and Compose example. Provider fan-out remains
explicitly out of scope.

## M2 — Host Scheduler Proof

Status: Complete.

Exit criteria:

- Nako can register or use the notification bridge manifest.
- A scheduled `library.scanned` event delivery reaches the sidecar path.
- Delivery attempts and diagnostics remain redaction-safe.

Primary gates:

- focused Nako `addon_event` scheduler tests.
- official addon smoke or fixture test.

Result: ANB-030 added `nako.official.notification-bridge` to the built-in
official Addon catalog, proved catalog resolve, registration, health-check,
routing-plan sync, and scheduled `library.scanned` delivery to the ACK path,
and fixed the official sidecar health `resource_count` response to match its
declared webhook resource.

## M3 — First Provider Or Provider Split

Status: Complete.

Exit criteria:

- The lane either implements one narrow provider adapter or records a split.
- Provider credentials stay sidecar-owned.
- Message body redaction and retry ownership are explicit.

Result: ANB-040 split provider breadth into
`docs/workstreams/addon-notification-provider-adapters/`. No provider adapter
was implemented inside the ACK-only bridge lane.

## M4 — Closeout

Status: Complete.

Exit criteria:

- Evidence is recorded.
- `WORKSTREAM.json` status reflects final state.
- Remaining provider breadth is named as follow-on work.

Result: ANB-050 closed the ACK-only notification bridge lane. Final host, DB,
official sidecar, formatting, JSON, and diff hygiene gates passed. Real provider
adapters are split into
`docs/workstreams/addon-notification-provider-adapters/`.
