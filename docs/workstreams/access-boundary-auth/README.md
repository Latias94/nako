# Access Boundary And Token Authentication

Status: Completed
Last updated: 2026-05-17

This workstream owns M31 inbound HTTP access-boundary hardening. It established
a small bearer-token authentication foundation before future Flutter, web, CLI,
remote access, or tunnel/NAT traversal work depends on unauthenticated server
APIs.

Closeout:

- inbound client/admin auth is separate from addon, webhook, provider,
  automation, storage, and WebDAV outbound integration secrets;
- auth is enabled by default through `TARU_ADMIN_TOKEN`;
- `GET /health` remains public;
- every other HTTP route requires `Authorization: Bearer <token>` when auth is
  enabled;
- auth failures return the M30-compatible `unauthorized` error envelope without
  leaking tokens.

Authoritative files:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
