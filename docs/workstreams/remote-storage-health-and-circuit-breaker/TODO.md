# Remote Storage Health And Circuit Breaker - TODO

Status: Active
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

- [x] RSHC-010 [owner=planner] [deps=none] [scope=docs/workstreams/remote-storage-health-and-circuit-breaker,docs/architecture]
  Goal: Freeze problem, target state, non-goals, lane ownership, and first validation gates.
  Validation: `python -m json.tool docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json`; `git diff --check -- docs/workstreams/remote-storage-health-and-circuit-breaker docs/architecture/STORAGE_VFS.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md CONTEXT.md`
  Evidence: `docs/workstreams/remote-storage-health-and-circuit-breaker/DESIGN.md`
  Context: `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`
  Handoff: DONE. Planner opened the lane and made `RSHC-020` the first executable task.

## M1 - Durable Health Contract

- [x] RSHC-020 [owner=codex] [deps=RSHC-010] [scope=crates/nako-core/src/repository/vfs.rs,crates/nako-core/src/vfs*,crates/nako-db/src/**/vfs*,crates/nako-db/src/**/tests*]
  Goal: Add a durable **Storage Backend Health** contract with SQLite and PostgreSQL repository parity.
  Validation: `cargo nextest run -p nako-db storage_backend_health --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: Use `review-workstream` before accepting completion.
  Evidence: repository contract tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`.
  Handoff: DONE. Durable **Storage Backend Health** records, SQLite/PostgreSQL adapters, facade dispatch, baseline schema, and repository contract tests were added. Playback staging, cache repair, and Admin routes were not changed.

## M2 - Runtime Policy Adapter

- [x] RSHC-030 [owner=codex] [deps=RSHC-020] [scope=crates/nako-server/src/app/storage.rs,crates/nako-server/src/app/tests/storage*.rs]
  Goal: Make server storage/VFS runtime paths record health updates and consult an explainable **Storage Circuit Breaker** decision before starting bounded work.
  Validation: `cargo nextest run -p nako-server storage_health --no-fail-fast`; `cargo nextest run -p nako-server storage --no-fail-fast`
  Review: Use `review-workstream` before accepting completion.
  Evidence: runtime policy tests and `EVIDENCE_AND_GATES.md`.
  Context: `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`.
  Handoff: DONE. Server storage runtime paths now persist durable **Storage Backend Health** updates and consult durable **Storage Circuit Breaker** state before bounded storage work. Playback staging, scan scheduling, durable jobs, cache repair, Admin routes, and schema were not changed.

## M3 - Operator Diagnostics And Reset

- [ ] RSHC-040 [owner=codex] [deps=RSHC-030] [scope=crates/nako-api/src/admin/storage.rs,crates/nako-api/src/admin_contract.rs,crates/nako-server/src/app/storage.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests/system.rs]
  Goal: Surface redaction-safe backend health and add an operator reset action that clears circuit-breaker state through the durable contract.
  Validation: `cargo nextest run -p nako-server admin_v1_storage --no-fail-fast`; `cargo nextest run -p nako-server storage_health --no-fail-fast`; `cargo nextest run -p nako-api --no-fail-fast`
  Review: Use `review-workstream` before accepting completion.
  Evidence: Admin API contract and HTTP tests.
  Context: `docs/workstreams/remote-storage-health-and-circuit-breaker/CONTEXT.jsonl`.
  Handoff: Scope expanded on 2026-05-31 after worker BLOCKED report. Preferred access path is `NakoApp::storage()` -> `StorageDiagnosticsAppService`; do not expose raw `NakoDatabase` to Admin HTTP unless the app service path proves insufficient. Regenerate generated contracts only if the existing repo workflow requires it for Admin DTO changes.

## M4 - Verification And Closeout

- [ ] RSHC-050 [owner=planner] [deps=RSHC-040] [scope=docs/workstreams/remote-storage-health-and-circuit-breaker,docs/architecture/STORAGE_VFS.md,docs/architecture/WORKSTREAM_LINKS.md]
  Goal: Run fresh gates, record evidence, and close or split remaining storage/VFS follow-ons.
  Validation: `cargo nextest run -p nako-db storage_backend_health --no-fail-fast`; `cargo nextest run -p nako-server storage_health --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout notes.
  Handoff: Split cache repair, hash escalation, or PostgreSQL runtime harness work if still relevant.
