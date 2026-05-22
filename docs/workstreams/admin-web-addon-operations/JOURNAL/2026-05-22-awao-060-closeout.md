# AWAO-060 Closeout

Date: 2026-05-22

Closed Admin Web Addon Operations.

Completed:

- Added Addon Operations route constants and DTOs to the generated Admin API
  TypeScript contract.
- Deepened `apps/admin-web/src/adminApi` with typed Addon client methods,
  live/mock fallback, safe fixtures, and UI-oriented Addon read models.
- Rendered a live-capable Admin Web Addons surface for registered Addons,
  selected detail, health, grants/tokens summary, manifest surfaces,
  configuration schema metadata, and resource-call diagnostics.
- Wired enable/disable, health check, and resource-call diagnostic actions
  through the data-source seam.

Evidence:

- Rust/Admin API:
  - `cargo fmt --all -- --check`
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo check -p nako-api -p nako-server --tests`
  - `cargo nextest run -p nako-server addons --no-fail-fast`
- Admin Web:
  - `npm run check`
  - `npm test`
  - `npm run build`
- Hygiene:
  - `git diff --check`

Follow-ons:

- Addon Install Guide generation for Docker Compose/systemd snippets.
- Separate Addon Manager lane only if Nako will own discovery, install,
  update, package signing, or sidecar process supervision.
