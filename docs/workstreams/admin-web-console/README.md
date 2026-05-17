# Admin Web Console

Status: Proposed
Last updated: 2026-05-17

This workstream owns the planning baseline for Taru's first web-based server
administration console.

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

Taru already has backend boundaries for media libraries, catalog browse,
metadata provider diagnostics, NFO import/export, storage diagnostics,
playback decisions, transcode sessions, webhooks, automation providers, and
sidecar addons. These are operationally powerful but need a coherent admin UI
shape before a front-end generation tool or implementation team can build the
first console.

This lane records the product context, page families, Admin API implications,
and v0-oriented prompt context without committing Taru to a front-end
framework or detailed visual implementation.

## Current Direction

- Treat web as the first-class administration surface.
- Keep playback clients separate from the admin console direction.
- Use Taru's project language from `CONTEXT.md`.
- Provide v0.dev with product and routing context, not implementation lock-in.
- Preserve the distinction between Public Client API and versioned Admin API
  surfaces.
