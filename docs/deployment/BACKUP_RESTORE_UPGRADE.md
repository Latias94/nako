# Backup, Restore, And Upgrade Runbook

Status: Draft release baseline

This runbook defines the current Taru self-hosted backup, restore, and upgrade
operator procedure for SQLite and PostgreSQL deployments.

## State Classification

Durable state you must protect:

| State | Example | Why it matters |
| --- | --- | --- |
| Database | SQLite `taru.db` or PostgreSQL `taru` database | Holds libraries, catalog, metadata, jobs, Addons, Webhooks, playback sessions, Managed Artwork records, and staging manifests. |
| Managed Artwork artifact root | `artwork.artifact_root` | Holds Taru-owned artwork bytes referenced by DB records. |
| Media libraries | `[[libraries]].root` | Source media files and user-owned sidecars. |
| NFO sidecars and NFO backups | `.nfo` files beside media and backup files created by Taru write policy | User-visible metadata exchange and recovery state. |
| Secrets | `TARU_ADMIN_TOKEN`, provider tokens, WebDAV password, PostgreSQL password, Webhook secrets, Addon Tokens | Required to authenticate clients and integrations. |
| Config | `taru.toml`, service manager units, reverse-proxy config | Defines paths, DB backend, auth, providers, playback, and resource budgets. |

Cache or rebuildable state:

| State | Example | Handling |
| --- | --- | --- |
| Remux/HLS outputs | `remux_staging_root` | Can be deleted while Taru is stopped; sessions may be recreated. |
| Remote probe/FFmpeg input staging | `[staging]` manifest-tracked files | Can be cleaned by startup cleanup or manually while stopped. |
| Provider raw-cache rows | DB rows governed by metadata raw-cache retention | Useful for diagnostics but not the source of truth. |
| `target/` release evidence | `target/release-gate`, `target/postgres-contract` | Developer-only; never part of production backup. |

Rule of thumb: back up DB, artifact root, config, secrets, media/NFO sidecars;
do not back up generated remux/HLS/cache directories unless you have a special
forensics need.

## Pre-Backup Checklist

1. Stop Taru or put it into a maintenance window.
2. Record the current git commit or release build identifier.
3. Record the database backend and `database_url` host/path, but do not paste
   raw passwords into tickets or logs.
4. Record:
   - `artwork.artifact_root`
   - `remux_staging_root`
   - every `[[libraries]].root`
   - NFO sidecar backup policy location
   - secret environment variable names
5. Run the local release gate when practical:

```bash
bash scripts/release-gate.sh --mode fast
```

or:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
```

## SQLite Backup

Stop Taru before copying the SQLite database and artifact root.

```bash
systemctl stop taru
install -d -m 0700 /backup/taru/$(date +%F)
sqlite3 /var/lib/taru/taru.db ".backup '/backup/taru/$(date +%F)/taru.db'"
tar -C /var/lib/taru -czf /backup/taru/$(date +%F)/artwork.tar.gz artwork
cp /etc/taru/taru.toml /backup/taru/$(date +%F)/taru.toml
systemctl start taru
```

If `sqlite3` is unavailable, copying the DB file while Taru is stopped is
acceptable. Include any `-wal` and `-shm` files if you copy instead of using
`.backup`.

PowerShell sketch:

```powershell
$stamp = Get-Date -Format yyyy-MM-dd
New-Item -ItemType Directory -Force "C:\Backups\Taru\$stamp" | Out-Null
Copy-Item C:\Taru\data\taru.db "C:\Backups\Taru\$stamp\taru.db"
Compress-Archive -Path C:\Taru\data\artwork -DestinationPath "C:\Backups\Taru\$stamp\artwork.zip"
Copy-Item C:\Taru\taru.toml "C:\Backups\Taru\$stamp\taru.toml"
```

## SQLite Restore

1. Stop Taru.
2. Move the current DB and artifact root aside; do not overwrite them in place.
3. Restore the DB file.
4. Restore `artwork.artifact_root`.
5. Restore `taru.toml` and secret environment variables.
6. Start Taru and check health plus Admin diagnostics.

```bash
systemctl stop taru
mv /var/lib/taru/taru.db /var/lib/taru/taru.db.before-restore
mv /var/lib/taru/artwork /var/lib/taru/artwork.before-restore
cp /backup/taru/2026-05-21/taru.db /var/lib/taru/taru.db
tar -C /var/lib/taru -xzf /backup/taru/2026-05-21/artwork.tar.gz
systemctl start taru
curl http://127.0.0.1:3000/health
```

Run Managed Artwork drift diagnostics after restore. Missing artifact files
mean DB records and artifact root are inconsistent.

## PostgreSQL Backup

Stop Taru or ensure no schema migrations are running.

```bash
systemctl stop taru
install -d -m 0700 /backup/taru/$(date +%F)
pg_dump --format=custom --file=/backup/taru/$(date +%F)/taru.pgcustom "$TARU_DATABASE_URL"
tar -C /var/lib/taru -czf /backup/taru/$(date +%F)/artwork.tar.gz artwork
cp /etc/taru/taru.toml /backup/taru/$(date +%F)/taru.toml
systemctl start taru
```

If using the compose example, run `pg_dump` from a host with PostgreSQL client
tools or from a temporary client container attached to the same network.

## PostgreSQL Restore

1. Stop Taru.
2. Create a new empty database or move the old one aside.
3. Restore with `pg_restore`.
4. Restore `artwork.artifact_root`.
5. Restore config and secrets.
6. Start Taru and run diagnostics.

```bash
systemctl stop taru
createdb taru_restore
pg_restore --dbname=taru_restore --clean --if-exists /backup/taru/2026-05-21/taru.pgcustom
tar -C /var/lib/taru -xzf /backup/taru/2026-05-21/artwork.tar.gz
systemctl start taru
curl http://127.0.0.1:3000/health
```

Prefer restoring into a new DB and switching `database_url` after validation
instead of destructive restore over the only production DB.

## NFO Sidecars

NFO sidecars live with media library files. Taru treats them as user-visible
library state, not cache. Include them in media library backups.

When restoring:

- Restore media files and NFO sidecars together.
- Restore Taru DB and artifact root from the same backup window when possible.
- If DB and NFO sidecars come from different points in time, run a deliberate
  NFO import/export reconciliation instead of assuming one silently wins.

## Secrets

Secrets are not serialized into the redacted Admin config response. Back up the
secret manager entries or service environment definitions separately:

- `TARU_ADMIN_TOKEN`
- `TARU_POSTGRES_PASSWORD`
- `TMDB_READ_ACCESS_TOKEN`
- `BANGUMI_TOKEN`
- `DOUBAN_API_KEY`
- `TARU_WEBDAV_PASSWORD`
- Webhook secret env values
- Addon Token issuance records and any sidecar-side stored token values

Never put raw secret values in `docs/`, workstream evidence, CI logs, or issue
comments.

## Upgrade Procedure

1. Read release notes and migration notes for the target commit.
2. Stop Taru.
3. Take a fresh backup using the backend-specific procedure above.
4. Build or deploy the new Taru binary.
5. Keep `taru.toml` under version control or a config-management system and
   review path/backend/resource changes.
6. Start Taru. Startup runs migrations through the configured database backend.
7. Check:
   - `GET /health`
   - `GET /admin/v1/overview`
   - `GET /admin/v1/system/config`
   - release gate relevant to the change, usually `fast` plus `postgres` if
     PostgreSQL is enabled.

## Migration Rollback And Forward Expectations

Taru migrations are expected to move forward with the running binary. Treat DB
migration rollback as restore-from-backup, not as an automatic down migration.

Rollback procedure:

1. Stop Taru.
2. Restore the pre-upgrade DB backup.
3. Restore matching artifact root and config if they changed.
4. Run the previous Taru binary.
5. Verify health and diagnostics.

Forward recovery procedure:

1. Fix the failing deployment or binary.
2. Restore the last known-good pre-upgrade backup if migration state is
   ambiguous.
3. Re-run the upgrade from a clean state.

Do not run an older binary against a newer migrated DB unless an explicit
release note says it is supported.

## Artifact Root Consistency

Managed Artwork DB rows reference files under `artwork.artifact_root`. A
consistent backup captures DB and artifact root from the same maintenance
window.

After restore or migration:

- Use Admin Managed Artwork storage drift diagnostics to find missing or stray
  artifact files.
- Missing artifact files indicate the DB points to bytes that were not
  restored.
- Stray files indicate files not referenced by active DB records. Use
  remediation only after reviewing the dry-run report.

## Post-Restore Verification

Run at minimum:

```bash
bash scripts/release-gate.sh --mode docs
bash scripts/release-gate.sh --mode fast
```

For PostgreSQL deployments:

```bash
bash scripts/postgres-contract-harness.sh --suite managed-artwork --database-url "$TARU_TEST_POSTGRES_URL"
```

Operator smoke checks:

```bash
curl http://127.0.0.1:3000/health
curl -H "Authorization: Bearer $TARU_ADMIN_TOKEN" http://127.0.0.1:3000/admin/v1/overview
curl -H "Authorization: Bearer $TARU_ADMIN_TOKEN" http://127.0.0.1:3000/admin/v1/system/config
```
