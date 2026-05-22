# Admin Operations Read Models

Status: Completed
Last updated: 2026-05-18

This workstream tracks M57-M59: the next batch of read-only Admin API v1
operational diagnostics needed by the admin web console.

## Goals

- M57: add a redacted event outbox list/filter read model.
- M58: add redacted storage staging/cache diagnostics.
- M59: add sanitized server configuration diagnostics.
- Keep these surfaces admin-owned and out of the Public Client API,
  public OpenAPI, generated TypeScript SDK, Rust client SDK, and
  `nako-client-protocol`.

## Non-Goals

- No frontend implementation.
- No admin mutation routes for event retry, staging cleanup, or config edits.
- No Admin OpenAPI generation.
- No Public Client API route or DTO changes.
- No `nako-client-protocol` changes.

## Authoritative Files

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)
