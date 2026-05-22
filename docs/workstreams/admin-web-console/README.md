# Admin Web Console

Status: Completed
Last updated: 2026-05-19

This workstream owns the completed planning and first implementation baseline
for Nako's first web-based server administration console.

The console is an operator and media-governance surface, not the flagship
playback client. It should help a self-hosted administrator configure media
libraries, inspect metadata provenance, run scans and jobs, diagnose playback
and transcode behavior, manage automation surfaces, and understand server
health.

Authoritative files:

- [DESIGN.md](DESIGN.md)
- [ADMIN_API_MATRIX.md](ADMIN_API_MATRIX.md)
- [V0_CONTEXT.md](V0_CONTEXT.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)

## Why This Lane Exists

Nako already has backend boundaries for media libraries, catalog browse,
metadata provider diagnostics, NFO import/export, storage diagnostics,
playback decisions, transcode sessions, webhooks, automation providers, and
sidecar addons. These are operationally powerful but need a coherent admin UI
shape before a front-end generation tool or implementation team can build the
first console.

This lane recorded the product context, page families, Admin API implications,
v0-oriented prompt context, the first `apps/admin-web` scaffold, and the
initial live/mock Admin API data-source boundary.

## Completed Direction

- Treat web as the first-class administration surface.
- Keep playback clients separate from the admin console direction.
- Use Nako's project language from `CONTEXT.md`.
- Provide v0.dev with product and routing context, not implementation lock-in.
- Preserve the distinction between Public Client API and versioned Admin API
  surfaces.
- Keep the real web app under `apps/admin-web`, separate from Rust server
  crates and Public Client SDK artifacts.
- Use Vite, React, and TypeScript for the first scaffold.
- Keep the `src/adminApi` boundary explicit: live Admin API reads are separate
  from mock or planned data.
- Do not put admin bearer tokens or other secrets into build-time frontend
  environment variables.

## Follow-On Split

The current baseline is complete. Follow-on Admin API contract work belongs in
[admin-api-typescript-contract](../admin-api-typescript-contract/README.md) so
the web app does not grow a long-lived hand-written DTO surface that can drift
from `nako-api`.
