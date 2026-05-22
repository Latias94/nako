# nako-api Module Split

Status: Completed

Goal: M46 `nako-api` module split.

This workstream makes `nako-api` a thin API adapter crate with explicit module
boundaries for:

- Public Client API DTO mapping and protocol re-exports.
- Admin/internal server DTOs.
- Metadata diagnostics and maintenance DTOs.
- Extension, webhook, automation, and addon DTOs.

The first slice is intentionally behavior-preserving. Root-level re-exports
remain so current server call sites, OpenAPI generation, TypeScript SDK
generation, and tests do not change wire contracts.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
