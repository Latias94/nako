# Douban Subject Kind Precision

Status: Active
Last updated: 2026-06-02

This workstream turns the proposed Douban provider precision follow-on into a
focused execution lane.

The current Douban adapter uses movie search and movie detail endpoints, but it
advertises Series, Season, and Episode support. That makes provider diagnostics
too broad for future Admin governance and durable candidate review.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `DSKP-010` opens the lane.
- `DSKP-020` is ready: tighten Douban media and Provider Subject capability
  claims around endpoint-backed movie behavior.

Boundary:

- no schema, Public Client API, Admin API, or Web changes;
- no Generated Artifact apply changes;
- no hierarchy graph preview;
- no automatic Media Item creation;
- no child Provider Subject or Provider Mapping writes;
- no raw Douban response, API key, header, or proxy URL exposure.
