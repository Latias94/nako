# Public Client Library Browse Query Contract - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Open Lane

- [x] PLBQ-010 [owner=planner] [deps=WDRP-065,WMLP-060] [scope=docs/workstreams/public-client-library-browse-query-contract]
  Goal: Open the library browse/query contract lane from WMLP closeout.
  Validation: `python -m json.tool docs/workstreams/public-client-library-browse-query-contract/WORKSTREAM.json`; `git diff --check -- docs/workstreams/public-client-library-browse-query-contract`.
  Evidence: Initial design, contract readiness, task ledger, and WDRP-065 update.
  Handoff: DONE. Next task is PLBQ-020.

## M1 - Contract Freeze

- [ ] PLBQ-020 [owner=Codex] [deps=PLBQ-010] [scope=crates/nako-client-protocol,crates/nako-api,docs/api/HTTP_API.md,docs/workstreams/public-client-library-browse-query-contract]
  Goal: Freeze route shape, query DTOs, sort/filter enums, access behavior, and SDK expectations for library-scoped browse.
  Validation: protocol/API tests or snapshots; HTTP API docs updated; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no raw DB column query contract or Admin DTO leakage.
  Evidence: `CONTRACT_READINESS.md` and protocol/API tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Server And SDK Implementation

- [ ] PLBQ-030 [owner=Codex] [deps=PLBQ-020] [scope=crates/nako-server,crates/nako-api,sdk/typescript,crates/nako-client]
  Goal: Implement the accepted query contract and regenerate SDKs.
  Validation: focused catalog/library route tests; SDK generation check; `cargo nextest run -p nako-server catalog --no-fail-fast`.
  Review: effective Library Access filters results.
  Evidence: server/API tests and generated SDK diff.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Web Browse Integration

- [ ] PLBQ-040 [owner=Codex] [deps=PLBQ-030] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Wire `/media/library` and selected rails to scoped live browse/query behavior.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Review: readiness states remain for unsupported filters.
  Evidence: data-source and route tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] PLBQ-050 [owner=planner] [deps=PLBQ-040] [scope=docs/workstreams/public-client-library-browse-query-contract]
  Goal: Close the lane with backend/API/SDK/web evidence and remaining browse follow-ons.
  Validation: final backend/frontend gates, JSON validation, and `git diff --check`.
  Review: no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md`.
  Handoff: DONE.
