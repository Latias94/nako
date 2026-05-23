# Fearless Future Architecture Refactor

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

M61-M63 gave Nako a strong modular-monolith baseline, PostgreSQL-ready
persistence, deeper runtime seams, and cleaner DTO boundaries. The next risk is
width, not compatibility. Several remaining modules still concentrate
orchestration, policy, and backend detail in ways that will harden when feature
breadth resumes.

This lane keeps the architecture moving before those surfaces become the
default shape of the codebase.

## Goals

- Split the remaining broad runtime, persistence, API, VFS, and inference
  modules into clear owners.
- Remove redundant forwards, dead helpers, and compatibility shims once their
  replacement tests exist.
- Make Docker-backed local validation part of the normal refactor loop.
- Treat `repo-ref/jellyfin` as a behavior and layout reference only.

## Closeout

This lane completed the planned fearless refactor wave:

- Server runtime control planes were split across playback and managed import.
- PostgreSQL persistence moved from one broad backend file into domain-owned
  backend modules.
- Admin API DTOs were split by operational surface with local redaction tests.
- Local VFS path authority, write transactions, apply/link planning, and
  cleanup/restore lifecycle handling now live in focused modules.
- `nako-naming` no longer depends on `nako-core`; library local inference owns
  the Nako-domain mapping.
- Workspace, Docker/container, and PostgreSQL contract gates passed at
  closeout.

## Non-Goals

- No new provider breadth, client UI, plugin ABI, network tunnel
  implementation, adaptive bitrate ladder, or AI model runtime.
- No historical compatibility burden.
- No copied reference source, comments, tests, migrations, or generated code.
- No broad product feature work hidden inside the refactor lane.

## Authoritative Docs

- `docs/workstreams/fearless-future-architecture-refactor/DESIGN.md`
- `docs/workstreams/fearless-future-architecture-refactor/TODO.md`
- `docs/workstreams/fearless-future-architecture-refactor/MILESTONES.md`
- `docs/workstreams/fearless-future-architecture-refactor/EVIDENCE_AND_GATES.md`
- `docs/workstreams/fearless-future-architecture-refactor/HANDOFF.md`
- `docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json`
