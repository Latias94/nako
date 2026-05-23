# Nako

[![release-gate](https://github.com/Latias94/nako/actions/workflows/release-gate.yml/badge.svg)](https://github.com/Latias94/nako/actions/workflows/release-gate.yml)
[![release-package](https://github.com/Latias94/nako/actions/workflows/release-package.yml/badge.svg)](https://github.com/Latias94/nako/actions/workflows/release-package.yml)
[![crates-publish](https://github.com/Latias94/nako/actions/workflows/crates-publish.yml/badge.svg)](https://github.com/Latias94/nako/actions/workflows/crates-publish.yml)
[![docker-publish](https://github.com/Latias94/nako/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/Latias94/nako/actions/workflows/docker-publish.yml)
![status: alpha.1](https://img.shields.io/badge/status-alpha.1-orange)
![rust: 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange)
![server license: AGPL-3.0-or-later](https://img.shields.io/badge/server%20license-AGPL--3.0--or--later-blue)
![addon/client SDK: Apache-2.0 OR MIT](https://img.shields.io/badge/addon%2Fclient%20SDK-Apache--2.0%20OR%20MIT-blue)

<p>
  <img src="assets/brand/nako-app-icon-1024.png" alt="Nako app icon" width="128">
</p>

**Your media home, gently kept.**

Nako is an open-source media server backend for people who want to keep their
own films, shows, anime, and personal collection on hardware they control.

## Status

Nako is currently `0.1.0-alpha.1`.

This is a technical preview. It is useful for development, self-hosted testing,
and early addon work, but it is not a stable Jellyfin or Plex replacement yet.
The public API, Admin API, Addon Protocol, database schema, and generated SDKs
may still change before beta.

Current Addon Protocol Version: `0.1.0-alpha.1`. Addon Version, protocol
compatibility, and Rust crate package versions are separate contracts.

## What works today

- Media Library scanning with Local Inference, Source State tracking, Media
  Source records, and Provisional Hierarchy.
- SQLite and PostgreSQL persistence paths with contract tests.
- Local filesystem VFS, with WebDAV-oriented pieces already in place.
- Metadata provider runtime for TMDB, Bangumi, and Douban.
- NFO import/export and local metadata authority.
- Managed Import, Nako-managed artwork, Library File Write, and promotion/apply
  flows for operator-confirmed file changes.
- Playback source selection, remux/transcode planning, hardware acceleration
  policy, runtime diagnostics, and bounded staging.
- Admin API and Admin Web Console pages for diagnostics, operations, Addon
  onboarding, credentials, grants, and runtime status.
- Addon Sidecar protocol support with scoped tokens, grants, health checks,
  install guides, and resource-call diagnostics.
- Docker/Compose examples, config preflight, release packaging, and operator
  docs.

## Current boundaries

- Addons are externally run Addon Sidecars. Nako does not yet install, update,
  start, stop, remove, log, or supervise addon processes.
- `nako-official-addons` is the companion repository for official Addon
  Sidecar experiments. The first alpha companion addon is
  `nako-metadata-scraper@0.1.0-alpha.1`:
  <https://github.com/Latias94/nako-official-addons>
- Network tunnel support is policy/readiness oriented. Nako does not currently
  run a built-in NAT traversal or relay service.
- AI-assisted workflows are intentionally not part of the `alpha.1` release
  promise.
- Keep Nako local-only, private-network, VPN, reverse-proxy, or tunnel-bounded
  with auth enabled. Do not expose placeholder configs to the public internet.

## Rust SDK crates

The first crates.io publishing lane is small on purpose:

- `nako`: facade crate for public Nako protocol and SDK surfaces.
- `nako-addon-protocol`: Addon Protocol wire types and validation helpers.
- `nako-addon-client`: optional Rust HTTP caller helper for Addon Sidecars.

Server implementation crates are not published as library APIs during alpha.
The first official companion addon, `nako-metadata-scraper`, uses
`nako-addon-protocol = 0.1.0-alpha.1` from crates.io.

## Quick start

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

Published alpha image:

```powershell
docker pull ghcr.io/latias94/nako-server:0.1.0-alpha.1
docker pull ghcr.io/latias94/nako-server:alpha
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

- [Self-hosted setup](https://github.com/Latias94/nako/blob/main/docs/deployment/SELF_HOSTED.md)
- [Release checklist](https://github.com/Latias94/nako/blob/main/docs/deployment/RELEASE_CHECKLIST.md)
- [Backup, restore, and upgrade](https://github.com/Latias94/nako/blob/main/docs/deployment/BACKUP_RESTORE_UPGRADE.md)

More docs:

- [HTTP API](https://github.com/Latias94/nako/blob/main/docs/api/HTTP_API.md)
- [Addon author guide](https://github.com/Latias94/nako/blob/main/docs/guides/ADDON_AUTHOR_GUIDE.md)
- [Licensing policy](https://github.com/Latias94/nako/blob/main/docs/LICENSING.md)

## Development gates

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

See the [licensing policy](https://github.com/Latias94/nako/blob/main/docs/LICENSING.md)
for the crate-level split.
