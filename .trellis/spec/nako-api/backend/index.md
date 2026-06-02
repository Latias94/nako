# nako-api Backend Development Guidelines

These specs document current API contract patterns in `crates/nako-api`.
Read them before changing public or admin wire DTOs, route inventories, OpenAPI
generation, TypeScript SDK generation, or Admin Web contract generation.

## Pre-Development Checklist

- Read [Admin and Public Contracts](./admin-and-public-contracts.md) before API
  DTO or route inventory work.
- Read [Directory Structure](./directory-structure.md) before adding an Admin,
  Public Client, OpenAPI, or generated-contract module.
- Read [Quality Guidelines](./quality-guidelines.md) before changing generated
  contract artifacts or redaction-sensitive DTOs.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Admin and Public Contracts](./admin-and-public-contracts.md) | DTO ownership, admin/public split, generated contracts, tests | Filled from code and ADRs |
| [Directory Structure](./directory-structure.md) | Admin/Public DTO, OpenAPI, SDK, and contract generator layout | Filled from code |
| [Database Guidelines](./database-guidelines.md) | Persistence non-ownership for API contracts | Filled as not-applicable boundary |
| [Error Handling](./error-handling.md) | Error DTO ownership and server-side mapping boundary | Filled from ADR 0023 |
| [Quality Guidelines](./quality-guidelines.md) | Generated contract, redaction, and route inventory gates | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | API crate no-runtime/logging boundary | Filled as no-runtime boundary |

## Authority / Evidence

- ADR 0023: public API version and stable error envelope.
- ADR 0025: OpenAPI/Public Client SDK generated from protocol-owned wire types.
- ADR 0027: versioned Admin API boundary for the web console.
- ADR 0053: bounded API scale contracts and redacted diagnostics.
- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/admin/*.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
