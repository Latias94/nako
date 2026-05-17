# Catalog Hydration Lookup Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p taru-catalog --tests
cargo nextest run -p taru-catalog --no-fail-fast
cargo check -p taru-metadata --tests
cargo nextest run -p taru-metadata strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite --no-fail-fast
cargo check -p taru-nfo --tests
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence To Record

- New `CatalogHydrationPort` shape.
- Confirmation that non-catalog crates no longer import lookup/snapshot/commit
  implementation types.
- Metadata fake-port test simplification.
- Existing catalog graph/search behavior still passing.
- Confirmation that public API, SDK, client, and DB schema did not change.

## Closeout Evidence

- `CatalogHydrationPort` now exposes `hydrate_catalog`.
- `CatalogHydrationSnapshot`, `CatalogHydrationLookup`, and
  `CatalogHydrationCommit` are internal to `taru-catalog`.
- Metadata fake-port tests now assert workflow hydration without constructing
  lookup vectors.
- Existing catalog graph/search behavior still passes through SQLite-backed
  hydration tests.
- Public API, SDK, client, and DB schema did not change.
- Validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p taru-catalog --tests`: passed.
  - `cargo nextest run -p taru-catalog --no-fail-fast`: 3 tests passed.
  - `cargo check -p taru-metadata --tests`: passed.
  - `cargo nextest run -p taru-metadata strategy::port_tests::refresh_service_uses_refresh_and_hydration_ports_without_sqlite --no-fail-fast`: 1 test passed.
  - `cargo check -p taru-nfo --tests`: passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 288 tests passed.
  - `git diff --check`: passed.
