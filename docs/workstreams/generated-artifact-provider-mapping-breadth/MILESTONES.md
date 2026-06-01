# Generated Artifact Provider Mapping Breadth - Milestones

Status: Active
Last updated: 2026-06-01

## GAPM-M0 - Lane Opened

Outcome: The Provider Mapping breadth lane is ready for implementation.

Deliverables:

- workstream docs;
- active lane registry entry;
- first executable task and campaign.

Exit criteria:

- `WORKSTREAM.json` validates;
- `TODO.md`, `TASKS.jsonl`, and `CAMPAIGNS.jsonl` agree;
- active queue points to `GAPM-020`.

## GAPM-M1 - Read-Only Provider Mapping Plan

Outcome: Admin can request a redacted plan that shows provider mapping effects
for an accepted metadata Generated Artifact without writing Provider Mappings.

Exit criteria:

- plan route remains Admin-only;
- review acceptance still performs no mutation;
- plan includes Provider Subject/Mapping proposal entries and counters;
- unsupported, invalid, existing, stale, and no-op cases are explicit;
- raw artifact JSON, prompts, provider payloads, Source Locators, paths,
  tokens, secrets, and idempotency keys are not exposed.

## GAPM-M2 - Durable Provider Mapping Apply

Outcome: Final metadata apply can create or update accepted Provider Mappings
idempotently through host-owned repositories.

Exit criteria:

- target freshness is rechecked before mutation;
- Provider Subject upsert and Provider Mapping upsert are idempotent;
- durable apply outcome replay does not duplicate mappings;
- mixed metadata field plus provider mapping apply does not produce a partial
  success claim;
- SQLite and PostgreSQL behavior are covered when transaction boundaries
  change.

## GAPM-M3 - Bulk/Admin/Web Surface

Outcome: Bulk apply and Web Admin display provider mapping plan/result facts
without duplicating backend apply logic.

Exit criteria:

- bulk summaries expose Provider Mapping counters and partial results;
- generated Admin contracts are synchronized;
- Web live/fallback behavior remains honest;
- fixture/fallback mode cannot claim live mutation;
- bundle-budget gates pass or any budget change is explicit closeout evidence.

## GAPM-M4 - Closeout

Outcome: The provider mapping breadth lane is verified and either closed or
split into explicit follow-ons.

Exit criteria:

- focused Rust/Web gates pass;
- PostgreSQL parity is recorded if repository transaction behavior changed;
- provider-depth, conflict diagnostics, and operations repair are not hidden in
  this lane;
- architecture and workstream docs reflect the shipped state.
