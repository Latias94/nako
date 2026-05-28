# Addon Resource Link Check Product Flow

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

Nako now has a first-class addon `resource_link_check` protocol and client
helper, but product callers still have no host-owned route that can check a
selected resource-search link without sending raw URLs or passwords from the
browser.

## Relevant Authority

- ADR:
  - `docs/adr/0050-acquisition-resource-action-boundaries.md`
- Related workstreams:
  - `docs/workstreams/addon-resource-link-check-contract/`
  - `docs/workstreams/addon-resource-search-product-flow/`

## Problem

The browser has opaque `search_id` and `selection_id` values from resource
search. It should be able to ask the host to check that selected link, but it
must not send raw URL/password material back to Nako. The host must retrieve the
selected link from its transient session and call a declared
`resource_link_check` addon resource under a separate scope.

## Target State

- Admin API exposes a product route for link checking a selected resource-search
  link by opaque ids.
- Request body does not accept raw URL, password, or arbitrary context.
- Host calls `call_addon_resource_link_check_with_outcome`.
- Response exposes safe facts only.
- Existing intake selection route remains unchanged.

## In Scope

- Nako API DTOs and admin contract.
- Nako server app service and HTTP route.
- Tests proving opaque-id flow, addon call shape, scope behavior, and no raw
  URL/password leakage.

## Out Of Scope

- Admin UI.
- Real checker addon/provider implementation.
- Downloader execution.
- Cloud-drive transfer.
- Durable password/code persistence.

## Architecture Direction

Add a sibling route under the resource-search session path:

```text
POST /admin/v1/addons/{addon_id}/resource-search/{search_id}/selections/{selection_id}/link-check
```

The route consumes:

- path ids: `addon_id`, `search_id`, `selection_id`;
- body: `refresh` only.

The app service retrieves the raw selected link from `ResourceSearchSessionStore`
and creates an `AddonResourceLinkCheckRequest` for the addon. The browser never
submits raw link material.

## Closeout Condition

This lane can close when:

- API/server route exists,
- route uses opaque selection ids only,
- tests prove no raw URL/password leaks in responses,
- targeted gates pass,
- and follow-on checker implementation/UI work is deferred.
