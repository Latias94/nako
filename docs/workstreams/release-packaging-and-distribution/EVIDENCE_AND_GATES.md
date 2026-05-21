# Release Packaging And Distribution — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Baseline Gates

Initial expected gate family:

```powershell
cargo fmt --all -- --check
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode workspace -SkipRedactionInventory
git diff --check
```

Packaging-specific gates will be refined by RPD-020 through RPD-050.

## Candidate Packaging Gates

```powershell
# Config/preflight gate placeholder
cargo nextest run -p taru-server config --no-fail-fast

# Compose static gate placeholder
docker compose -f deploy/compose/<taru-compose>.yml config

# Artifact script placeholder
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf
```

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | RPD-010 planning | `git status --short --branch` | Pass. Worktree was clean before opening the lane after housekeeping commits. |
| 2026-05-21 | RPD-010 planning | Baseline from `self-hosted-release-readiness` | Pass. Existing deploy examples, release gates, PostgreSQL harness, and self-host smoke are available; packaging/distribution remains the next operator gap. |

## Open Evidence Gaps

- Server config/preflight behavior needs a package-friendly command contract.
- Taru server container image is not yet defined.
- Release artifact scripts/checksums are not yet defined.
- Shell script execution should be proven in Linux/CI once packaging workflows
  exist.
