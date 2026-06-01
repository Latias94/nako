# Generated Artifact Apply Operations Repair - Closeout

Status: Closed
Closed: 2026-06-02
Task: GAOR-040

## Shipped

Nako now has a read-only Admin recovery surface for Generated Artifact apply
operations:

- one-artifact metadata apply outcomes are queryable through Admin list/detail
  routes and Web Admin read-model mapping;
- apply outcomes and bulk batch terminal items flow into a recovery queue
  grouped by operator attention: `needs_repair`, `needs_review`,
  `replay_only`, and `resolved`;
- recovery classification lives in `nako-core` so SQLite and PostgreSQL
  adapters do not duplicate repair semantics;
- the Admin recovery route
  `/admin/v1/automation/generated-artifact-apply-recovery` stays redaction-safe
  and does not expose raw artifact payloads, prompts, Source Locators, host
  paths, tokens, secrets, or idempotency keys;
- generated Admin TypeScript contracts and `web/` Admin data-source mappings
  are synchronized for the recovery queue.

## Final Evidence

Fresh verification on 2026-06-02:

```bash
cargo nextest run -p nako-api admin_contract generated_artifact_metadata_apply_recovery_response_classifies_repair_state --no-fail-fast
cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic --no-fail-fast
cargo nextest run -p nako-server admin_generated_artifact_metadata_apply_v1_commits_and_replays_redacted_result --no-fail-fast
cargo check -p nako-server --tests
cargo fmt --all -- --check
npm --prefix web run check
npm --prefix web run build:budget
python -m json.tool docs/workstreams/generated-artifact-apply-operations-repair/WORKSTREAM.json
git diff --check
```

Results:

- API/Admin contract and recovery DTO tests passed;
- SQLite repository contract test passed; PostgreSQL code was compile-checked,
  but the PostgreSQL runtime contract harness was not run because
  `NAKO_TEST_POSTGRES_URL` was not set;
- server Admin HTTP test passed for commit, idempotent replay, recovery queue
  read, and redaction;
- Web TypeScript check and bundle budget passed;
- workstream JSON and repository diff checks passed.

## Review Result

No blocking workstream compliance findings remain:

- `GAOR-020` audited the persistence/read-model gap and selected a read-first
  repair boundary;
- `GAOR-030` shipped the first Admin-facing repair surface as a read-only
  recovery queue;
- mutation scope was intentionally split because bounded repair actions need a
  separate idempotency and freshness proof.

No blocking code-quality findings remain:

- repository traits expose a recovery read seam instead of hiding queue logic
  inside HTTP handlers;
- DB adapters map rows and deserialize plans, while domain recovery
  classification stays in `nako-core`;
- Admin DTOs and Web read models preserve replay-vs-repair semantics without
  raw internal record leakage.

## Follow-Ons

- `proposed:web-admin-generated-artifact-recovery-ui`: render the recovery
  queue in Web Admin with filters, detail links, empty states, and browser
  smoke evidence.
- `proposed:generated-artifact-apply-repair-actions`: add bounded,
  confirmation-backed repair actions that reuse existing Metadata Authority
  apply semantics, target freshness checks, and durable idempotent outcomes.
- `proposed:metadata-provider-depth-and-precision`: improve provider identity
  precision separately from operations repair.

## Residual Risks

- The shipped surface is read-only; it helps operators identify repair work but
  does not yet execute a repair mutation.
- Web has a data-source/read-model contract for the recovery queue, but no
  dedicated route-level UI has been added in this lane.
- PostgreSQL recovery SQL is compile-covered and mirrors SQLite behavior, but
  runtime parity should be proven when a PostgreSQL harness is available.
