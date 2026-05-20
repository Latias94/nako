# Fearless Architecture Deepening

Status: Completed
Last updated: 2026-05-20

This workstream owns the next architecture-first fearless refactor pass after
M62 PostgreSQL Production Readiness.

## Purpose

Taru already has a strong modular monolith foundation, but several high-leverage
Modules need deeper Interfaces before future feature breadth hardens caller-side
ordering and cross-domain coupling.

The first execution slice is Addon Side Effect Module depth, followed by Addon
metadata commit atomicity, Library ingestion workflow depth, playback/transcode
identity, hardware diagnostics, search semantics, and test-locality cleanup.

## Authoritative Files

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)

## Closeout

- FAD-090 completed the final verification and closeout for M63.
- Full workspace validation passed with 498 tests run and 19 skipped.
- PostgreSQL opt-in contracts were not run because `TARU_TEST_POSTGRES_URL` was
  not available in this environment.

## Recommended Next Steps

- Continue the already-active `admin-api-typescript-contract` lane when Admin
  Web contract drift is the next priority.
- Pick up `managed-artwork-postgresql-parity` when Managed Artwork needs
  PostgreSQL runtime enablement.
- Open a new named product lane for provider breadth, AI/vector search, network
  traversal, adaptive playback, or client UX rather than reopening this
  architecture-deepening lane.

## Non-Goals

- Provider breadth.
- Native plugin ABI.
- Network traversal.
- Adaptive bitrate ladder implementation.
- AI runtime features.
- Managed Artwork PostgreSQL parity, which remains a separate proposed
  follow-on.
