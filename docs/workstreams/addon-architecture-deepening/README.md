# Addon Architecture Deepening

Status: Completed
Last updated: 2026-05-21

This workstream owns the fearless Addon architecture pass after the 2026-05-21
Addon review. It deepens the current Addon Sidecar, Addon Protocol, Addon
Token, Library-Scoped Addon Grant, Addon Side Effect, Protected Write, Library
File Write, and Admin Addon API seams before new Addon breadth hardens shallow
Interfaces.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Why This Lane Exists

Taru's Addon direction is sound: Addons are HTTP Addon Sidecars, strong Addon
Side Effects enter through Taru-owned APIs, and Addon Tokens are separate from
administrator bearer tokens. The remaining risk is Module depth. Several
Interfaces still expose too much implementation knowledge to callers, tests, or
Addon authors:

- Addon Side Effect lifecycle rules are spread across intake, apply routing,
  per-permission Adapters, and database commit code.
- Protected Write payload contracts live inside private server structs while
  Addon-facing requests use `serde_json::Value`.
- The Addon Manifest does not yet express the full Addon Protocol language from
  `CONTEXT.md`.
- Library File Write currently means a narrow NFO Export path, but the domain
  Interface will need subtitle, artwork-export, and sidecar-asset behavior.
- Addon idempotency replay is key-only and should distinguish true replay from
  conflicting reuse.
- Addon administration should live under `/admin/v1/addons`; the historical
  root `/addons` management surface must be removed because Taru has no
  compatibility obligation yet.

## First Executable Task

Start with AAD-010: freeze the shipped Addon authority, update stale ADR
statuses, and document the exact constraints that later code tasks must
preserve.
