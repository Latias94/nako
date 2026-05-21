# Release Packaging And Distribution — TODO

Status: Active
Last updated: 2026-05-21

Task IDs use the `RPD` prefix.

## M0 — Scope, Baseline, And Release Contract

- [ ] RPD-010 [owner=planner] [deps=none] [scope=docs/workstreams/release-packaging-and-distribution,docs/workstreams/README.md]
  Goal: Open the packaging lane, inventory current binary/config/deploy state,
  and freeze the artifact contract for this workstream.
  Validation: `git status --short --branch`; inventory of existing Docker,
  deploy, config, CI, and release scripts; `git diff --check`.
  Review: Do not implement packaging before the artifact contract is explicit.
  Evidence: baseline inventory in `DESIGN.md` and `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with server startup/config preflight.

## M1 — Server Startup And Config Preflight

- [ ] RPD-020 [owner=codex] [deps=RPD-010] [scope=crates/taru-server,docs/deployment,docs/workstreams/release-packaging-and-distribution]
  Goal: Provide an operator-safe config validation/preflight path for packaged
  runs, covering config file parsing, database backend selection, bind address,
  artifact/staging directories, auth, and redacted error output.
  Validation: focused server config/preflight tests; `cargo nextest run -p
  taru-server config --no-fail-fast`; `git diff --check`.
  Review: Validation must not require connecting to production services unless
  explicitly requested, and must not print secrets.
  Evidence: CLI/config tests and docs.
  Handoff: Continue with container build shape.

## M2 — Container And Compose Packaging

- [ ] RPD-030 [owner=codex] [deps=RPD-020] [scope=Dockerfile,deploy,docs/deployment,scripts,docs/workstreams/release-packaging-and-distribution]
  Goal: Add a Taru server container build path and compose examples that run
  Taru with durable external volumes and PostgreSQL or SQLite configuration.
  Validation: Dockerfile/static build checks where Docker is available;
  `docker compose config`; docs grep for durable state, auth, DB, artifact root,
  and backup warnings; `git diff --check`.
  Review: Examples must not ship unsafe public binds or real secrets.
  Evidence: container/compose outputs and docs.
  Handoff: Continue with release artifact scripts/CI.

## M3 — Release Artifact Script And CI Shape

- [ ] RPD-040 [owner=codex] [deps=RPD-030] [scope=scripts,.github,docs/deployment,docs/workstreams/release-packaging-and-distribution]
  Goal: Add a repeatable local/CI release artifact command that builds binaries
  or image artifacts, records version metadata, and emits checksums/evidence.
  Validation: local dry-run or focused artifact build; CI workflow syntax;
  checksum file generation; `scripts/release-gate.*` integration if suitable;
  `git diff --check`.
  Review: CI should call repo-owned scripts rather than duplicate long command
  recipes.
  Evidence: artifact manifest and workflow shape.
  Handoff: Continue with release checklist docs.

## M4 — Operator Release Checklist And Install Docs

- [ ] RPD-050 [owner=codex] [deps=RPD-040] [scope=docs/deployment,docs/workstreams/release-packaging-and-distribution]
  Goal: Document install, first start, upgrade, rollback, backup, logs,
  diagnostics, and support bundle expectations for packaged Taru.
  Validation: docs inventory covers install, config, start, verify, backup,
  upgrade, rollback, logs, diagnostics, and artifact checksums; `git diff
  --check`.
  Review: The docs must distinguish source-built development from packaged
  operation.
  Evidence: release checklist docs.
  Handoff: Continue with future-lane evaluation.

## M5 — Future Lane Evaluation: Metadata, NFO, Playback, Downloads

- [ ] RPD-060 [owner=planner] [deps=RPD-050] [scope=docs/workstreams/release-packaging-and-distribution,docs/workstreams/README.md]
  Goal: Decide which product lane should follow packaging and record explicit
  split criteria for Metadata provider breadth, NFO/link management,
  Playback/transcode hardening, Downloads, Network traversal, and AI.
  Validation: decision matrix exists; no implementation hidden in planning;
  `git diff --check`.
  Review: Downloads must be defined precisely before implementation.
  Evidence: decision matrix in `DESIGN.md` or a follow-on doc.
  Handoff: Open the selected next product workstream.

## M6 — Closeout

- [ ] RPD-070 [owner=planner] [deps=RPD-060] [scope=docs/workstreams/release-packaging-and-distribution,docs/workstreams/README.md]
  Goal: Verify and close the packaging lane or split incomplete work.
  Validation: release packaging gates, final `cargo fmt --all -- --check`,
  `git diff --check`, and all required artifact/container/docs evidence.
  Review: Close only when an operator can follow docs from artifact to running
  instance.
  Evidence: closeout journal and completed workstream docs.
