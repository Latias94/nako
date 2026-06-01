# Generated Artifact Apply Repair Actions

Status: Active
Last updated: 2026-06-02

This workstream turns the read-only Generated Artifact apply recovery queue
into bounded operator repair actions without adding a blind retry executor.

The core rule is that repair must reuse the existing Metadata Authority apply
semantics: redacted planning, target freshness checks, idempotent outcomes,
Admin-only confirmation, and audit records. A new endpoint or Web action is
allowed only when it adds recovery-context guards or operator ergonomics that
the existing apply and bulk apply routes do not already provide.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CONTEXT.jsonl`
- `HANDOFF.md`

Current execution:

- `GAARA-010` opens the lane.
- `GAARA-020` is the first executable task: audit and prove the repair action
  seam before adding mutation behavior.

Boundary:

- no raw retry of prior payloads or plans;
- no mutation without a fresh plan or existing Metadata Authority apply guard;
- no exposure of raw prompts, payloads, Source Locators, paths, tokens,
  secrets, provider responses, or idempotency keys;
- no provider-depth precision work in this lane.
