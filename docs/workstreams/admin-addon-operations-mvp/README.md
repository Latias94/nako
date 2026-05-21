# Admin Addon Operations MVP

Status: Completed
Last updated: 2026-05-21

This workstream productizes the Addon administration surface after Addon
Architecture Deepening. The goal is not a full Addon Manager; it is the minimum
operator-facing Addon operations layer needed for a packaged Taru instance:
safe lifecycle controls, health checks, unregister behavior, hosted surface
read models, and diagnostics.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Closeout

Closed on 2026-05-21. AAO-010 through AAO-070 are complete. The shipped
operator surface includes explicit enable/disable lifecycle mutation, terminal
unregister, redaction-safe Addon Health Checks, hosted surface read models,
bounded resource-call diagnostics, token management, and grant management under
`/admin/v1/addons`.

No hidden tail was kept inside this lane. Addon Manager discovery, install,
update, package signing, process supervision, logs, rollback, removal,
full Addon Task runtime, and Addon Event Subscription delivery remain explicit
non-goals for future named workstreams.
