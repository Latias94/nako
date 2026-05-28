# Web V0 Copy-First TanStack Refactor

Status: Complete
Last updated: 2026-05-28

This lane replaces the current thin `web/` product frontend foundation with a
copy-first import of `repo-ref/nako-admin-web`, then fearlessly refactors that
complete v0-generated product shell into Nako-owned frontend architecture.

The intent is to keep the full product feel and interaction inventory from the
v0 reference while removing Next.js server runtime assumptions, Vercel
assumptions, provider-secret routes, mock-only unsafe controls, third-party
artwork assumptions, and unbounded bundle growth before they harden into the
desktop and browser release line.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
