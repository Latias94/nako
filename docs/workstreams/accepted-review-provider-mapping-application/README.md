# Accepted Review Provider Mapping Application

Status: Active
Last updated: 2026-06-02

This workstream applies accepted Metadata Candidate Reviews to Provider Subject
and Provider Mapping state through a named backend boundary. It follows the
closed durable review lane and intentionally stays separate from Admin/Web
provider depth governance.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `ARPMA-010` opens the lane.
- `ARPMA-020` is ready: define a read-only Provider Mapping application plan
  for accepted Metadata Candidate Reviews without writing Provider Mapping rows.

Boundary:

- no Admin/Web route in the first slice;
- no Public Client API expansion;
- no raw provider payload, token, header, proxy URL, path, or provider body
  exposure;
- no related graph node hierarchy creation;
- no second Generated Artifact apply executor.
