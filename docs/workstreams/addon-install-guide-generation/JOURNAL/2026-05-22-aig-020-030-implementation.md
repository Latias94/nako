# AIG-020/AIG-030 Implementation

Date: 2026-05-22

## Summary

Implemented the server-owned Addon Install Guide route and Admin Web preview.

## Server

- Added `AdminAddonInstallGuideResponse` DTOs in `taru-api`.
- Added generated Admin API route constant
  `addonInstallGuide: /admin/v1/addons/:addon_id/install-guide`.
- Added `GET /admin/v1/addons/{addon_id}/install-guide`.
- Generated Docker Compose and systemd snippets from the stored manifest.
- Generated Secret Reference placeholders, direct sidecar health-check steps,
  Taru Admin health-check steps, and registration/surface verification steps.
- Focused test proves no raw tokens, resolved secrets, local paths, Docker
  socket/process-control terms, or Source Locator/storage terms leak.

## Admin Web

- Added client/data-source/mock/read-model support for the guide.
- Rendered inert Docker Compose/systemd previews, Secret Reference checklist,
  health-check verification, registration verification, and lifecycle boundary
  copy in the Addon Operations panel.

## Evidence

- `cargo nextest run -p taru-server install_guide --no-fail-fast`
- `cargo run -q -p taru-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `npm run check`
- `npm test -- --run src/adminApi src/App.test.tsx`
- `npm run build`
