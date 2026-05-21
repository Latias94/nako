# Admin Addon Operations MVP — TODO

Status: Active
Last updated: 2026-05-21

Task IDs use the `AAO` prefix.

## M0 — Contract And Goal Baseline

- [x] AAO-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-addon-operations-mvp,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md,docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md,docs/api/HTTP_API.md]
  Goal: Open the product Addon operations lane, freeze MVP route semantics,
  and decide unregister lifecycle policy before implementation.
  Validation: `git diff --check`.
  Review: Do not hide Addon Manager scope inside this lane. The unregister
  policy must explicitly choose terminal lifecycle state or physical deletion.
  Handoff: Continue with AAO-020 lifecycle mutation.
  Progress: Frozen MVP contract chooses terminal `unregistered` lifecycle
  state instead of physical deletion. Unregister preserves registration,
  tokens, side effects, and candidate audit history, revokes active tokens,
  clears accepted grants, and prevents all runtime Addon Token authentication.
  Reserved Admin routes: `PATCH /admin/v1/addons/{addon_id}/status`, `POST
  /admin/v1/addons/{addon_id}/unregister`, `POST
  /admin/v1/addons/{addon_id}/health-check`, `GET
  /admin/v1/addons/{addon_id}/surfaces`, and `POST
  /admin/v1/addons/{addon_id}/diagnostics/resource-call`. AAO explicitly does
  not mount `DELETE /admin/v1/addons/{addon_id}` and remains outside Addon
  Manager scope.
  Validation: `git diff --check`.

## M1 — Lifecycle Mutation

- [x] AAO-020 [owner=codex] [deps=AAO-010] [scope=crates/taru-core/src/repository/addon.rs,crates/taru-db,crates/taru-api/src/extension.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/app/addons/principal.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/api/HTTP_API.md]
  Goal: Add an Admin Addon lifecycle command for enable/disable without using
  full registration upsert as status mutation.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`; `git diff --check`.
  Review: Disabled Addons must not authenticate runtime Addon routes. Admin
  responses must not expose token hashes, raw tokens, or persistence-only
  fields.
  Handoff: Continue with AAO-030 unregister semantics.
  Progress: Added `PATCH /admin/v1/addons/{addon_id}/status` with
  `UpdateAddonStatusRequest`, explicit repository status mutation for SQLite
  and PostgreSQL, Admin-service enable validation against the stored manifest
  snapshot/granted scopes, and runtime principal resolution that rejects
  disabled registrations before marking Addon Tokens used. Admin responses use
  the existing redaction-safe registration detail envelope. The route only
  accepts `enabled` / `disabled`; `unregistered` remains reserved for AAO-030.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server
  --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo
  nextest run -p taru-db addon --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`.

- [x] AAO-030 [owner=codex] [deps=AAO-020] [scope=crates/taru-core/src/addon.rs,crates/taru-core/src/repository/addon.rs,crates/taru-db/migrations,crates/taru-db/migrations/postgres,crates/taru-db/src/sqlite/addons.rs,crates/taru-db/src/postgres.rs,crates/taru-api/src/extension.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/api/HTTP_API.md]
  Goal: Implement unregister/delete semantics with token revocation and
  redaction-safe Admin response.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`; focused DB Addon contract; focused Admin Addon nextest; PostgreSQL opt-in when `TARU_TEST_POSTGRES_URL` is available; `git diff --check`.
  Review: Prefer terminal lifecycle state preserving audit unless AAO-010
  documents physical deletion. Runtime Addon Token access must fail after
  unregister.
  Handoff: Continue with AAO-040 health checks.
  Progress: Added terminal `unregistered` Addon status, explicit
  `POST /admin/v1/addons/{addon_id}/unregister`, and backend-neutral
  `unregister_addon_registration` that transitions registration state, revokes
  active Addon Tokens, clears accepted grants, and preserves retained token /
  Side Effect / Addon Artwork Candidate history. Re-registration of a terminal
  manifest creates a new disabled registration ID through the normal register
  route; direct enable/token issue/token rotate/grant replace against the
  terminal registration is rejected. `DELETE /admin/v1/addons/{addon_id}` is
  not mounted.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server
  --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo
  nextest run -p taru-db addon --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`. PostgreSQL opt-in contract was not run because
  `TARU_TEST_POSTGRES_URL` was not available in this session.

## M2 — Health And Diagnostics

- [x] AAO-040 [owner=codex] [deps=AAO-010] [scope=crates/taru-addon-protocol,crates/taru-addon-client,crates/taru-api/src/extension.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/guides/ADDON_AUTHOR_GUIDE.md,docs/api/HTTP_API.md]
  Goal: Add an Admin Addon Health Check that proves reachability and manifest
  compatibility through a bounded, redaction-safe Addon Protocol contract.
  Validation: `cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast`; focused Admin Addon nextest; `git diff --check`.
  Review: Never pass administrator bearer tokens to an Addon Sidecar. Health
  reports must contain safe status, latency, protocol/manifest facts, and safe
  error codes only.
  Handoff: Continue with AAO-050 hosted surface read models.
  Progress: Added Addon Health Check request/response protocol envelopes,
  mockable `taru-addon-client::check_addon_health`, reference-addon `/health`,
  Admin DTOs, and `POST /admin/v1/addons/{addon_id}/health-check`.
  Health checks call `{base_url}/health` with protocol headers and a bounded
  timeout, never with admin bearer tokens, Addon Tokens, resolved Secret
  References, or resource payloads. Admin responses classify reachable,
  degraded/unhealthy, protocol mismatch, and unreachable cases with safe facts
  only.
  Validation: `cargo check -p taru-addon-protocol -p taru-addon-client -p
  taru-api -p taru-server --tests`; `cargo nextest run -p
  taru-addon-protocol -p taru-addon-client --no-fail-fast`; `cargo nextest run
  -p taru-server addons --no-fail-fast`; `cargo fmt --all -- --check`; `git
  diff --check`.

- [x] AAO-050 [owner=codex] [deps=AAO-040] [scope=crates/taru-api/src/extension.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md,docs/api/HTTP_API.md]
  Goal: Add Admin read models for Addon Entry Points, Hosted Pages,
  Configuration Schema metadata, Addon Task declarations, and Event
  Subscription declarations.
  Validation: `cargo check -p taru-api -p taru-server --tests`; focused Admin
  Addon nextest; `git diff --check`.
  Review: Hosted Pages are external Addon Sidecar pages, not trusted embedded
  Admin UI. Do not expose secrets or admin bearer-token launch URLs.
  Handoff: Continue with AAO-060 resource-call diagnostics.
  Progress: Added `GET /admin/v1/addons/{addon_id}/surfaces` and Admin DTOs
  for Entry Points, Hosted Pages, Configuration Schema, Secret Reference field
  declarations, Addon Tasks, and Addon Event Subscriptions. Hosted Page URLs
  are derived from stored manifest `base_url` plus declared absolute path and
  never include administrator bearer tokens, Addon Tokens, launch secrets, or
  resolved Secret Reference values.
  Validation: `cargo check -p taru-api -p taru-server --tests`; `cargo
  nextest run -p taru-server addons --no-fail-fast`; `cargo fmt --all --
  --check`; `git diff --check`.

- [ ] AAO-060 [owner=codex] [deps=AAO-040] [scope=crates/taru-addon-client,crates/taru-api/src/extension.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/api/HTTP_API.md]
  Goal: Add bounded resource-call diagnostics for declared Addon Resources so
  admins can distinguish unreachable sidecars, protocol mismatch, missing
  resource declarations, authorization gaps, and unsafe responses.
  Validation: `cargo check -p taru-addon-client -p taru-api -p taru-server --tests`; focused Admin Addon nextest; `git diff --check`.
  Review: Diagnostics must not echo raw payloads, Addon Tokens, admin tokens,
  Source Locators, storage paths, provider secrets, or raw response bodies.
  Handoff: Continue with AAO-070 closeout.

## M3 — Closeout

- [ ] AAO-070 [owner=planner] [deps=AAO-020,AAO-030,AAO-040,AAO-050,AAO-060] [scope=docs/workstreams/admin-addon-operations-mvp,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Verify and close the Admin Addon Operations MVP lane, or split any
  remaining independent tails into named follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`; focused Addon nextest gates; workspace nextest when practical; PostgreSQL opt-in contracts when available; `git diff --check`.
  Review: No vague Addon Manager bucket. Close only when an operator can manage
  Addon lifecycle and diagnose sidecar reachability safely.
