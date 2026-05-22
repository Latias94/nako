# Release Artifacts

Status: Draft packaging baseline

This document describes the local/CI artifact contract for a self-hosted Nako
server release. It does not publish to a registry or app store.

## Build Locally

PowerShell:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1
```

Bash:

```bash
bash scripts/package-release.sh
```

Dry-run shape checks:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf
bash scripts/package-release.sh --dry-run
```

If `target/release/nako-server` already exists and you only want to verify the
packaging layout:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -SkipBuild
bash scripts/package-release.sh --skip-build
```

## Output

Artifacts are written to `target/package-release/`:

- `nako-server-v<version>-<target>-<git>.zip` on PowerShell.
- `nako-server-v<version>-<target>-<git>.tar.gz` on Bash.
- `<package>.release-manifest.json`.
- `SHA256SUMS`.

The archive contains:

- `bin/nako-server` or `bin/nako-server.exe`.
- `release-manifest.json`.
- license/readme files.
- Dockerfile and `.dockerignore`.
- SQLite/PostgreSQL config examples.
- container-native config examples.
- compose examples and `.env.example`.
- release artifact guide.
- self-hosted deployment and backup/restore/upgrade docs.

The release manifest records version, git revision, dirty-state flag, target
triple, build command, preflight command, archive name, binary path, and included
files. `SHA256SUMS` covers the archive and copied manifest.

## CI Shape

`.github/workflows/release-package.yml` calls the Bash package script and uploads
the generated archive, manifest, and checksums as workflow artifacts. CI should
call repo-owned scripts instead of duplicating long build/package recipes.

## Operator Verification

After downloading an artifact:

```bash
sha256sum -c SHA256SUMS
```

Then extract the archive, copy a config, set secrets, and run:

```bash
nako-server --config /config/nako.toml config-check --create-dirs
```

See `docs/deployment/SELF_HOSTED.md` and
`docs/deployment/RELEASE_CHECKLIST.md` for install and first start. See
`docs/deployment/BACKUP_RESTORE_UPGRADE.md` for backup, upgrade, and rollback
guidance.
