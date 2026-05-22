# Admin Web Addon Operations

Status: Completed
Last updated: 2026-05-22

This workstream productizes Taru's completed Admin Addon Operations MVP into
the first Admin Web Console Addons surface.

The backend already supports safe **Addon Sidecar** registration, lifecycle
mutation, terminal unregister, token/grant management, **Addon Health Check**,
manifest surface read models, and redaction-safe resource-call diagnostics.
The current frontend still marks Addons as planned. This lane closes that gap
without turning Taru into an **Addon Manager**.

Authoritative files:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)

## Closeout

All planned tasks are complete. The Admin Web Console now has a live-capable
Addon Operations surface backed by the generated Admin API TypeScript
contract, safe mock fallback, and data-source mediated lifecycle/diagnostic
actions.

## Scope

- Extend the generated Admin API TypeScript contract with Addon Operations
  DTOs and route constants.
- Deepen `apps/admin-web/src/adminApi` so Addons are a live-capable data
  source with safe mock fallback.
- Add an Admin Web Addons panel/page slice for Addon list/detail facts,
  status, grants, tokens, surfaces, health checks, and resource-call
  diagnostics.
- Wire safe lifecycle actions only where backend semantics already exist.
- Keep **Addon Hosted Pages** clearly external and untrusted.

## Non-Goals

- No **Addon Manager** discovery, install, update, marketplace, package
  signing, Docker socket control, or sidecar process supervision.
- No new Addon Protocol behavior.
- No OAuth-first authorization.
- No embedded trusted frontend plugin runtime.
- No full **Addon Task** execution or **Addon Event Subscription** delivery
  runtime.
