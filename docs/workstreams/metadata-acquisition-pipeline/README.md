# Metadata Acquisition Pipeline

Status: completed 2026-05-25

This lane turned scan-time metadata acquisition into a configurable pipeline for
Media Libraries. The first implementation keeps the existing NFO import
behavior, preserves suggestion-only Addon bulk scrape by default, and adds an
explicit writeback path that lets approved Addons submit Canonical Metadata
through Nako-owned Addon Side Effects.

## Goals

- Move scan-time metadata acquisition out of the library scan orchestration
  code and into a focused application service.
- Keep NFO import as one pipeline phase instead of the only hard-coded metadata
  path.
- Make scan-triggered Addon Bulk Metadata Scrape configurable and explicit
  about whether metadata writeback is requested.
- Prove that Addon writeback enters the existing Canonical Metadata merge and
  catalog/search commit path through `/addon/v1/side-effects`.

## Non-Goals

- Do not interpret arbitrary Addon Task output as trusted metadata writes.
- Do not add new third-party provider breadth in this lane.
- Do not add automatic artwork publication, UI management, or scheduler breadth.
- Do not change NFO XML preservation or sidecar write policies.

## References

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
