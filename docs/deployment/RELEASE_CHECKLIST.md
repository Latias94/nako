# Operator Release Checklist

Status: Draft packaging baseline

Use this checklist for packaged Nako self-hosted releases. It assumes you are
installing from a release artifact or the provided compose files, not from an
ad-hoc source checkout.

## 1. Verify Artifact Integrity

1. Download the archive, copied release manifest, and `SHA256SUMS`.
2. Verify checksums:

```bash
sha256sum -c SHA256SUMS
```

3. Read `<package>.release-manifest.json` and record:
   - `version`
   - `git_revision`
   - `target_triple`
   - `binary`
   - `included_files`

Do not run an artifact whose checksum does not match. Do not paste raw secrets
or private paths into support tickets when sharing the manifest.

## 1a. Public Crate Publishing

Nako publishes only public permissive crates to crates.io during alpha:

- `nako-addon-protocol`
- `nako-addon-client`
- `nako`

Server implementation crates are marked `publish = false` and are not release
library APIs. Before publishing, run the crates publish readiness check:

```bash
python scripts/publish_crates.py --mode dry-run
```

Actual crates.io publishing is manual-approval only through the
`crates-publish` workflow, using the `crates-io` GitHub environment and
`CARGO_REGISTRY_TOKEN` secret. Publish order is dependency order:
`nako-addon-protocol`, `nako-addon-client`, then `nako`.

## 1b. Docker Image Publishing

Nako publishes release images to GitHub Container Registry from immutable
release tags:

- `ghcr.io/latias94/nako-server:<version>`
- `ghcr.io/latias94/nako-server:alpha` for alpha releases

The `docker-publish` workflow builds the image, runs container smoke checks,
then pushes only after the smoke checks pass. For an existing tag, run it
manually with:

```text
release_ref = v0.1.0-alpha.1
publish = true
```

Do not push `latest` during alpha.

## 2. Prepare Config And Secrets

1. Choose SQLite or PostgreSQL.
2. Copy a config example:
   - Source/host install: `deploy/sqlite/nako.toml` or
     `deploy/postgres/nako.toml`.
   - Container install: `deploy/container/sqlite.nako.toml` or
     `deploy/container/postgres.nako.toml`.
3. Set secrets outside the config file:
   - `NAKO_ADMIN_TOKEN`
   - `NAKO_DATABASE_URL` for PostgreSQL
   - Provider/WebDAV/Webhook secrets as needed
4. Confirm durable paths:
   - database path or PostgreSQL volume
   - `artwork.artifact_root`
   - `remux_staging_root`
   - media library mounts
   - config and secret manager location

## 3. First Start

Run preflight before serving traffic:

```bash
nako-server --config /config/nako.toml config-check --create-dirs
```

Expected hard failures:

- auth enabled but token environment variable missing,
- public bind with auth disabled,
- database backend/URL mismatch,
- unresolved `${...}` placeholders,
- missing local media library root,
- Nako-owned artifact/staging path cannot be created or write-probed.

Start Nako only after hard failures are fixed.

## 4. Container Start

Copy `.env.example`, replace all values, then run one stack:

```bash
docker compose --env-file deploy/compose/.env -f deploy/compose/nako-sqlite.yml up --build
docker compose --env-file deploy/compose/.env -f deploy/compose/nako-postgres.yml up --build
```

The compose stacks:

- bind Nako to `127.0.0.1:3000`,
- run `config-check --create-dirs` before `serve`,
- mount media read-only,
- keep DB/artifact/cache state outside the image layer.

## 5. Health And Diagnostics

```bash
curl http://127.0.0.1:3000/health
curl -H "Authorization: Bearer $NAKO_ADMIN_TOKEN" \
  http://127.0.0.1:3000/admin/v1/overview
curl -H "Authorization: Bearer $NAKO_ADMIN_TOKEN" \
  http://127.0.0.1:3000/admin/v1/system/config
```

Diagnostics should be redacted. They may show schemes, counts, booleans, env var
names, and capability summaries; they must not expose raw tokens, DB passwords,
provider secrets, artifact paths, or source locators.

## 6. Backup Before Upgrade

Before upgrading:

1. Stop Nako or enter a maintenance window.
2. Back up the database.
3. Back up Managed Artwork artifact root.
4. Back up config and secret-manager entries.
5. Back up media/NFO sidecars through your normal media-library backup process.
6. Record the old release manifest and git revision.

See `docs/deployment/BACKUP_RESTORE_UPGRADE.md` for SQLite/PostgreSQL commands.

## 7. Upgrade

1. Verify new artifact checksums.
2. Read the new release manifest and release notes.
3. Take a fresh backup.
4. Replace the binary or pull/build the new image.
5. Run `config-check --create-dirs`.
6. Start Nako.
7. Check health, overview, system config, and relevant Admin diagnostics.

## 8. Rollback

Treat database migration rollback as restore-from-backup:

1. Stop Nako.
2. Restore pre-upgrade DB backup.
3. Restore matching artifact root and config if they changed.
4. Run the previous binary/image.
5. Verify health and diagnostics.

Do not run an older Nako binary against a newer migrated DB unless an explicit
release note says it is supported.

## 9. Support Bundle Expectations

A safe support bundle may include:

- release manifest,
- `SHA256SUMS`,
- command output from `config-check --json`,
- redacted Admin overview/system config JSON,
- compose config with secrets removed,
- logs around startup failure with tokens/passwords redacted,
- OS/container/runtime versions.

Never include:

- raw `NAKO_ADMIN_TOKEN`,
- raw `NAKO_DATABASE_URL` with password,
- provider tokens/API keys,
- Addon Tokens,
- Webhook secrets,
- raw local media paths unless explicitly needed and manually reviewed,
- DB dumps or media/NFO files unless a private support process has been agreed.
