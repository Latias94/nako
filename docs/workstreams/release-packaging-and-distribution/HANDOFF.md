# Release Packaging And Distribution — Handoff

Status: Completed
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

## Closeout

- Task ID: RPD-070
- Owner: codex
- Files:
  - `docs/deployment`
  - `docs/workstreams/release-packaging-and-distribution/`
- Validation:
  - docs inventory covers install, config, start, verify, backup, upgrade,
    rollback, logs, diagnostics, support bundle, and checksums
  - future lane matrix exists
  - `git diff --check`
- Status: DONE

## Next Recommended Action

Open the next product workstream. Recommended: Metadata Provider Breadth. If
downloads is prioritized instead, open `managed-import-staging` with a narrow
quarantine/validate/manual-promote scope rather than a broad downloads lane.

## Residual Risks

- Docker image build was not repeated during closeout; the Dockerfile uses the
  proven locked release build command and compose config is covered.
- Bash package script dry-run was proven locally; full Bash package execution
  should be proven by Linux CI through `.github/workflows/release-package.yml`.
- Full workspace release gate remains a broader CI/workspace responsibility and
  was not rerun for this packaging closeout.

## Follow-On Candidates

After packaging, evaluate:

- Metadata provider breadth,
- NFO/link management,
- Playback/transcode product hardening,
- Downloads / managed import staging,
- Network traversal,
- AI-assisted matching/inference.
