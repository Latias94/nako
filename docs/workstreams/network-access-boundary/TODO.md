# Network Access Boundary — TODO

Status: Complete
Last updated: 2026-05-22

Task IDs use the `NAB` prefix.

## M0 — Scope And Evidence Freeze

- [x] NAB-010 [owner=planner] [deps=PRPH-130] [scope=docs/workstreams/network-access-boundary,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open the Network Access Boundary lane with endpoint/proxy/tunnel policy
  scope, non-goals, first executable slice, gates, and parent routing.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, parent umbrella, and workstream index agree.
  Evidence: `docs/workstreams/network-access-boundary/DESIGN.md`.
  Handoff: Continue with NAB-020.

## M1 — Network Policy Domain And Config Validation

- [x] NAB-020 [owner=codex] [deps=NAB-010] [scope=crates/taru-server/src/config.rs,crates/taru-server/src/config_check.rs,docs/deployment]
  Goal: Add or refine a stable network access policy/config validation model
  for local-only, reverse-proxy, private-network, and tunnel-provider modes
  without starting a tunnel runtime.
  Validation: focused config-check tests; `cargo nextest run -p taru-server config --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: `review-workstream` must check safe defaults, auth dependency,
  secret redaction, and no Public Client API churn.
  Evidence: config structs, validation tests, deployment doc updates, and
  redacted config-check diagnostics.
  Handoff: DONE. Add request-time HTTP enforcement/readiness in NAB-030.

## M2 — HTTP Boundary Enforcement

- [x] NAB-030 [owner=codex] [deps=NAB-020] [scope=crates/taru-server/src/http,crates/taru-server/src/http/tests]
  Goal: Enforce trusted forwarded headers, external scheme/host handling, and
  CORS/origin policy through the HTTP boundary while keeping auth protection
  intact.
  Validation: focused HTTP system tests for trusted/untrusted forwarded headers,
  origin behavior, auth preservation, and redaction.
  Review: `review-workstream` must check middleware order, default-deny trust,
  and compatibility with health/readiness.
  Evidence: HTTP middleware/tests proving unsafe headers are ignored and only
  configured origins/proxies are trusted.
  Handoff: DONE. Add Admin readiness diagnostics in NAB-040.

## M3 — Admin Network Readiness Diagnostics

- [x] NAB-040 [owner=codex] [deps=NAB-030] [scope=crates/taru-api/src/admin.rs,crates/taru-api/src/admin_contract.rs,crates/taru-server/src/http/admin.rs,apps/admin-web/src/adminApi]
  Goal: Expose Admin-only network access readiness diagnostics and typed Admin
  web contract/client support without changing Public Client API or
  `taru-client-protocol`.
  Validation: `cargo nextest run -p taru-api admin_contract --no-fail-fast`;
  `cargo nextest run -p taru-server http::tests::system --no-fail-fast`;
  `npm run check` from `apps/admin-web`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must check Admin API ownership and redaction of
  bearer tokens, tunnel credentials, raw headers, internal URLs with secrets,
  and local paths.
  Evidence: DONE. Admin DTO/contract, `/admin/v1/system/config` Admin-only
  diagnostics, Admin web generated contract/data mapping, system route redaction
  tests, and Public Client protocol boundary check.
  Handoff: Continue with NAB-050 closeout/follow-on split for concrete tunnel
  providers, client endpoint discovery, identity/RBAC, protocol downloader
  integrations, AI, and Addon runtime.

## M4 — Closeout And Follow-On Split

- [x] NAB-050 [owner=planner] [deps=NAB-040] [scope=docs/workstreams/network-access-boundary,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split tunnel runtime, endpoint discovery,
  identity/RBAC, protocol downloader, AI, and Addon follow-ons, then return the
  next lane decision to the post-RPD umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; workstream
  JSON and parent umbrella JSON validate with `python -m json.tool`; `git diff
  --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  post-NAB re-score notes. Built-in NAT traversal runtime, client endpoint
  discovery, identity/RBAC, protocol downloader integrations, AI-assisted
  apply depth, and Addon runtime/distribution were split rather than hidden in
  this lane.
  Handoff: DONE. Return to `post-rpd-product-hardening`; AI Assisted Library
  Ops is the next mainline lane before Addon runtime/distribution.
