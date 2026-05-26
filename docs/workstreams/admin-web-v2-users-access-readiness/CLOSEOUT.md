# Admin Web V2 Users Access Readiness - Closeout

Status: Complete
Closed: 2026-05-26

## Delivered

- Redaction-safe Admin API route:
  - `GET /admin/v1/access/summary`.
- Generated Admin Web contract route and DTO coverage for the access summary.
- HTTP API documentation for the Single-Admin Mode access summary.
- Server coverage proving the current `local-admin` principal, Single-Admin
  Mode, effective access to configured Media Libraries, readiness states, and
  redaction of token/path/source details.
- Admin Web client, data source, mock data, and type exports for the access
  summary.
- Route-owned Admin Web page:
  - `/access`.
- Navigation entry for Users & Access.
- Live/mock source truth, current principal, auth summary, readiness, effective
  Library Access, and mutation-readiness notice.
- UI tests for live rendering, mock fallback, and unsafe-field exclusion.

## Verification

- `cargo fmt --all --check`
- `cargo nextest run -p nako-server admin_v1_access_summary --no-fail-fast`
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
- `cargo test -p nako-api admin_web_generated_contract_matches_generator_output -- --nocapture`
- `cd apps/admin-web && npm run generate:admin-api`
- `cd apps/admin-web && npm run test -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`
- `cd apps/admin-web && npm run check`
- `cd apps/admin-web && npm run build`
- `cd apps/admin-web && npm run test`
- `python -m json.tool docs\workstreams\admin-web-v2-users-access-readiness\WORKSTREAM.json`
- `git diff --check`
- Browser smoke via Playwright CLI for `/access` at desktop width and 390px
  mobile width.
- Browser smoke with intercepted Admin API JSON proving the page renders `Live
  Admin API` source state when `GET /admin/v1/access/summary` succeeds.

## Review

No blocking workstream-compliance or code-quality findings remain for this
lane. The shipped route follows ADR 0024 and ADR 0028: inbound bearer auth stays
separate from outbound secrets, accepted requests resolve to `local-admin` in
Single-Admin Mode, and token values or env var names are not exposed.

The implementation is intentionally read-only. It does not add account
persistence, Role assignment, per-library policy storage, Public Client API
shape, playback-client account switching, or fake Admin Web mutations.

## Follow-On Split

Open a new backend-authority lane before adding mutation controls for:

- user account persistence and identity provider integration;
- Role model and Role assignment;
- per-user or per-role Library Access policy storage;
- migration from Single-Admin Mode to account-backed principals;
- Admin Web user, Role, and Library Access editing flows;
- audit records for account or access-policy changes.

## Residual Risk

- The page is a readiness/effective-access diagnostic only. Operators cannot
  create accounts or edit Library Access until a backend policy model exists.
- Browser live smoke used route-intercepted JSON because no live backend Admin
  API was running behind the local Vite dev server.
- `npm run build` reports existing Vite chunk-size/plugin-timing warnings. The
  build succeeds; code-splitting remains a broader Admin Web performance task.
