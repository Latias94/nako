# Addon Resource Link Check Product Flow Closeout

Status: Closed
Closed: 2026-05-28

## Delivered

- Added `AdminAddonResourceLinkCheckRequest` and
  `AdminAddonResourceLinkCheckResponse`.
- Added generated Admin TypeScript contract entries for the link-check route,
  scope, resource, status enum, request, and response.
- Added a product route:
  `POST /admin/v1/addons/{addon_id}/resource-search/{search_id}/selections/{selection_id}/link-check`.
- Implemented host-owned lookup of the selected resource link by opaque
  resource-search ids.
- Called the addon `resource_link_check` resource through the existing addon
  client outcome path.
- Added tests proving that browser requests cannot carry raw URL/password/context
  fields and that product responses do not leak raw link material, passwords,
  provider messages, local source locators, tokens, or private images.

## Verification

- `cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-api admin_resource_link_check_response_uses_safe_facts_only --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests`
- `git diff --check`

## Follow-Ons

- Admin UI for invoking and presenting link checks.
- Real checker addon/provider implementations.
- Downloader and cloud-drive transfer action contracts.
- Durable password/code reference policy.
