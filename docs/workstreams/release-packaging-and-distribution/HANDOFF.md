# Release Packaging And Distribution — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

Workstream opened after `self-hosted-release-readiness` completed and after
stale active workstream statuses were cleaned.

The repo currently has:

- self-host release gates in `scripts/release-gate.*`,
- PostgreSQL contract harnesses,
- SQLite/PostgreSQL deployment examples,
- backup/restore/upgrade docs,
- self-host smoke tests,
- GitHub Actions release-gate shape.

The missing layer is packaging/distribution: artifact contract, startup/config
preflight, container build path, release artifact script, checksums, and
operator install/release docs.

## Active Task

- Task ID: RPD-030
- Owner: codex
- Files:
  - `crates/taru-server`
  - `Dockerfile`
  - `.dockerignore`
  - `deploy`
  - `scripts`
  - `docs/deployment`
  - `docs/workstreams/release-packaging-and-distribution/`
- Validation:
  - focused server config/preflight tests
  - `cargo build --locked --release -p taru-server`
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container -SkipRedactionInventory`
  - `git diff --check`
- Status: DONE

## Next Recommended Action

Continue with RPD-040 release artifact scripts and CI shape. RPD-030 added the
Taru server container contract, SQLite/PostgreSQL compose stacks, `database_url_env`
for secret DB URL injection, and a `release-gate` container mode.

## Follow-On Candidates

After packaging, evaluate:

- Metadata provider breadth,
- NFO/link management,
- Playback/transcode product hardening,
- Downloads / managed import staging,
- Network traversal,
- AI-assisted matching/inference.
