# Admin Candidate Review List Navigation

Status: Active
Last updated: 2026-06-02

This workstream makes durable Metadata Candidate Reviews discoverable from
Admin/Web surfaces after `admin-web-provider-depth-governance` shipped the
direct review detail/apply route.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `ACRN-010` opens the lane.
- `ACRN-020` added the item-scoped Admin API list/read-model for durable
  Candidate Reviews without exposing raw provider payloads.
- `ACRN-030` added Web Admin item-scoped Candidate Review list/navigation into
  the existing detail/apply page.
- `ACRN-040` is ready: close the lane or split global queue/search, batch
  governance, and related hierarchy application follow-ons.

Boundary:

- item-scoped list/navigation only;
- no related graph node hierarchy mutation;
- no Public Client API expansion;
- no schema migration in the first slice;
- no global queue, batch apply, or broad provider governance in this lane.
