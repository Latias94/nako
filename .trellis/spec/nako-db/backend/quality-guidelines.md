# Quality Guidelines

Persistence work must prove repository behavior, not only compile.

## Required Patterns

- Add or extend `contract_tests.rs` for every repository behavior that should
  hold across SQLite and Postgres.
- Include both fresh-store and migrated-store cases when the schema change
  affects migration behavior.
- Use `NAKO_TEST_POSTGRES_URL` for optional Postgres contract runs. Do not make
  local development require Postgres for SQLite-only checks.
- Keep SQLite and Postgres row shape comparable even when SQL dialect differs.
- Keep list methods bounded with `PageRequest`.

## Gate Selection

- Narrow DB compile:
  `cargo check -p nako-db --tests`
- Focused DB behavior:
  `cargo nextest run -p nako-db <filter> --no-fail-fast`
- Cross-crate repository contract:
  `cargo nextest run -p nako-core -p nako-db <filter> --no-fail-fast`
- Docs-only spec update:
  `git diff --check`

## Forbidden Patterns

- Do not skip Postgres adapter updates by hiding new behavior behind SQLite
  helper methods.
- Do not use live external services in contract tests other than the optional
  Postgres URL path already modeled in the crate.
- Do not add unbounded scans or list queries for Admin/Public surfaces.
- Do not store raw secrets, playback tickets, or raw local paths in diagnostic
  columns unless an ADR and redaction contract explicitly require it.

## Review Checklist

- Migration file exists for SQLite and Postgres.
- Both migration arrays include the same version.
- Inserts, updates, row readers, list filters, and contract tests all know about
  the new field.
- Stored enum conversions reject unknown values.
- The new SQL preserves existing pagination, ordering, and transaction rules.
