# TMDB Season Episode Graph Depth

Status: Active
Last updated: 2026-06-02

This workstream extends the closed
`metadata-provider-depth-and-precision` lane from TMDB series -> season graph
preview into TMDB season -> episode graph preview.

The existing boundary remains: graph depth is preview evidence only. Refresh
and Provider Mapping persistence must stay root-only until a future durable
candidate review or Admin confirmation lane deliberately changes that behavior.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `TSEG-010` opens the lane.
- `TSEG-020` added TMDB season -> episode graph preview nodes.
- `TSEG-030` is ready: prove season refresh remains root-only when graph
  preview nodes are present.

Boundary:

- no automatic episode Media Item creation;
- no child Provider Subject or Provider Mapping writes from graph preview;
- no schema, Public Client API, Admin API, or Web confirmation changes;
- no Generated Artifact apply changes;
- no raw provider payload exposure.
