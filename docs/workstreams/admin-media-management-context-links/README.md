# Admin Media Management Context Links

Status: Active
Last updated: 2026-05-29

This workstream owns the frontend route matrix and implementation plan for
permission-gated links between Media Web and Admin Web.

The backend contract already exists: Public Client
`GET /management/context-links` computes safe link descriptors from library,
item, source, or playback-session context. This lane makes the current product
frontend in `web/` consume those descriptors without hard-coding admin
authority in Media UI.

Authoritative docs:

- `DESIGN.md`
- `ROUTE_MATRIX.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

The old `apps/admin-web` console is validation-only. New product work belongs
in `web/` unless a task explicitly says otherwise.
