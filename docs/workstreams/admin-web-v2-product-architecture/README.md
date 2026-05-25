# Admin Web V2 Product Architecture

Status: Active
Last updated: 2026-05-25

This workstream carries the Admin Web V2 research and design baseline after
the completed `admin-web-console` and `admin-api-typescript-contract` lanes.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [V2_RESEARCH.md](V2_RESEARCH.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)

## Goals

- Convert the first Admin Web scaffold into a route-first product plan.
- Preserve the generated Admin API TypeScript contract boundary.
- Use shadcn/ui-style dashboard composition to ship admin functionality before
  custom product polish.
- Decide which UI stack additions are justified before implementation.
- Define V2 information architecture, page ownership, and validation gates.
- Keep sensitive admin data, Addon Sidecar boundaries, and Public Client API
  separation explicit.

## Non-Goals

- No full UI rewrite in the opening slice.
- No new Admin API mutation semantics without a separate API workstream.
- No Public Client API changes.
- No desktop packaging decision before the web app proves its route and data
  architecture.
