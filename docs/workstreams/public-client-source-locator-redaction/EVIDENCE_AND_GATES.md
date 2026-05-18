# Public Client Source Locator Redaction Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http
```

Current known public exposure anchors include:

- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`

## Gate Set

### Audit Gate

```powershell
rg "locator|input_locator" crates/taru-client-protocol crates/taru-api crates/taru-server/src/http
git diff --check
```

Proves the exposure inventory is current before public DTO changes.

### Public DTO Gate

```powershell
cargo check -p taru-client-protocol --tests
cargo check -p taru-api --tests
cargo nextest run -p taru-server <public-route-filter> --no-fail-fast
```

Proves protocol/server mapping changes compile and public route JSON tests
protect the redaction behavior.

### Contract Sync Gate

```powershell
cargo nextest run -p taru-api --no-fail-fast
```

Add existing OpenAPI and SDK generation checks from the client contract lanes
when DTO or generated artifacts change.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to workspace gates if protocol changes affect SDK/client crates.

### Review Gate

Run `review-workstream` before accepting DTO changes and before closeout.
Record blocking findings, missing gates, and residual risks here.

## Evidence Anchors

- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/api/HTTP_API.md`
- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-server/src/http/tests`

## Fresh Evidence

2026-05-18, PCLR-010:

- Workstream opened from ARF-005.
- Known locator exposure anchors identified with `rg`.
- First executable task set to exposure audit and contract decision before DTO
  field removal.

Fresh verification is required before marking implementation tasks or the lane
complete.
