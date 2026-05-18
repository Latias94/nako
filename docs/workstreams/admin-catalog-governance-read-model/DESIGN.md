# M60 Design: Admin Catalog Governance Read Model

## Problem

The admin console catalog page currently has to rely on public browse/search
routes. Those routes are correct for client browsing, but they do not explain
why a Media Item is suspicious: unknown kind, weak Local Inference, missing
accepted Provider Mapping, or duplicate Source signals.

Without an Admin API read model, a future console would either overuse public
client DTOs or duplicate local inference/provider lookup logic in the UI.

## Target State

M60 introduces:

```text
GET /admin/v1/catalog/governance/items
```

The route lists one row per Media Item and Media Library pair when the item is
unknown or its best Local Inference confidence is at or below the requested
threshold. The row includes:

- Media Item ID, kind, parent ID, title, release date;
- Media Library ID;
- source count and representative Media Source ID/file name;
- redacted Local Inference summary without raw evidence value;
- Provider Mapping counts and accepted Provider Mapping count;
- duplicate Source relationship count;
- computed issue codes for admin triage.

## Architecture Direction

The implementation adds a narrow `CatalogGovernanceRepository` port in
`taru-core`. `taru-db` owns the SQL joins across Media Items, Media Sources,
Local Inference Evidence, Provider Mappings, and Source Duplicate
Relationships.

`taru-server` stays thin:

- query parsing and auth live in the HTTP adapter;
- `CatalogAppService` calls the repository port;
- `taru-api::admin` maps records to redacted DTOs.

This keeps graph lookup internals out of handlers and preserves the Public
Client API boundary.

## Redaction Rules

The Admin DTO may expose IDs, titles, representative file name, confidence,
inferred fields, provider/source counts, and issue categories.

It must not expose:

- source locator;
- local filesystem path;
- raw Local Inference `evidence_value`;
- raw NFO sidecar path;
- provider raw response body;
- secret or token values.

## Future Follow-Ups

- Direct item-to-sources admin diagnostics route.
- Provider Mapping list/detail read model.
- Local Inference evidence detail route with explicit redaction policy.
- Duplicate Source review queue.
- NFO sidecar status read model.
- Repair/rematch mutations after audit and conflict semantics are documented.
