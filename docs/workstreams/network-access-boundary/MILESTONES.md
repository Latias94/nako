# Network Access Boundary — Milestones

Status: Complete
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Status: completed on 2026-05-22.

Exit criteria:

- [x] Workstream docs exist and agree.
- [x] Scope is endpoint/proxy/tunnel policy and readiness first.
- [x] Built-in NAT traversal runtime, identity/RBAC, downloader protocols, AI,
  and Addon runtime are out of scope.
- [x] Parent `post-rpd-product-hardening` points at this lane.

Primary evidence:

- `docs/workstreams/network-access-boundary/DESIGN.md`
- `docs/workstreams/network-access-boundary/TODO.md`

## M1 — Network Policy Domain And Config Validation

Status: completed on 2026-05-22.

Exit criteria:

- [x] Network access policy vocabulary exists for local-only, reverse-proxy,
  private-network, and tunnel-provider modes.
- [x] Unsafe public exposure combinations produce config-check errors or explicit
  warnings.
- [x] Auth-enabled-by-default assumptions are preserved.
- [x] Diagnostics redact tokens, credentials, internal secret-bearing URLs, and raw
  headers.

Primary evidence:

- server config/config-check tests
- deployment docs

## M2 — HTTP Boundary Enforcement

Status: completed on 2026-05-22.

Exit criteria:

- [x] Trusted forwarded headers are ignored unless configured.
- [x] External scheme/host handling follows policy and trusted proxy source
  policy, including exact-IP and CIDR matching.
- [x] CORS/origin behavior is explicit and default-safe, including allowed
  origin response headers and preflight handling.
- [x] Auth middleware remains authoritative for all non-health routes.

Primary evidence:

- HTTP system tests
- middleware/order review

## M3 — Admin Network Readiness Diagnostics

Status: completed on 2026-05-22.

Exit criteria:

- [x] Admin-only diagnostics report network readiness and blockers.
- [x] Admin TypeScript contract, typed Admin Web mapping, and mocks are
  synchronized.
- [x] Public Client API and `nako-client-protocol` remain unchanged.
- [x] Redaction tests cover tokens, tunnel credentials, raw headers, local paths,
  and secret-bearing URLs.

Primary evidence:

- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/http/admin.rs`
- `apps/admin-web/src/adminApi`
- HTTP/Admin tests

## M4 — Closeout And Follow-On Split

Status: completed on 2026-05-22.

Exit criteria:

- [x] Final gates pass with fresh evidence.
- [x] Workstream status and completed tasks are updated.
- [x] Parent post-RPD umbrella re-scores AI, Addon runtime, protocol downloaders,
  endpoint discovery, identity/RBAC, and concrete tunnel runtime follow-ons.
- [x] Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/network-access-boundary/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
