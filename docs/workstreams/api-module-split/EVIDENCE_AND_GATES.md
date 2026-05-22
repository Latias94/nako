# nako-api Module Split Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- `crates/nako-api/src/lib.rs` contains Public Client mapping functions,
  protocol re-exports, server admin DTOs, metadata diagnostics, storage
  diagnostics, webhook DTOs, automation DTOs, addon DTOs, and unit tests.
- `nako-client-protocol` already owns the stable permissive Public Client API
  wire types and route inventory.
- `nako-api` remains the AGPL adapter/schema aggregation layer.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-api --tests
cargo check -p nako-api --examples
cargo nextest run -p nako-api --no-fail-fast
npm run check --prefix sdk/typescript
```

## Closeout Gates

```powershell
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M46.
- 2026-05-17: Added `public_client`, `admin`, `metadata_diagnostics`, and
  `extension` modules under `crates/nako-api/src`.
- 2026-05-17: Reduced `crates/nako-api/src/lib.rs` to module declarations and
  compatibility re-exports.
- 2026-05-17: Moved Public Client adapter tests into `public_client` and
  ingestion failure admin DTO tests into `admin`.
- 2026-05-17: Focused validation passed:
  - `cargo fmt --all -- --check`.
  - `cargo check -p nako-api --tests`.
  - `cargo check -p nako-api --examples`.
  - `cargo nextest run -p nako-api --no-fail-fast`: 12 tests passed.
  - `npm run check --prefix sdk/typescript`.
  - `cargo check --workspace --tests`.
- 2026-05-17: Closeout validation passed:
  - `cargo fmt --all -- --check`.
  - `cargo nextest run --workspace --no-fail-fast`: 293 tests passed.
  - `git diff --check`: passed with Git CRLF normalization warnings only.

## Closeout Evidence

- `nako-api` now has explicit module boundaries:
  - `public_client`: Public Client protocol exports and server model adapters.
  - `admin`: job, ingestion failure, and storage backend diagnostics.
  - `metadata_diagnostics`: metadata provider, maintenance, raw response, and
    cleanup DTOs.
  - `extension`: webhook, automation, and addon DTOs.
- `public_client.rs` does not contain admin, metadata diagnostics, storage
  diagnostics, webhook, automation, or addon DTO names.
- `nako-api` root-level imports remain compatible through `pub use`.
- OpenAPI and TypeScript SDK generation behavior remains unchanged.
