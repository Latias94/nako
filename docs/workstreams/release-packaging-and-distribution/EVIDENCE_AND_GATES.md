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
| 2026-05-21 | RPD-010 inventory | `Get-ChildItem -Recurse deploy`; `Get-ChildItem scripts`; `Get-ChildItem -Recurse .github` | Pass. SQLite/PostgreSQL config examples, PostgreSQL-only compose, release gates, self-host smoke, PostgreSQL harness, and one release-gate workflow exist. No Taru server Dockerfile, `.dockerignore`, full compose app stack, package script, artifact manifest, or checksum script exists yet. |
| 2026-05-21 | RPD-010 artifact contract | `docs/workstreams/release-packaging-and-distribution/DESIGN.md` | Pass. Artifact contract V0 now defines shipped files, excluded runtime state/secrets, durable layout, and operator safety rules for RPD-020 through RPD-050. |
| 2026-05-21 | RPD-020 tests | `cargo nextest run -p taru-server config --no-fail-fast` | Pass. Added config preflight coverage for safe SQLite config with create/write probes, backend/URL mismatch redaction, unresolved database templates, public bind with disabled auth, enabled auth without token env, missing local media roots without path leaks, and duplicate library roots. |
| 2026-05-21 | RPD-020 formatting | `cargo fmt --all -- --check` | Pass. Rust formatting is clean after config preflight implementation. |
| 2026-05-21 | RPD-020 CLI smoke | temporary local SQLite config + `cargo run -q -p taru-server -- --config <temp>/taru.toml config-check --json --create-dirs` | Pass. `config-check` emits a passing redaction-safe JSON report and create/write-probes Taru-owned runtime directories. Build emitted pre-existing unused-code warnings from unrelated runtime modules. |
| 2026-05-21 | RPD-020 diff hygiene | `git diff --check` | Pass. No whitespace errors; Git reported line-ending normalization warnings only. |
| 2026-05-21 | RPD-020 module split verification | `cargo fmt --all -- --check`; `cargo nextest run -p taru-server config --no-fail-fast`; `git diff --check` | Pass. Split preflight implementation into `crates/taru-server/src/config/preflight.rs` while preserving the focused config gate. |

## Open Evidence Gaps

- Server config/preflight command should be smoke-proven from packaged examples
  after container paths are added.
- Taru server container image is not yet defined.
- Release artifact scripts/checksums are not yet defined.
- Shell script execution should be proven in Linux/CI once packaging workflows
  exist.
