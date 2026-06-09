# Postgres catalog access DISTINCT order fix

## Goal

Fix PostgreSQL `catalog_access` contract failures exposed by the opt-in Docker-backed Postgres gate after root catalog aggregate projection work.

## What I already know

- `postgres_catalog_access_contract_filters_root_aggregates_before_pagination` passed against PostgreSQL 17.
- Existing Postgres item and relation-item access contracts failed with:
  - `for SELECT DISTINCT, ORDER BY expressions must appear in select list`
- The failing queries select `items.id::text AS id` / `mi.id::text AS id` but order by `items.id` / `mi.id`.
- SQLite focused gates passed because SQLite accepts this query shape.

## Requirements

- Fix PostgreSQL accessible item and accessible relation-item queries so `SELECT DISTINCT` order expressions are valid in PostgreSQL.
- Preserve existing ordering semantics: title ascending, id ascending.
- Avoid schema changes and avoid changing SQLite unless needed.
- Re-run the focused Postgres `catalog_access` ignored contracts against the running Docker database.

## Acceptance Criteria

- [ ] `postgres_catalog_access_contract_filters_items_before_pagination` passes.
- [ ] `postgres_catalog_access_contract_filters_relation_items_before_pagination` passes.
- [ ] `postgres_catalog_access_contract_filters_root_aggregates_before_pagination` remains passing.
- [ ] `cargo check -p nako-db --tests` passes if code changes require compile confirmation.
- [ ] `git diff --check` passes.

## Technical Notes

- Temporary Docker PostgreSQL URL used for reproduction:
  - `postgres://nako:nako@127.0.0.1:28841/nako`
- Suspect files:
  - `crates/nako-db/src/postgres/core_catalog.rs`
  - `crates/nako-db/src/postgres/metadata_catalog.rs`
