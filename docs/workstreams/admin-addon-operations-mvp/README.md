# Admin Addon Operations MVP

Status: Active
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

## First Executable Task

Start with AAO-010: freeze the MVP contract and lifecycle semantics before
adding routes. The first implementation task should not proceed until the
workstream decides whether unregister is terminal soft state or physical
deletion.
