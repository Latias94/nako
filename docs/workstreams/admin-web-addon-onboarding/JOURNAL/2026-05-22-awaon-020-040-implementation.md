# 2026-05-22 AWAON-020-040 Implementation

Implemented the Admin Web Addon onboarding vertical slice:

- generated Admin API contract now includes `RegisterAddonRequest` and fuller
  manifest declaration types needed by pasted manifest registration;
- `AdminApiClient.registerAddon` posts to `/admin/v1/addons` with
  `status: "disabled"` and no granted scopes by default;
- `createAdminDataSource` can preview pasted manifest JSON and register it,
  returning a safe onboarding handoff state;
- `App.tsx` renders an Addon Onboarding panel with JSON paste, local preview,
  disabled registration, and next-step handoff to Install Guide / sidecar start
  / Health Check;
- docs explain registration is not installation and sidecar reachability belongs
  to Health Check.

Focused evidence:

- `npm test -- --run src/adminApi/client.test.ts`
- `npm test -- --run src/adminApi/dataSource.test.ts`
- `npm test -- --run src/App.test.tsx`
- `npm run check`
