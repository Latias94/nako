# Self-Hosted Deployment

Status: Draft release baseline

This guide covers the current self-hosted Taru server shape for SQLite and
PostgreSQL operators. It is deliberately conservative: bind locally by default,
keep secrets in environment variables, and keep durable state separate from
cache/rebuildable state.

## Quick Start

1. Install a Rust toolchain, FFmpeg/FFprobe, and `cargo-nextest`.
2. Choose a database backend:
   - SQLite: copy `deploy/sqlite/taru.toml`.
   - PostgreSQL: copy `deploy/postgres/taru.toml` and optionally start
     `deploy/compose/postgres.yml`.
3. Replace example paths with host paths.
4. Export secrets:

```bash
export TARU_ADMIN_TOKEN='replace-with-a-long-random-token'
export TMDB_READ_ACCESS_TOKEN='optional-provider-token'
export BANGUMI_TOKEN='optional-provider-token'
export DOUBAN_API_KEY='optional-provider-key'
export TARU_POSTGRES_PASSWORD='replace-with-a-long-random-password'
```

5. Validate the config without connecting to production databases or external
   metadata providers:

```bash
cargo run -p taru-server -- --config /etc/taru/taru.toml config-check --create-dirs
```

For JSON output suitable for CI/support bundles:

```bash
cargo run -p taru-server -- --config /etc/taru/taru.toml config-check --json
```

6. Start the server:

```bash
cargo run -p taru-server -- --config /etc/taru/taru.toml serve
```

For PowerShell:

```powershell
$env:TARU_ADMIN_TOKEN = 'replace-with-a-long-random-token'
cargo run -p taru-server -- --config C:\Taru\taru.toml config-check --create-dirs
cargo run -p taru-server -- --config C:\Taru\taru.toml serve
```

`config-check` fails for hard safety problems such as unresolved database
templates, backend/URL mismatches, missing auth token environment variables,
unsafe public binds with auth disabled, missing local media library roots, and
runtime paths that cannot be created or write-probed when `--create-dirs` is
used. It may return warnings for intentionally local/dev shapes, such as
disabled auth on loopback or missing cache directories when no create/write
probe was requested.

## Network Exposure

The example configs bind to `127.0.0.1:3000`. Keep that default until Taru is
behind a trusted reverse proxy, VPN, tunnel, or private network boundary.

Do not expose Taru directly to the public internet with placeholder tokens,
disabled auth, or an unaudited reverse proxy. `GET /health` is intentionally
public for readiness checks; all other routes should require bearer auth when
`[auth].enabled = true`.

## Database Configuration

### SQLite

SQLite is the simplest self-hosted mode:

```toml
database_backend = "sqlite"
database_url = "sqlite:///var/lib/taru/taru.db"
```

Use a local disk path, not a network filesystem, for the SQLite database. Taru
creates the database file if it is missing and uses the workspace SQLite runtime
policy for on-disk operation.

### PostgreSQL

PostgreSQL mode is explicit:

```toml
database_backend = "postgres"
database_url = "postgres://taru:${TARU_POSTGRES_PASSWORD}@127.0.0.1:5432/taru"
```

Current Taru config treats `database_url` as a literal string. If your runtime
does not expand `${TARU_POSTGRES_PASSWORD}` before Taru reads the file, render
the config from your secret manager or service manager before launch. Do not
commit a real database password. `config-check` intentionally fails unresolved
`${...}` markers so a packaged server does not start with a placeholder
credential.

To start only PostgreSQL for local testing:

```bash
TARU_POSTGRES_PASSWORD='replace-with-a-long-random-password' \
  docker compose -f deploy/compose/postgres.yml up -d
```

The compose example binds PostgreSQL to `127.0.0.1:5432` only and stores DB
bytes in a named volume.

## Durable State And Cache State

Durable state:

- SQLite database file or PostgreSQL database.
- `artwork.artifact_root` for Managed Artwork bytes.
- Local media library files and NFO sidecars.
- Operator-owned secrets in the environment or secret manager.

Cache/rebuildable state:

- `remux_staging_root` remux/HLS outputs and remote FFmpeg inputs.
- `[staging]` manifest-tracked remote probe and FFmpeg input staging.
- Provider raw-cache rows when retention cleanup is enabled.

Keep `database_url`, `artwork.artifact_root`, `remux_staging_root`, and library
roots on separate, clearly named paths. Do not place cache roots inside media
library roots, otherwise scans may ingest generated files.

## Auth

Example:

```toml
[auth]
enabled = true
token_env = "TARU_ADMIN_TOKEN"
```

Set `TARU_ADMIN_TOKEN` to a long random value. Do not reuse it for Addons,
Webhooks, metadata providers, WebDAV, PostgreSQL, or reverse proxies.

## Metadata Providers

The example configs include TMDB, Bangumi, and Douban provider sections but keep
them disabled. Enable only providers you have credentials and terms-of-service
permission to use:

```toml
[[metadata.providers]]
provider = "tmdb"
enabled = true
token_env = "TMDB_READ_ACCESS_TOKEN"
language = "en-US"
include_adult = false
```

Use `token_env`, `api_key_env`, and `value_env` for provider secrets. Do not
place provider tokens in URLs, headers committed to git, logs, NFO sidecars, or
public/admin response bodies.

## Addons And Webhooks

Addon and Webhook routes are runtime-managed through the HTTP API, not by
static registration in `taru.toml`.

Operational guidance:

- Register Addons with Admin API routes, then issue Addon Tokens through the
  Addon token routes. Addon Tokens are not Admin bearer tokens.
- Keep Addon Sidecars on trusted private networks unless their own auth and
  transport security are configured.
- Register Webhook endpoints with explicit secret environment references.
- `webhook_concurrency` bounds delivery attempts; keep it low until endpoint
  latency and retry behavior are understood.
- Webhook diagnostics are redacted: do not depend on raw secret or payload
  echoing for debugging.

## Playback Runtime

Playback depends on FFmpeg/FFprobe paths and resource budgets:

```toml
ffprobe_path = "ffprobe"
ffmpeg_path = "ffmpeg"
remux_concurrency = 1
remux_timeout_ms = 1800000
remux_staging_root = "/var/cache/taru/remux"

[transcode]
hardware_acceleration = "none"
hardware_fallback = "cpu"
cpu_concurrency = 1
gpu_concurrency = 1

[playback]
remote_stream_concurrency = 8
remote_stage_concurrency = 2
```

Supported hardware acceleration config values are `none`, `vaapi`, `nvenc`,
and `quick_sync`; fallback values are `cpu` and `fail`. The release baseline
does not require GPU access to pass tests, but production operators should
verify FFmpeg device availability before enabling GPU policies.

## Diagnostics

Useful checks:

```bash
cargo run -p taru-server -- --config /etc/taru/taru.toml config-check --json
curl http://127.0.0.1:3000/health
curl -H "Authorization: Bearer $TARU_ADMIN_TOKEN" \
  http://127.0.0.1:3000/admin/v1/overview
curl -H "Authorization: Bearer $TARU_ADMIN_TOKEN" \
  http://127.0.0.1:3000/admin/v1/system/config
```

Admin diagnostics are designed to be redacted: they report configured
capabilities and booleans rather than raw local paths, database URLs, source
URIs, provider tokens, Addon Tokens, Webhook secrets, or bearer token values.

For local release confidence:

```bash
bash scripts/release-gate.sh --mode fast
bash scripts/release-gate.sh --mode postgres
```

On Windows:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres
```

## Example Inventory

- SQLite config: `deploy/sqlite/taru.toml`
- PostgreSQL config: `deploy/postgres/taru.toml`
- PostgreSQL compose service: `deploy/compose/postgres.yml`
- Backup/restore/upgrade runbook: `docs/deployment/BACKUP_RESTORE_UPGRADE.md`
- Release gate: `scripts/release-gate.ps1` and `scripts/release-gate.sh`
- PostgreSQL contract harness: `scripts/postgres-contract-harness.ps1` and
  `scripts/postgres-contract-harness.sh`
