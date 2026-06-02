# Database Guidelines

`nako` has no database responsibilities. Treat database isolation as part of
the public SDK contract.

## Required Patterns

- Keep all persistence, migrations, repository traits, and server-side database
  adapters outside this crate.
- Re-export only protocol or SDK crates that are safe for third-party addon and
  integration authors.
- Review dependency changes for accidental pulls of `nako-db`, `sqlx`, server,
  API, storage, or catalog internals.
- Keep crate metadata aligned with its permissive SDK purpose.

## Forbidden Patterns

- Do not depend on `nako-db`, `nako-server`, `nako-api`, `nako-core`,
  `nako-vfs`, or persistence adapters from this facade.
- Do not expose repository traits, migration helpers, database IDs, internal
  server records, or SQL-related feature flags.
- Do not use facade convenience as a shortcut for leaking implementation
  details to addon authors.

## Review Checklist

- Does the dependency graph still contain only public protocol or SDK crates?
- Does a new export make sense for third-party addon/integration authors?
- Would the exported type require server persistence knowledge to use?
- Is the behavior better placed in a dedicated public SDK crate?

## Evidence

- `crates/nako/Cargo.toml`
- `crates/nako/src/lib.rs`
