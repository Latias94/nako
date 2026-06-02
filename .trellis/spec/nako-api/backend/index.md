# nako-api Backend Development Guidelines

These specs document current API contract patterns in `crates/nako-api`.
Read them before changing public or admin wire DTOs, route inventories, OpenAPI
generation, TypeScript SDK generation, or Admin Web contract generation.

## Pre-Development Checklist

- Read [Admin and Public Contracts](./admin-and-public-contracts.md) before API
  DTO or route inventory work.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Admin and Public Contracts](./admin-and-public-contracts.md) | DTO ownership, admin/public split, generated contracts, tests | Filled from code and ADRs |
| [Directory Structure](./directory-structure.md) | Module organization and file layout | To fill beyond contract slice |
| [Database Guidelines](./database-guidelines.md) | ORM patterns, queries, migrations | Not applicable unless contract work touches persistence |
| [Error Handling](./error-handling.md) | Error types, handling strategies | To fill |
| [Quality Guidelines](./quality-guidelines.md) | Code standards, forbidden patterns | To fill |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging, log levels | To fill |

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
