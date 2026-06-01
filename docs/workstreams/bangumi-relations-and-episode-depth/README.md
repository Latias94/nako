# Bangumi Relations And Episode Depth

Status: Active
Last updated: 2026-06-02

This workstream turns the proposed Bangumi provider-depth follow-on into a
focused execution lane.

Bangumi is anime-first and subject-oriented. The current Nako adapter can
search/fetch subject metadata, but it also advertises season and episode
support before it uses Bangumi's relation or episode endpoints. That is too
broad for future Admin diagnostics and client-facing provider capability
claims.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `BRED-010` opens the lane.
- `BRED-020` tightened Bangumi provider capability claims around
  endpoint-backed behavior before adding graph depth.
- `BRED-030` added endpoint-backed Bangumi episode graph preview for series
  fetches without changing persistence.
- `BRED-040` proved refresh remains root-only when graph preview nodes are
  present.
- `BRED-050` is ready for lane closeout or explicit follow-on split.

Boundary:

- no schema, Public Client API, Admin API, or Web changes;
- no Generated Artifact apply changes;
- no automatic episode Media Item creation;
- no child Provider Subject or Provider Mapping writes from graph preview;
- no raw Bangumi response, token, header, or proxy URL exposure.
