# Architecture Review Follow-Ups Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

This is a documentation and routing lane. The smallest current validation is:

```bash
git diff --check
```

## Gate Set

### Documentation Consistency Gate

```bash
git diff --check
```

This proves the newly added Markdown and JSON files do not contain whitespace
errors that would block review.

### Routing Consistency Gate

Manual review:

- `DESIGN.md` finding routing table and `TODO.md` task ledger agree.
- `WORKSTREAM.json` authoritative docs list the same core files.
- `HANDOFF.md` names the next routing action.

### Execution Lane Gate

Before any implementation starts, the target execution lane must define its own
focused validation commands. This lane should not use workspace Rust gates
unless it starts changing code, which would be a scope violation.

## Evidence Anchors

- `docs/workstreams/architecture-review-followups/README.md`
- `docs/workstreams/architecture-review-followups/DESIGN.md`
- `docs/workstreams/architecture-review-followups/TODO.md`
- `docs/workstreams/architecture-review-followups/MILESTONES.md`
- `docs/workstreams/architecture-review-followups/WORKSTREAM.json`
- `docs/workstreams/architecture-review-followups/HANDOFF.md`
- `docs/workstreams/README.md`

## Review Gate

Run `review-workstream` before closing this lane or before accepting any
follow-up execution lane as correctly routed.

## Notes

Fresh verification is required before marking any task or lane complete. This
lane currently validates documentation consistency only; implementation lanes
must carry their own Rust checks and `cargo nextest` gates.

## Fresh Evidence

2026-05-18, ARF-020:

- Routing table updated so every finding is Closed, Assigned, or Deferred.
- `WORKSTREAM.json` finding status map now agrees with the routing table.
- `HANDOFF.md` points to ARF-040 and `metadata-merge-policy-unification` as
  the next execution action.

2026-05-18, ARF-040:

- `metadata-merge-policy-unification` opened with complete durable workstream
  docs.
- New lane explicitly names NFO XML preservation and provider breadth as
  non-goals.
- First executable task is MMP-020 characterization before shared policy
  movement.
