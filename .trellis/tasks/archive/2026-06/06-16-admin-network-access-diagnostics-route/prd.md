# Admin Network Access Diagnostics Route

## Goal

Expose U6 remote endpoint readiness as a dedicated redacted Admin API route so
operators and Admin Web can consume network access posture without pulling the
full system config diagnostic payload.

## Requirements

- Add an Admin-only `GET /admin/v1/network/access` route returning
  `AdminNetworkAccessDiagnostics`.
- Reuse the existing network readiness and redaction mapping used by
  `AdminServerConfigDiagnosticsResponse.network`; do not create a second
  interpretation of readiness.
- Add the route to the generated Admin route inventory with a stable key.
- Preserve Admin authorization behavior inherited from `admin::routes()`.
- Prove the response omits raw `external_base_url`, allowed origins, trusted
  proxy sources, forwarded header names, endpoint hosts, URL query strings,
  credentials, and tunnel token values.
- Keep the route out of Public Client OpenAPI/SDK/inventory surfaces.

## Acceptance Criteria

- `GET /admin/v1/network/access` returns exposure mode, readiness checks,
  external endpoint configured/scheme/fingerprint, trusted proxy counts, origin
  counts, and tunnel provider declarations.
- The route returns the same network diagnostics shape as the system config
  network subsection for the same config.
- Non-admin callers are rejected by the existing Admin route guard.
- Admin contract route parity includes the new route.
- Focused tests pass for route behavior, route inventory, API contract, format,
  and whitespace.

## Scope Boundaries

- No endpoint discovery route for Public Client in this slice.
- No DDNS, port mapping, NAT traversal, or tunnel process supervision.
- No wildcard CORS behavior and no new trusted proxy policy.
- No raw hostname, raw origin, trusted proxy source, URL, token, credential, or
  local listener disclosure in the dedicated route.
- No Admin Web UI work unless required by generated contract drift.

## Technical Notes

- Primary packages: `nako-server`, `nako-api`.
- Reuse `network_access_diagnostics` in `crates/nako-server/src/http/admin.rs`.
- Add a generated route suffix in `crates/nako-api/src/admin_contract.rs`.
- Add route tests near existing system/network diagnostics tests.
