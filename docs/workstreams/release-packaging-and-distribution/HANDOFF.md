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

- Task ID: RPD-010
- Owner: planner
- Files:
  - `docs/workstreams/release-packaging-and-distribution/`
  - `docs/workstreams/README.md`
- Validation:
  - `git status --short --branch`
  - packaging/deploy baseline inventory
  - `git diff --check`
- Status: READY

## Next Recommended Action

Execute RPD-010, then continue with RPD-020 server startup/config preflight.

## Follow-On Candidates

After packaging, evaluate:

- Metadata provider breadth,
- NFO/link management,
- Playback/transcode product hardening,
- Downloads / managed import staging,
- Network traversal,
- AI-assisted matching/inference.
