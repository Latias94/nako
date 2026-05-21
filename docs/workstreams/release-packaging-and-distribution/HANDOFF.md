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

- Task ID: RPD-020
- Owner: codex
- Files:
  - `crates/taru-server`
  - `docs/deployment`
  - `docs/workstreams/release-packaging-and-distribution/`
- Validation:
  - focused server config/preflight tests
  - `cargo nextest run -p taru-server config --no-fail-fast`
  - `git diff --check`
- Status: DONE

## Next Recommended Action

Continue with RPD-030 container build shape. RPD-020 added a `config-check`
server command, redaction-safe config preflight report, create/write probes for
Taru-owned runtime directories, and self-hosted docs.

## Follow-On Candidates

After packaging, evaluate:

- Metadata provider breadth,
- NFO/link management,
- Playback/transcode product hardening,
- Downloads / managed import staging,
- Network traversal,
- AI-assisted matching/inference.
