---
name: nako-release
description: Prepares and verifies Nako releases with changelog generation, version bumps, docs updates, package/image/crate checks, and local CI gates. Use when the user asks to release Nako, prepare an alpha/beta, write a changelog, cut a tag, run release gates, or publish Docker/crates artifacts.
---

# Nako Release

## Quick Start

1. Confirm the release target and previous tag:
   - `git status --short`
   - `git tag --sort=-creatordate`
   - `git log --oneline <previous-tag>..HEAD`
2. Draft a user-facing `CHANGELOG.md` entry from `<previous-tag>..HEAD`.
3. Bump the workspace release version in `Cargo.toml`, then refresh
   `Cargo.lock` with `cargo metadata --format-version 1`.
4. Update release-facing docs and examples.
5. Run local CI gates before tagging.
6. Commit release prep, then create/push the `v<version>` tag only after gates
   are green or explicitly documented as unavailable.

## Version And Docs Checklist

- `Cargo.toml`: update `[workspace.package].version`.
- `Cargo.lock`: refresh workspace package versions.
- `CHANGELOG.md`: add a user-facing release entry with Added/Changed/Upgrade
  Notes.
- `README.md`: update status badge, current version, image tags, release
  promise wording, SDK crate examples, and companion addon version if it is part
  of the release train.
- `crates/nako/README.md`: update the public facade crate dependency example.
- `docs/deployment/RELEASE_CHECKLIST.md`: update release refs and default
  artifact/image/addon versions.
- `docs/guides/ADDON_AUTHOR_GUIDE.md`: update only release-loop wording and
  companion addon defaults.
- `scripts/official-addon-e2e-smoke.ps1`: update default Nako image and
  companion addon version when the published smoke should target the new
  release.

Do not bump `ADDON_PROTOCOL_VERSION` merely because the server release version
changed. Bump it only when the runtime Addon Protocol compatibility contract
changes, then update protocol docs, examples, manifests, and tests together.

## Local CI Gates

Run the same shape as `.github/workflows/release-gate.yml` where local tooling
allows:

```powershell
npm ci --prefix sdk/typescript
npm ci --prefix apps/admin-web
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode api
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1 -Backend sqlite
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork -RequireTooling
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container
```

For broader confidence before a public tag, also run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode workspace
python scripts/publish_crates.py --mode dry-run
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1
```

If Docker or local PostgreSQL tooling is missing, report the skipped gate
explicitly and rely on the GitHub workflow for that environment-specific check.

## Tag And Publish Order

1. Commit release prep with `chore(release): prepare <version>`.
2. Create the immutable tag: `git tag -a v<version> -m "v<version>"`.
3. Push the commit and tag.
4. Verify GitHub `release-package`, `docker-publish`, and `crates-publish`
   dry-run workflows.
5. Manually approve crates.io publishing only for public permissive crates.
6. Manually approve Docker publish when the tagged image smoke passes.

Never publish `latest` during alpha. Use immutable `v<version>`/`<version>` tags
and the moving `alpha` channel only for alpha releases.
