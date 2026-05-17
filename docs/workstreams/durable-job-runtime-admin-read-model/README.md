# Durable Job Runtime And Admin Read Model

Status: Completed
Last updated: 2026-05-17

This workstream tracks M54: deepen Taru's durable job runtime and add the
first Admin API read model for job list/filter views.

## Why This Lane Exists

Taru already has a `RuntimeSupervisor`, durable job rows, startup recovery for
unfinished jobs, and Admin API v1 overview support. The next service-side risk
is locality: scan, metadata, and NFO workflows still each manage job
start/succeed/fail persistence themselves, while the Admin Console needs a
stable job list surface.

M54 makes job lifecycle behavior deeper without broadening Public Client API
contracts or adding UI code.

## Outcome

- `taru-server::app::job_runtime` centralizes durable job start/succeed/fail
  handling and typed summary serialization for scan, metadata, and NFO
  workflows.
- `GET /admin/v1/jobs` lists jobs with status, kind, resource class, Media
  Library, Media Source, and pagination filters.
- Admin job list DTOs are redacted list items. Raw job input, summary, and
  error payloads stay out of the list response.
- Public OpenAPI, TypeScript SDK, and `taru-client-protocol` boundaries remain
  unchanged.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
