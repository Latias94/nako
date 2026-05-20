# Fearless Architecture Deepening

Status: Active
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

## Current Task

- FAD-020 — Addon Side Effect Module depth.

## Non-Goals

- Provider breadth.
- Native plugin ABI.
- Network traversal.
- Adaptive bitrate ladder implementation.
- AI runtime features.
- Managed Artwork PostgreSQL parity, which remains a separate proposed
  follow-on.
