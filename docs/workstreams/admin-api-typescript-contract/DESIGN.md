# Admin API TypeScript Contract Design

Status: Active
Last updated: 2026-05-19

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

Default direction: generate an app-local artifact under
`apps/admin-web/src/adminApi/generated`.

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

Acceptable first implementation:

- a new `taru-api` generator module or focused functions in `sdk.rs`;
- a new example such as `emit-admin-typescript-contract`;
- generated TypeScript interfaces and route constants;
- optional tiny fetch client if it removes duplicated route paths from
  admin-web;
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

The `client.ts` boundary can either:

- keep a hand-written fetch wrapper that imports generated response types; or
- use a generated admin client runtime for covered routes.

The first option is lower-risk. The second option is worthwhile if route paths
or query encoding start duplicating across the app.

## Follow-On UI Work

After this contract lands, the next admin-web implementation slices should be:

- Jobs filters and job detail entry point;
- Catalog Governance filters and item review detail;
- Playback sessions filters and session detail;
- Settings diagnostics layout polish before editable settings.
