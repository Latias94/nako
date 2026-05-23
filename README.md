# Nako

![Nako app icon](assets/brand/nako-app-icon-1024.png)

**Your media home, gently kept.**

Nako is an open-source, self-hosted media home for gently organizing, keeping,
and playing your films, shows, anime, and personal collection.

## Status

Nako is currently `0.1.0-alpha.1`.

This is a technical preview for self-hosted operators and contributors. It is
not yet a stable Jellyfin/Plex replacement, and public API, Admin API, Addon
Protocol, database schema, and generated SDK shapes may change before beta.

## What Works Today

- Media Library scanning with Local Inference, Source State tracking, Media
  Source records, and Provisional Hierarchy creation.
- SQLite and PostgreSQL persistence paths with contract tests.
- Local filesystem and WebDAV-oriented VFS foundations.
- Metadata provider runtime surfaces for TMDB, Bangumi, and Douban.
- NFO import/export and local metadata authority groundwork.
- Managed Import, Nako-managed artwork, Library File Write, and promotion/apply
  workflows for operator-confirmed library mutation.
- Playback source selection, remux/transcode planning, hardware acceleration
  policy, runtime diagnostics, and bounded staging.
- Admin API and Admin Web Console surfaces for diagnostics, operations, Addon
  onboarding, Addon credentials/grants, and runtime status.
- Addon Sidecar protocol support with scoped tokens, grants, health checks,
  install guide generation, and resource-call diagnostics.
- Docker/Compose examples, config preflight, release packaging, and
  backup/restore documentation for self-hosted operation.

## Current Boundaries

- Addons are externally run Addon Sidecars. Nako does not yet install, update,
  start, stop, remove, log, or supervise addon processes.
- `nako-official-addons` is an experimental companion repository for official
  Addon Sidecar experiments: <https://github.com/Latias94/nako-official-addons>
- Network tunnel support is policy/readiness oriented. Nako does not currently
  run a built-in NAT traversal or relay service.
- AI-assisted workflows are intentionally not part of the `alpha.1` release
  promise.
- Keep Nako local-only, private-network, VPN, reverse-proxy, or tunnel-bounded
  with auth enabled. Do not expose placeholder configs to the public internet.

## Quick Start

From source:

```powershell
Copy-Item deploy/sqlite/nako.toml .\nako.local.toml
# Edit nako.local.toml first: database path, artifact/cache roots, and library root.
$env:NAKO_ADMIN_TOKEN = 'replace-with-a-long-random-token'
cargo run -p nako-server -- --config .\nako.local.toml config-check --create-dirs
cargo run -p nako-server -- --config .\nako.local.toml serve
```

Container examples:

```powershell
Copy-Item deploy/compose/.env.example deploy/compose/.env
# Edit deploy/compose/.env before running.
docker compose --env-file deploy/compose/.env -f deploy/compose/nako-sqlite.yml up --build
```

Direct image build:

```powershell
docker build -t nako:0.1.0-alpha.1 .
```

If `deb.debian.org` is unstable from your network, override the Debian mirrors:

```powershell
docker build `
  --build-arg DEBIAN_MIRROR=http://mirrors.aliyun.com/debian `
  --build-arg DEBIAN_SECURITY_MIRROR=http://mirrors.aliyun.com/debian-security `
  -t nako:0.1.0-alpha.1 .
```

For full operator notes, see:

- `docs/deployment/SELF_HOSTED.md`
- `docs/deployment/RELEASE_CHECKLIST.md`
- `docs/deployment/BACKUP_RESTORE_UPGRADE.md`

## Development Gates

Common local checks:

```powershell
rustc --version # workspace rust-version is 1.95
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
```

## License

Nako server-side code is licensed under AGPL-3.0-or-later unless a crate or file
states otherwise. Addon protocol, addon client, client SDK, and reference addon
crates are licensed as Apache-2.0 OR MIT for Addon and client authors.

See `docs/LICENSING.md` for the crate-level policy.
