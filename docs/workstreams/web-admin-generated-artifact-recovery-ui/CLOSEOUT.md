# Web Admin Generated Artifact Recovery UI — Closeout

Closed: 2026-06-02

## Final Status

Closed after `WAGR-020` and `WAGR-030`.

The lane shipped a read-only Web Admin route for inspecting Generated Artifact
apply recovery state without adding mutation controls or exposing raw internal
records.

## Shipped

- Route: `/admin/automation/generated-artifacts/recovery`.
- Navigation from the Generated Artifacts Admin page to the recovery queue.
- Attention filters for `needs_repair`, `needs_review`, `replay_only`,
  `resolved`, and all entries.
- Limit/offset route state and table pagination.
- Summary counters and redaction-safe row facts for artifact, item, outcome,
  batch, plan snapshot counts, error code/message, and updated time.
- Row action that opens the existing Metadata Authority apply plan route; it
  does not execute repair.
- Mobile shell header sizing fix so the new route does not inherit top-bar
  overlap at 390px.
- A narrow aggregate Web bundle budget update from 340 KiB to 341 KiB gzip.

## Gates

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke at desktop 1280px and mobile 390px

## Evidence

- `docs/workstreams/web-admin-generated-artifact-recovery-ui/EVIDENCE_AND_GATES.md`
- `target/codex-smoke/wagr-recovery-desktop.png`
- `target/codex-smoke/wagr-recovery-mobile.png`

## Follow-Ons

- `proposed:generated-artifact-apply-repair-actions`
- `proposed:metadata-provider-depth-and-precision`
- `proposed:admin-settings-api-backed-restoration`

## Residual Risk

- The page is intentionally read-only. Operators can inspect and route to an
  apply plan, but cannot execute a repair action until a separate workstream
  proves idempotency, target freshness, authorization, and audit semantics.
