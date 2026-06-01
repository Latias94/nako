# Generated Artifact Apply Repair Actions

Status: Closed
Last updated: 2026-06-02

This workstream proves how the read-only Generated Artifact apply recovery
queue should lead operators into bounded repair actions without adding a blind
retry executor.

The core rule is that repair must reuse the existing Metadata Authority apply
semantics: redacted planning, target freshness checks, idempotent outcomes,
Admin-only confirmation, and audit records. A new endpoint or Web action is
allowed only when it adds recovery-context guards or operator ergonomics that
the existing apply and bulk apply routes do not already provide. `GAARA-020`
selected Web-only preparation over the existing apply route for the current
product shape. `GAARA-050` closed the lane and split one-click wrapper or UX
polish as explicit follow-ons.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Final execution:

- `GAARA-010` opens the lane.
- `GAARA-020` proved the seam and selected no backend repair wrapper.
- `GAARA-050` closed the lane and split deferred one-click wrapper / UX polish
  work as follow-ons.

Boundary:

- no raw retry of prior payloads or plans;
- no mutation without a fresh plan or existing Metadata Authority apply guard;
- no exposure of raw prompts, payloads, Source Locators, paths, tokens,
  secrets, provider responses, or idempotency keys;
- no provider-depth precision work in this lane.
