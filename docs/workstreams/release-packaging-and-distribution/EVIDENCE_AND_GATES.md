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
cargo nextest run -p nako-server config --no-fail-fast

# Compose static gate placeholder
docker compose -f deploy/compose/<nako-compose>.yml config

# Artifact script gate placeholder
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf
bash scripts/package-release.sh --dry-run
```

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | RPD-010 planning | `git status --short --branch` | Pass. Worktree was clean before opening the lane after housekeeping commits. |
| 2026-05-21 | RPD-010 planning | Baseline from `self-hosted-release-readiness` | Pass. Existing deploy examples, release gates, PostgreSQL harness, and self-host smoke are available; packaging/distribution remains the next operator gap. |
| 2026-05-21 | RPD-010 inventory | `Get-ChildItem -Recurse deploy`; `Get-ChildItem scripts`; `Get-ChildItem -Recurse .github` | Pass. SQLite/PostgreSQL config examples, PostgreSQL-only compose, release gates, self-host smoke, PostgreSQL harness, and one release-gate workflow exist. No Nako server Dockerfile, `.dockerignore`, full compose app stack, package script, artifact manifest, or checksum script exists yet. |
| 2026-05-21 | RPD-010 artifact contract | `docs/workstreams/release-packaging-and-distribution/DESIGN.md` | Pass. Artifact contract V0 now defines shipped files, excluded runtime state/secrets, durable layout, and operator safety rules for RPD-020 through RPD-050. |
| 2026-05-21 | RPD-020 tests | `cargo nextest run -p nako-server config --no-fail-fast` | Pass. Added config preflight coverage for safe SQLite config with create/write probes, backend/URL mismatch redaction, unresolved database templates, public bind with disabled auth, enabled auth without token env, missing local media roots without path leaks, and duplicate library roots. |
| 2026-05-21 | RPD-020 formatting | `cargo fmt --all -- --check` | Pass. Rust formatting is clean after config preflight implementation. |
| 2026-05-21 | RPD-020 CLI smoke | temporary local SQLite config + `cargo run -q -p nako-server -- --config <temp>/nako.toml config-check --json --create-dirs` | Pass. `config-check` emits a passing redaction-safe JSON report and create/write-probes Nako-owned runtime directories. Build emitted pre-existing unused-code warnings from unrelated runtime modules. |
| 2026-05-21 | RPD-020 diff hygiene | `git diff --check` | Pass. No whitespace errors; Git reported line-ending normalization warnings only. |
| 2026-05-21 | RPD-020 module split verification | `cargo fmt --all -- --check`; `cargo nextest run -p nako-server config --no-fail-fast`; `git diff --check` | Pass. Split preflight implementation into `crates/nako-server/src/config/preflight.rs` while preserving the focused config gate. |
| 2026-05-21 | RPD-030 container shape | `Dockerfile`, `.dockerignore`, `deploy/container/*.toml`, `deploy/compose/nako-*.yml` | Added initial Nako server container contract with non-root runtime image, FFmpeg/FFprobe, local-only compose ports, config preflight before serve, durable volumes, read-only media mount, and secret env placeholders. |
| 2026-05-21 | RPD-030 config support | `cargo nextest run -p nako-server config --no-fail-fast` | Pass. Added `database_url_env` support so container PostgreSQL can inject the database URL without storing credentials in committed TOML. |
| 2026-05-21 | RPD-030 binary build | `cargo build --locked --release -p nako-server` | Pass. Host release binary builds with locked dependencies; Dockerfile uses the same locked release-package build command. |
| 2026-05-21 | RPD-030 compose gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container -SkipRedactionInventory` | Pass. Container gate runs Rust config tests plus `docker compose config` for SQLite and PostgreSQL Nako stacks with placeholder env values and a generated local media root. |
| 2026-05-21 | RPD-040 dry-run scripts | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf`; `bash scripts/package-release.sh --dry-run` | Pass. Both packaging entrypoints expose a non-publishing dry-run path. Bash dry-run tolerates the current WSL environment without rustc and reports `unknown-target`. |
| 2026-05-21 | RPD-040 package output | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -SkipBuild -OutputDir target/package-release-rpd040` | Pass. Generated zip archive, copied release manifest, and `SHA256SUMS`; manifest includes binary, Docker/compose/config examples, deployment docs, release artifact guide, license, README, git revision, target triple, dirty flag, and preflight command. |
| 2026-05-21 | RPD-050 operator docs | `docs/deployment/RELEASE_CHECKLIST.md`; `docs/deployment/RELEASE_ARTIFACTS.md`; `docs/deployment/SELF_HOSTED.md`; `docs/deployment/BACKUP_RESTORE_UPGRADE.md` | Pass. Operator docs now cover artifact verification, config/secrets, first start, compose start, health/diagnostics, backup, upgrade, rollback, and support bundle boundaries. |
| 2026-05-21 | RPD-060 future lane decision | `docs/workstreams/release-packaging-and-distribution/DESIGN.md` | Pass. Added decision matrix for Metadata, NFO/link, Playback/transcode, Downloads/managed import staging, Network traversal, AI, and Addon distribution; recommends Metadata Provider Breadth next, with Downloads split to `managed-import-staging` if prioritized. |
| 2026-05-21 | RPD-070 closeout docs gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs -SkipRedactionInventory` | Pass. Formatting and diff hygiene passed for docs-safe closeout. |
| 2026-05-21 | RPD-070 closeout container gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container -SkipRedactionInventory` | Pass. Re-ran focused config tests and compose config checks for SQLite/PostgreSQL Nako stacks. |
| 2026-05-21 | RPD-070 closeout package gate | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf`; `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -SkipBuild -OutputDir target/package-release-rpd-closeout` | Pass. Re-proved packaging dry-run plus real manifest/archive/checksum emission from existing release binary. |
| 2026-05-21 | RPD-070 closeout diff hygiene | `cargo fmt --all -- --check`; `git diff --check` | Pass. Rust formatting and whitespace checks are clean; Git reported line-ending normalization warnings only. |

## Open Evidence Gaps

- Nako server image build should be proven where Docker build time is acceptable.
- Shell script execution should be proven in Linux/CI once packaging workflows
  exist.
- Full `release-gate fast/workspace` and Docker image build were not repeated
  during closeout to keep this packaging closeout bounded. RPD-030/RPD-040
  proved focused config, compose, host release build, and package emission; CI
  retains broader release-gate coverage.
