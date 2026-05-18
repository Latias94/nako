# Admin Web Console TODO

Status: Active
Last updated: 2026-05-17

## AWC.0 Planning Baseline

- [x] AWC-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-console]
  Goal: Create the workstream, product design baseline, v0 context document,
  milestones, gates, and handoff notes.
  Validation: docs exist and agree on scope, non-goals, and route families.
  Evidence: this workstream.
  Handoff: Continue with AWC-020 before generating UI so v0 receives stable
  product context.

## AWC.1 Admin API Inventory

- [x] AWC-020 [owner=codex] [deps=AWC-010] [scope=docs/api, crates/taru-api, crates/taru-server]
  Goal: Inventory which existing HTTP routes can support the admin console and
  which Admin API routes or DTOs are missing.
  Validation: documented route matrix with current, missing, read-only, and
  mutation surfaces.
  Evidence: `ADMIN_API_MATRIX.md`.
  Handoff: Continue with AWC-030. Do not expand the Public Client API for
  admin-only diagnostics.

- [x] AWC-030 [owner=codex] [deps=AWC-020] [scope=docs/adr, docs/workstreams/admin-web-console]
  Goal: Decide whether the Admin API needs an ADR or a workstream design note
  for route namespace, versioning, DTO ownership, and leakage rules.
  Validation: accepted ADR or explicit note explaining why existing ADRs are
  sufficient.
  Evidence: ADR 0027 and `DESIGN.md`.
  Handoff: Admin-only routes should use `/admin/v1/*`; admin DTOs stay in
  `taru-api`; preserve separation from `taru-client-protocol` public client
  contracts unless a route is genuinely client-facing.

## AWC.2 Generated Prototype Preparation

- [x] AWC-035 [owner=codex] [deps=AWC-030] [scope=crates/taru-api, crates/taru-server, docs/workstreams/admin-web-console]
  Goal: Implement the first read-only Admin API v1 overview seam for the web
  console at `GET /admin/v1/overview`.
  Validation: `cargo fmt --all -- --check`, `cargo check -p taru-api --tests`,
  `cargo nextest run -p taru-api --no-fail-fast`, `cargo check -p taru-server
  --tests`, focused `taru-server` HTTP admin/system tests, public OpenAPI and
  TypeScript SDK leakage tests, `git diff --check`, and no
  `crates/taru-client-protocol` diff.
  Evidence: `taru_api::AdminOverviewResponse`, `crates/taru-server/src/http/admin.rs`,
  `http::tests::system::admin_v1_overview_composes_safe_read_only_diagnostics`,
  public OpenAPI/SDK leakage checks, and this workstream's evidence log.
  Handoff: M52 is complete. AWC-040/AWC-050 now refine the prototype context
  and capture the v0 prompt around the live overview seam.

- [x] AWC-040 [owner=planner] [deps=AWC-030] [scope=docs/workstreams/admin-web-console/V0_CONTEXT.md]
  Goal: Refine the v0 context with the confirmed Admin API inventory and first
  prototype scope.
  Validation: v0 context has page list, route families, mock-data guidance,
  brand direction, and safety rules.
  Evidence: updated `V0_CONTEXT.md` with the first prototype data-source split.
  Handoff: Keep framework choice open unless the user explicitly chooses one.

- [x] AWC-050 [owner=planner] [deps=AWC-040] [scope=external-v0-prompt]
  Goal: Produce a concise v0.dev prompt derived from `V0_CONTEXT.md` for the
  first admin console prototype.
  Validation: prompt is short enough to use directly and does not over-specify
  technology or component internals.
  Evidence: prompt captured in `HANDOFF.md`.
  Handoff: Generated UI should be treated as a prototype until API wiring,
  accessibility, and responsive checks pass.

## AWC.3 Implementation Follow-On

- [ ] AWC-060 [owner=codex] [deps=AWC-050] [scope=frontend-workspace-location]
  Goal: Choose and create the actual web app workspace location only after the
  prototype direction and front-end stack are accepted.
  Validation: app scaffold builds and has a clear API/mock-data boundary.
  Evidence: future implementation workstream or task notes.
  Handoff: Do not place generated code into the Rust server crates.

- [ ] AWC-070 [owner=codex] [deps=AWC-060] [scope=admin-api-sdk-or-client]
  Goal: Wire the first pages to real Admin API or a generated/admin-specific
  client layer.
  Validation: focused UI smoke tests or browser verification against local
  server/mock server.
  Evidence: future implementation evidence.
  Handoff: Secrets and local paths must stay redacted in UI and test fixtures.
