# 0009: Resolve Provider Secrets from Environment References

## Status

Accepted.

## Context

Nako needs external metadata and automation providers, starting with TMDB. These
providers require user-managed credentials. Persisted jobs, logs, and raw
provider cache records must remain auditable without storing plaintext API
secrets.

The first server milestone does not yet have a full secret manager, user
accounts, encrypted configuration store, or UI for rotating provider
credentials.

## Decision

Provider configuration stores references to environment variables, not the
secret values themselves.

For TMDB, `NakoServerConfig` stores:

- whether the provider is enabled
- the environment variable name for the read access token
- provider API and image base URLs
- language and adult-content settings

At runtime, the server resolves the environment variable immediately before
constructing the provider client. Job inputs and summaries may include provider
names, item IDs, language, matched keys, and status, but they must not include
the resolved secret.

## Consequences

- SQLite jobs and provider raw responses remain free of plaintext API tokens.
- Local deployments can use shell profiles, service managers, containers, or
  OS-level secret injection.
- The server can fail fast with an explicit configuration error when a provider
  is enabled but its environment variable is missing or empty.
- Future secret storage can replace environment resolution without changing the
  metadata refresh job contract.

## Alternatives Considered

- Store plaintext provider tokens in `nako.toml`: rejected because config files
  are often copied into issue reports, examples, and backups.
- Store plaintext provider tokens in SQLite: rejected because job and provider
  tables are designed for auditability and should not become secret stores.
- Require a full secret manager in M3.2: deferred because it would block the
  metadata provider MVP and is better handled with authentication and admin UI
  work.

## Related Workstreams

- `docs/workstreams/server-foundation/PHASE3_2_TMDB_METADATA_REFRESH.md`
- `docs/workstreams/server-foundation/TODO.md`
