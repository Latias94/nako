# OpenAPI And Public Client SDK Contract

Status: Completed
Last updated: 2026-05-17

This workstream owns M32: the first machine-readable Public Client API
contract for Nako. It turned the M29 public protocol DTOs, M30 version/error
contract, and M31 bearer-auth boundary into an OpenAPI v1 artifact that future
Flutter, web, CLI, and SDK work can consume.

Closeout:

- `nako-client-protocol` owns playback session response wire DTOs;
- playback session responses no longer expose server-local output paths;
- `nako-api` generates OpenAPI v1 JSON through
  `cargo run -p nako-api --example emit-openapi`;
- checker tests verify public route coverage, auth/version/error/pagination
  semantics, and internal/admin leakage rejection.

Authoritative files:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
