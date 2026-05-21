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

- Task ID: RPD-040
- Owner: codex
- Files:
  - `scripts`
  - `.github`
  - `docs/deployment`
  - `docs/workstreams/release-packaging-and-distribution/`
- Validation:
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf`
  - `bash scripts/package-release.sh --dry-run`
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -SkipBuild -OutputDir target/package-release-rpd040`
  - `git diff --check`
- Status: DONE

## Next Recommended Action

Continue with RPD-050 operator release checklist and install docs. RPD-040 added
repo-owned package scripts, release manifest/checksum output, artifact docs, and
a GitHub Actions workflow shape that calls the Bash package script.

## Follow-On Candidates

After packaging, evaluate:

- Metadata provider breadth,
- NFO/link management,
- Playback/transcode product hardening,
- Downloads / managed import staging,
- Network traversal,
- AI-assisted matching/inference.
