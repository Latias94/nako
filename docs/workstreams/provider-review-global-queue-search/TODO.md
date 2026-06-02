# Provider Review Global Queue Search - TODO

Status: Closed
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] PRGQ-010 [owner=planner] [deps=none] [scope=docs/workstreams/provider-review-global-queue-search,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the global Candidate Review queue/search lane split from ACRN closeout.
  Validation: `python -m json.tool docs/workstreams/provider-review-global-queue-search/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Review: lane must remain read-only and must not reopen item-scoped navigation, batch governance, hierarchy application, or Public Client API.
  Evidence: `DESIGN.md`; `WORKSTREAM.json`; `CONTEXT.jsonl`.
  Handoff: Execution begins at `PRGQ-020`.

## M1 - Read-Only Admin API Global Queue

- [x] PRGQ-020 [owner=codex] [deps=PRGQ-010] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,docs/workstreams/provider-review-global-queue-search]
  Goal: Add a read-only Admin API global Metadata Candidate Review queue route with redaction-safe rows, filters, pagination, and deterministic ordering.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`; focused `nako-db` queue query tests if repository contract changes; `cargo fmt --all -- --check`; `git diff --check`.
  Review: query must not filter broad result sets in HTTP memory, must not write Provider Subject, Provider Mapping, Canonical Metadata, or hierarchy state, and must not expose raw provider payloads or secrets.
  Evidence: `EVIDENCE_AND_GATES.md` (`PRGQ-020 Evidence`).
  Handoff: Continue to `PRGQ-030`.

## M2 - Web Admin Global Queue Navigation

- [x] PRGQ-030 [owner=codex] [deps=PRGQ-020] [scope=web/src/api/admin,web/src/features/admin,web/src/shell,web/src/test,docs/workstreams/provider-review-global-queue-search]
  Goal: Add Web Admin global Candidate Review queue/filter navigation that routes into the existing detail/apply page.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke if a route or navigation mode is added; `git diff --check`.
  Review: Web must present queue rows as discovery/triage only, not as batch apply, status mutation, or hierarchy application.
  Evidence: `EVIDENCE_AND_GATES.md` (`PRGQ-030 Evidence`).
  Handoff: Continue to `PRGQ-040`.

## M3 - Closeout And Follow-On Split

- [x] PRGQ-040 [owner=planner] [deps=PRGQ-030] [scope=docs/workstreams/provider-review-global-queue-search,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split batch governance, related hierarchy application, and provider endpoint depth follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: Admin/Web can triage global Candidate Reviews without broadening Public Client API, hierarchy mutation, or bulk apply.
  Evidence: `CLOSEOUT.md` if closed.
  Handoff: DONE or explicit follow-on split.
