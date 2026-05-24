# Addon Ecosystem Foundation — TODO

Status: Active
Last updated: 2026-05-25

## M0 — Scope And Authority

- [x] AEF-010 [owner=planner] [deps=none] [scope=docs]
  Goal: Record the Addon Package / Addon Suite deployment decision and open the
  ecosystem foundation lane.
  Validation: ADR 0034, `CONTEXT.md`, `DESIGN.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree.
  Evidence: `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
  Handoff: Planner owns the initial authority freeze.

## M1 — Addon Task Correctness

- [x] AEF-020 [owner=codex] [deps=AEF-010] [scope=crates/nako-core,crates/nako-db,crates/nako-server]
  Goal: Add deterministic Addon Task request fingerprints and reject
  mismatched idempotency-key reuse.
  Validation:
  `cargo nextest run -p nako-db sqlite_event_addon_automation_contract_addon_task_run_idempotency_fingerprint --no-fail-fast`
  and
  `cargo nextest run -p nako-server addon_task_run_runtime_is_host_owned_and_reports_progress_result --no-fail-fast`.
  Review: Review idempotency behavior, SQLite/PostgreSQL parity, and safe
  public error mapping before accepting completion.
  Evidence: `crates/nako-db/src/sqlite/addon_tasks.rs`,
  `crates/nako-db/src/postgres/addon_tasks.rs`,
  `crates/nako-db/src/contract_tests.rs`,
  `crates/nako-core/src/addon_task.rs`.
  Handoff: Preserve compatibility only where a live migration requires it; Nako
  has no deployed users, so prefer clean schemas.

## M2 — Official Catalog Drift Prevention

- [x] AEF-030 [owner=codex] [deps=AEF-010] [scope=crates/nako-server,docs,official-addon-repo]
  Goal: Prevent built-in official addon catalog descriptors from drifting from
  official addon manifest/task/config facts.
  Validation: `cargo nextest run -p nako-server addon_source_catalog --no-fail-fast`,
  `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`, and
  official addon focused manifest tests.
  Review: Confirm the catalog remains a discovery surface, not package
  supervision or hidden lifecycle execution.
  Evidence: `crates/nako-official-addon-catalog`,
  `crates/nako-server/src/app/addons.rs`,
  `F:\SourceCodes\Rust\nako-official-addons\crates\nako-metadata-scraper\src\manifest.rs`.
  Handoff: Prefer generated/shared descriptor facts or a drift test over
  duplicated hand-written version/task facts.

## M3 — Addon Event Delivery Runtime

- [ ] AEF-040 [owner=codex] [deps=AEF-020,AEF-030] [scope=crates/nako-core,crates/nako-db,crates/nako-server,crates/nako-addon-client]
  Goal: Deliver manifest-declared Addon Event Subscriptions from Nako's durable
  event outbox to Addon Sidecars through a host-owned runtime.
  Validation: `cargo nextest run -p nako-server addon_event --no-fail-fast`
  and `cargo nextest run -p nako-db event --no-fail-fast`.
  Review: Check retry/backoff, grant checks, token authority, redaction,
  cancellation, and event replay behavior.
  Evidence: event delivery runtime module and tests.
  Handoff: Keep webhook delivery and Addon Event Delivery as separate adapters
  over the same durable outbox concept.

## M4 — Official Event Addon Proof

- [ ] AEF-050 [owner=codex] [deps=AEF-040] [scope=official-addon-repo,scripts,docs]
  Goal: Add the first official event-driven addon proof path and suite-facing
  deployment guidance.
  Validation: focused official addon tests and a Nako-hosted smoke where
  feasible.
  Review: Confirm the proof is small and does not expand into a full
  notification provider matrix.
  Evidence: official addon crate or suite route and docs.
  Handoff: Split notification bridge, watch-state sync, MCP, Arr-stack, and
  compatibility protocols into named follow-ons after the event proof.

## M5 — Closeout And Follow-On Split

- [ ] AEF-060 [owner=planner] [deps=AEF-050] [scope=docs/workstreams/addon-ecosystem-foundation]
  Goal: Close this lane or split any remaining Tier 1/2/3 feature work into
  named workstreams.
  Validation: `cargo fmt --all -- --check`, focused nextest gates, relevant
  official addon gates, `git diff --check`, and `WORKSTREAM.json` parse.
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Do not leave broad addon feature ideas as unowned prose.
