# Repository Seam Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Required Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-catalog --tests
cargo nextest run -p nako-catalog --no-fail-fast
cargo check -p nako-metadata --tests
cargo nextest run -p nako-metadata --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence To Record

- Workstream docs and top-level roadmap updates.
- `CatalogHydrationPort` interface and adapter implementation.
- Fake-port tests for hydration behavior.
- SQLite-backed tests proving existing catalog persistence behavior.
- Metadata and NFO caller-bound narrowing.
- Confirmation that public API, SDK, CLI, playback, NFO Round Trip, and DB
  schema were not changed.

## Closeout Evidence

- `crates/nako-catalog/src/lib.rs` defines `CatalogHydrationPort`,
  `CatalogHydrationSnapshot`, `CatalogHydrationLookup`, and
  `CatalogHydrationCommit`.
- `hydrate_item_catalog` depends on `CatalogHydrationPort` instead of the wide
  `CatalogRepository + MediaRepository + SearchIndex` caller contract.
- `nako-catalog` has a fake-port test for hydration behavior without SQLite.
- Existing SQLite-backed catalog hydration tests still pass.
- `nako-metadata` narrows metadata refresh and hierarchy confirmation bounds to
  the catalog hydration port.
- `nako-nfo` narrows import bounds to the catalog hydration port, and
  sidecar-discovery source listing now requires only `MediaRepository`.
- Focused validation so far:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -p nako-catalog --tests`: passed.
  - `cargo nextest run -p nako-catalog --no-fail-fast`: 3 tests passed.
  - `cargo check -p nako-metadata --tests`: passed.
  - `cargo nextest run -p nako-metadata --no-fail-fast`: 26 tests passed.
  - `cargo check -p nako-nfo --tests`: passed.
  - `cargo nextest run -p nako-nfo --no-fail-fast`: 8 tests passed.
  - `cargo check --workspace --tests`: passed.
  - `cargo nextest run --workspace --no-fail-fast`: 285 tests passed.
  - `git diff --check`: passed.
- Non-goals preserved: no public API, SDK, CLI, playback, NFO Round Trip, or
  database schema changes.
