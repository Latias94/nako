# Admin Candidate Review List Navigation - TODO

Status: Closed
Last updated: 2026-06-02

## M0 - Lane Opening

- [x] ACRN-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-candidate-review-list-navigation,docs/architecture,docs/workstreams/README.md,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Open the Admin Candidate Review list/navigation lane split from AWPDG closeout.
  Validation: `python -m json.tool docs/workstreams/admin-candidate-review-list-navigation/WORKSTREAM.json`; JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`; `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `CONTEXT.jsonl`.
  Handoff: Execution begins at `ACRN-020`.

## M1 - Item-Scoped Admin API List

- [x] ACRN-020 [owner=codex] [deps=ACRN-010] [scope=crates/nako-api,crates/nako-server,crates/nako-metadata,docs/workstreams/admin-candidate-review-list-navigation]
  Goal: Add a redaction-safe Admin API list route for durable Metadata Candidate Reviews scoped to one Media Item.
  Validation: `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: list entries must not expose raw provider payloads, local paths, source locators, tokens, image URLs, raw idempotency keys, or related hierarchy mutations.
  Evidence: `EVIDENCE_AND_GATES.md` (`ACRN-020 Evidence`); `cargo test -p nako-server candidate_review -- --nocapture`; `cargo test -p nako-api admin_contract -- --nocapture`.
  Handoff: Continue to `ACRN-030`.

## M2 - Web Admin List And Navigation

- [x] ACRN-030 [owner=codex] [deps=ACRN-020] [scope=web/src/api/admin,web/src/features/admin,web/src/shell,web/src/test,docs/workstreams/admin-candidate-review-list-navigation]
  Goal: Add Web Admin item-scoped Candidate Review list/navigation that routes into the existing detail/apply page.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke if a route or navigation mode is added; `git diff --check`.
  Review: Web must present list rows as review navigation/triage only, not as hierarchy application or batch apply.
  Evidence: `EVIDENCE_AND_GATES.md` (`ACRN-030 Evidence`); `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; Edge CDP browser smoke; `git diff --check`.
  Handoff: Continue to `ACRN-040`.

## M3 - Closeout And Follow-On Split

- [x] ACRN-040 [owner=planner] [deps=ACRN-030] [scope=docs/workstreams/admin-candidate-review-list-navigation,docs/architecture,docs/GOALS.md,docs/ROADMAP.md]
  Goal: Close the lane or split global Candidate Review queues, batch governance, and related hierarchy application follow-ons.
  Validation: fresh gate evidence in `EVIDENCE_AND_GATES.md`; JSON/JSONL validation; `git diff --check`.
  Review: Admin/Web can navigate durable Candidate Reviews without broadening Public Client API, hierarchy mutation, or bulk apply.
  Evidence: `CLOSEOUT.md`; `EVIDENCE_AND_GATES.md` (`ACRN-040 Evidence`); JSON/JSONL validation; `git diff --check`.
  Handoff: DONE.
