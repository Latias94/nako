# Directory Structure

`nako-api` owns wire contracts and generated contract inventories. It does not
own HTTP handler logic, database queries, app services, or Admin Web runtime
state.

## Current Layout

```text
crates/nako-api/src/
├── lib.rs                 # API crate exports
├── public_client.rs       # Public Client DTOs and route inventory
├── admin.rs               # Admin API root DTO module
├── admin/                 # Admin area DTOs
├── admin_contract.rs      # generated Admin route/type inventory source
├── openapi.rs             # OpenAPI generation
├── sdk.rs                 # public SDK generation helpers
├── metadata_diagnostics.rs
└── extension.rs
```

## Module Rules

- Put `/admin/v1/*` request/response DTOs under `admin.rs` or `admin/*.rs`.
- Put stable Public Client API DTOs and route inventory in `public_client.rs`.
- Keep generated Admin Web contract source in `admin_contract.rs`; regenerate
  `apps/admin-web/src/adminApi/generated/contract.ts` from the generator.
- Keep OpenAPI and SDK generation under `openapi.rs` and `sdk.rs`.
- Keep server-side route handlers and auth in `nako-server`, not here.

## Forbidden Placement

- Do not add SQL, repository calls, app services, or Axum extractors here.
- Do not put Admin routes in the Public Client route inventory.
- Do not hand-edit generated TypeScript contract output.
- Do not expose raw internal records, local paths, tokens, playback tickets, or
  provider cache payloads in DTOs.

## Examples

- `admin/metadata_candidate_review.rs`: Admin DTOs for Candidate Review
  governance.
- `admin/playback.rs`: Admin playback DTOs separate from pure planner records.
- `admin_contract.rs`: generator-owned Admin route inventory.
- `public_client.rs`: Public Client API contract surface.
