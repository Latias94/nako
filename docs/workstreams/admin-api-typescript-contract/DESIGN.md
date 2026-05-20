# Admin API TypeScript Contract Design

Status: Active
Last updated: 2026-05-20

## Contract Boundary

The Admin API TypeScript contract is an internal web-console integration
contract. It is not the Public Client SDK and should not be treated as a
permissive client protocol surface.

Boundary rules:

- source DTO ownership stays in `taru-api`;
- route inventory is limited to accepted `/admin/v1/*` routes;
- `taru-client-protocol` remains public-client-only;
- `sdk/typescript` remains the Public Client SDK package;
- admin-web can import app-local generated admin types or a future admin
  package, but not public SDK types for admin-only diagnostics.

## Covered First Routes

The first contract slice should cover the existing AWC-070 read models:

- `GET /admin/v1/overview`
- `GET /admin/v1/catalog/governance/items`
- `GET /admin/v1/events`
- `GET /admin/v1/jobs`
- `GET /admin/v1/playback/sessions`
- `GET /admin/v1/playback/runtime`
- `GET /admin/v1/storage/staging`
- `GET /admin/v1/system/config`

Mutation routes, detail routes, settings editing, catalog repair, and addon
token lifecycle operations stay out of the first contract slice unless a
separate task accepts their server semantics.

## Artifact Location Decision

Accepted AATC-020 direction: generate an app-local artifact under
`apps/admin-web/src/adminApi/generated/contract.ts`.

Rationale:

- `apps/admin-web` is currently the only consumer.
- Publishing a reusable admin SDK would create packaging and compatibility
  obligations before Taru has a stable Admin API.
- App-local generation still lets Rust tests compare generated output and
  prevents silent DTO drift.
- A later package lane can move the artifact if multiple admin clients appear.

## Generator Direction

The public TypeScript SDK uses `crates/taru-api/src/sdk.rs` and the public
OpenAPI document. Admin generation should reuse small schema-conversion helpers
where practical, but keep its route inventory and leakage tests separate.

Accepted first implementation:

- a focused `taru-api` generator entry point, preferably in a new
  `admin_contract` module rather than in the public SDK generator;
- a new example such as `emit-admin-typescript-contract`;
- generated TypeScript interfaces, query interfaces, and route constants;
- no generated fetch client in the first slice;
- a sync test comparing generated output with the committed app-local file.

Do not create a combined public+admin OpenAPI or combined SDK. That would undo
the accepted ADR boundary.

## Redaction Rules

The Admin API is operationally richer than the Public Client API, but the
generated contract must still prove it does not encode sensitive raw fields.

Forbidden generated terms include:

- plaintext secret values, tokens, bearer credentials, webhook secrets, addon
  tokens, or provider API keys;
- `source_uri`, `cache_uri`, `storage_uri`, `output_path`, `local_path`, and
  raw transcode output paths unless a future route explicitly accepts a
  redacted admin-only path policy;
- raw provider response bodies outside explicit diagnostics contracts;
- addon-hosted pages as trusted first-party UI.

The exact forbidden-term test list should be maintained with the generator so
new admin routes cannot quietly weaken the web-console contract.

## Migration Strategy For Admin Web

The current `apps/admin-web/src/adminApi/types.ts` should shrink after the
generated contract lands. It may keep UI-only summary types, source-map types,
and view models, but wire DTOs for covered `/admin/v1/*` routes should come
from the generated contract.

The `client.ts` boundary should keep a hand-written fetch wrapper in the first
slice. It should import generated response types and route constants, while
base URL normalization, bearer auth, request failure behavior, and
section-level fallback stay app-owned.

Generated query interfaces should exist for the covered list routes even if
the first UI wiring still calls them without filters. Filters are the next
admin-web workflow, and query names are part of the HTTP contract.

## AATC-020 Decision

See [ADMIN_CONTRACT_INVENTORY.md](ADMIN_CONTRACT_INVENTORY.md) for the route
and DTO audit.

The chosen artifact shape is **route constants + wire interfaces + query
interfaces**. Interfaces-only generation leaves route strings duplicated; a
generated client takes too much ownership of admin-web runtime policy.

## Follow-On UI Work

After this contract lands, the next admin-web implementation slices should be:

- Jobs filters and job detail entry point;
- Catalog Governance filters and item review detail;
- Playback sessions filters and session detail;
- Settings diagnostics layout polish before editable settings.

## Generation And Separation Commands

Refresh the app-local Admin API contract from the repository root with:

```bash
cd apps/admin-web
npm run generate:admin-api
```

The script writes `apps/admin-web/src/adminApi/generated/contract.ts` from the
`taru-api` Admin Contract generator. The generated file is intentionally
app-local and must not be edited by hand.

Verify the Public Client SDK remains separate with:

```bash
npm run generate --prefix sdk/typescript
npm run check --prefix sdk/typescript
git diff --name-only -- crates/taru-client-protocol sdk/typescript
```

For this lane, `git diff --name-only -- crates/taru-client-protocol
sdk/typescript` should stay empty after regeneration/checking. Admin API routes
belong in `TARU_ADMIN_ROUTES`, not in Public Client route inventory or
`@taru/sdk`.
