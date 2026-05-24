# Addon Ecosystem Foundation — Milestones

Status: Active
Last updated: 2026-05-25

## M0 — Scope And Authority

Exit criteria:

- ADR 0034 is accepted.
- `CONTEXT.md` defines Addon Package and Addon Suite.
- Workstream docs exist and agree on scope, non-goals, and task order.

Primary evidence:

- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `docs/workstreams/addon-ecosystem-foundation/DESIGN.md`
- `docs/workstreams/addon-ecosystem-foundation/TODO.md`

## M1 — Addon Task Correctness

Exit criteria:

- Addon Task run records include a deterministic request fingerprint.
- Same idempotency key plus same fingerprint returns replay.
- Same idempotency key plus different fingerprint returns a safe conflict.
- SQLite and PostgreSQL schemas/contracts stay aligned.

Primary gates:

- `cargo nextest run -p nako-db addon_task --no-fail-fast`
- `cargo nextest run -p nako-server addon_task --no-fail-fast`

## M2 — Official Catalog Drift Prevention

Exit criteria:

- Built-in official source catalog facts match the official metadata addon
  manifest version, task, config, entry point, and scope shape.
- A focused drift test fails if the hand-written catalog diverges again.
- Install Guide and Manager Plan behavior remains discovery/planning only.

Primary gates:

- `cargo nextest run -p nako-server addon_source_catalog --no-fail-fast`
- official addon manifest tests in `F:\SourceCodes\Rust\nako-official-addons`

## M3 — Addon Event Delivery Runtime

Exit criteria:

- Nako can dispatch durable outbox events to enabled, granted, manifest-declared
  Addon Event Subscriptions.
- Delivery attempts, retries, safe errors, and redacted diagnostics are
  test-visible.
- Follow-up writes still require Addon Token authority.

Primary gates:

- `cargo nextest run -p nako-server addon_event --no-fail-fast`
- `cargo nextest run -p nako-db event --no-fail-fast`

## M4 — Official Event Addon Proof

Exit criteria:

- The official addon repository has a minimal event-driven proof path.
- Suite deployment guidance avoids one Compose service per small Addon.
- The proof does not expand into full notification, watch-sync, MCP, Arr, or
  compatibility feature breadth.

Primary gates:

- focused official addon tests;
- Nako-hosted event delivery smoke when feasible.

## M5 — Closeout And Follow-On Split

Exit criteria:

- All Tier 0 tasks are done or explicitly split.
- Tier 1/2/3 feature work has named follow-ons or clear deferral.
- Final evidence is recorded.
- `WORKSTREAM.json` status is updated.
