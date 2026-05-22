# 2026-05-22 AWAON-050 Closeout

Closed `admin-web-addon-onboarding`.

Outcome:

- Admin Web can paste and preview Addon manifest JSON.
- The registration action posts to `/admin/v1/addons` as `disabled` with no
  grants by default.
- Server registration now permits disabled registrations without granted scopes,
  while enabled registration still requires every resource scope to be granted.
- The UI explicitly states registration does not install or start sidecars and
  hands the operator to Install Guide, external sidecar start, and Health Check.
- Docs and top-level trackers record the completed boundary.

Closeout evidence:

- `cargo fmt --all -- --check`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `cargo nextest run -p taru-server register_addon_routes_disabled_by_default_and_validate_contract --no-fail-fast`
- `cargo check -p taru-api -p taru-server --tests`
- `npm run check`, `npm test`, `npm run build` in `apps/admin-web`
- `git diff --check`

Recommended next follow-on: Addon token/grant onboarding UX before URL-based
manifest discovery.
