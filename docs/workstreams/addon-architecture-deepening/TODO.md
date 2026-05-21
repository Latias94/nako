# Addon Architecture Deepening — TODO

Status: Completed
Last updated: 2026-05-21

Task IDs use the `AAD` prefix.

## M0 — Authority Freeze And Workstream Baseline

- [x] AAD-010 [owner=codex] [deps=none] [scope=docs/adr/0003-http-addons-before-in-process-plugins.md,docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md,docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md,docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md,docs/workstreams/addon-architecture-deepening,docs/workstreams/README.md,docs/GOALS.md]
  Goal: Freeze the accepted Addon architecture constraints, reconcile stale ADR
  statuses with shipped behavior, and publish this workstream as the
  authoritative lane for Addon architecture deepening.
  Validation: `git diff --check`.
  Review: Do not rewrite design history. Mark accepted constraints clearly and
  split still-deferred Addon Manager/Event Subscription/Task breadth from
  shipped Addon runtime behavior.
  Evidence: Updated ADR statuses or notes; workstream index entry; goal map
  entry.
  Progress: Marked ADR 0003, 0014, 0015, and 0020 Accepted with status notes
  that preserve deferred Addon Manager, OAuth, Native Plugin, and Jellyfin
  Plugin Compatibility non-goals. Added this workstream to the workstream index
  and made Addon Architecture Deepening the active goal in `docs/GOALS.md`.
  Validation: `git diff --check`.
  Handoff: Continue with AAD-020 after the authority baseline is clear.

## M1 — Addon Side Effect Runtime Depth

- [x] AAD-020 [owner=codex] [deps=AAD-010] [scope=crates/taru-server/src/app/addons/intake.rs,crates/taru-server/src/app/addons/runtime.rs,crates/taru-server/src/app/addons/side_effect_apply.rs,crates/taru-server/src/app/addons/metadata_write.rs,crates/taru-server/src/app/addons/artwork_write.rs,crates/taru-server/src/app/addons/library_file_write.rs,crates/taru-server/src/app/addons/target.rs,crates/taru-server/src/app/addons/principal.rs,crates/taru-core/src/addon.rs,crates/taru-server/src/http/tests/addons.rs]
  Goal: Deepen Addon Side Effect lifecycle into one runtime Interface that
  owns submit, authority, target validation, journaling, apply dispatch,
  apply outcome persistence, failure taxonomy, and replay behavior.
  Validation: `cargo check -p taru-core -p taru-server --tests`; `cargo
  nextest run -p taru-server addon_side_effect --no-fail-fast`; `cargo fmt
  --all -- --check`; `git diff --check`.
  Review: The new Module must reduce caller knowledge. Per-permission Adapters
  should return one apply result shape and should not decide journal lifecycle.
  Handoff: Continue with AAD-030 fingerprinted idempotency.
  Progress: Added `AddonSideEffectRuntime` as the submit/replay/authority/target
  validation/journaling lifecycle boundary. Kept `intake.rs` as a thin App
  Service adapter. Moved Addon principal resolution/authorization into reusable
  helpers so runtime and access-check share the same authority rules. Reworked
  per-permission side-effect Adapters to return `AddonSideEffectApplyCommand`,
  so the apply router/runtime owns outcome persistence, validation rejection,
  apply failure taxonomy, and metadata write commit execution. Preserved
  existing redaction and replay behavior ahead of AAD-030 fingerprinting.
  Validation: `cargo check -p taru-core -p taru-server --tests`; `cargo
  nextest run -p taru-server addon_side_effect --no-fail-fast`; `cargo fmt
  --all -- --check`; `git diff --check`.
  Handoff: Continue with AAD-030 to add deterministic request fingerprint
  semantics and conflict detection for same-key different-request reuse.

- [x] AAD-030 [owner=codex] [deps=AAD-020] [scope=crates/taru-core/src/addon.rs,crates/taru-db/migrations,crates/taru-db/migrations/postgres/0001_contract_jobs.sql,crates/taru-db/src/sqlite/addons.rs,crates/taru-db/src/sqlite/codec.rs,crates/taru-db/src/sqlite/migrations.rs,crates/taru-db/src/postgres.rs,crates/taru-db/src/contract_tests.rs,crates/taru-db/src/tests.rs,crates/taru-server/src/app/addons/runtime.rs,crates/taru-server/src/http/tests/addons.rs]
  Goal: Add request fingerprint semantics to Addon Side Effect idempotency so
  same Addon plus same idempotency key and same request replays, while same key
  with different permission/library/target/payload/provenance conflicts.
  Validation: `cargo check -p taru-core -p taru-db -p taru-server --tests`;
  `cargo nextest run -p taru-db addon --no-fail-fast`; `cargo nextest run -p
  taru-server addon_side_effect --no-fail-fast`; PostgreSQL opt-in contract
  when `TARU_TEST_POSTGRES_URL` is available; `git diff --check`.
  Review: The fingerprint must be deterministic, redaction-safe, and not expose
  raw payload/provenance in DTOs.
  Handoff: Continue with AAD-040 payload contract extraction.
  Progress: Added `AddonSideEffectRequestFingerprint` to `taru-core` and
  persisted request fingerprints in SQLite/PostgreSQL Addon Side Effect rows.
  Runtime replay now compares same Addon plus same idempotency key against the
  deterministic permission/library/target/provenance/payload fingerprint:
  matching requests replay, different requests return `409 conflict` without
  exposing raw payload, provenance, Source Locators, or tokens. AAD-090 later
  removed the temporary compatibility migration/fallback and folded the
  fingerprint into the clean base schemas because Taru has no deployed users.
  PostgreSQL schema parity was updated in
  `migrations/postgres/0001_contract_jobs.sql`; opt-in PostgreSQL contracts
  were skipped because `TARU_TEST_POSTGRES_URL` was not set.
  Validation: `cargo check -p taru-core -p taru-db -p taru-server --tests`;
  `cargo nextest run -p taru-db addon --no-fail-fast`; `cargo nextest run -p
  taru-server addon_side_effect --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`.
  Handoff: Continue with AAD-040 to extract shipped Protected Write payload
  contracts behind explicit Addon-facing Interfaces.

## M2 — Protected Write Payload Contracts

- [x] AAD-040 [owner=codex] [deps=AAD-030] [scope=crates/taru-addon-protocol,crates/taru-api/src/extension.rs,crates/taru-reference-addon,crates/taru-server/src/app/addons/metadata_write.rs,crates/taru-server/src/app/addons/artwork_write.rs,crates/taru-server/src/app/addons/library_file_write.rs,crates/taru-server/src/http/tests/addons.rs,docs/api/HTTP_API.md,docs/guides/ADDON_AUTHOR_GUIDE.md]
  Goal: Move shipped Protected Write payload shapes behind explicit Interfaces:
  Canonical Metadata patch, Addon Artwork Candidate proposal, and Library File
  Write command.
  Validation: `cargo check -p taru-addon-protocol -p taru-api -p
  taru-reference-addon -p taru-server --tests`; `cargo nextest run -p
  taru-addon-protocol --no-fail-fast`; `cargo nextest run -p taru-server
  addon_side_effect --no-fail-fast`; `git diff --check`.
  Review: Preserve the permissive protocol boundary. Do not move server-only
  records into `taru-addon-protocol`. The Interface should be Addon-author
  useful, not a mirror of persistence records.
  Handoff: Continue with AAD-050 Addon Manifest depth.
  Progress: Added Addon-facing payload DTOs to `taru-addon-protocol`:
  `AddonMetadataPatch`, `AddonArtworkWritePayload`, and
  `AddonLibraryFileWritePayload` plus supporting enums. Updated server
  metadata/artwork/library-file Adapters to parse those protocol contracts while
  keeping Taru-only validation, merge, candidate, storage, and report behavior
  inside `taru-server`. Added protocol wire-shape tests and reference Addon
  helper payload builders. Updated HTTP API and Addon Author Guide so Addon
  authors no longer need to infer shipped payloads from private server structs.
  Validation: `cargo check -p taru-addon-protocol -p taru-api -p
  taru-reference-addon -p taru-server --tests`; `cargo nextest run -p
  taru-addon-protocol --no-fail-fast`; `cargo nextest run -p taru-server
  addon_side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `git diff
  --check`.
  Handoff: Continue with AAD-050 Addon Manifest depth.

## M3 — Addon Manifest And Protocol Depth

- [x] AAD-050 [owner=codex] [deps=AAD-040] [scope=crates/taru-addon-protocol,crates/taru-reference-addon,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/guides/ADDON_AUTHOR_GUIDE.md,docs/api/HTTP_API.md,docs/workstreams/addon-architecture-deepening]
  Goal: Introduce a validated Addon Manifest Module and first-class declaration
  types for Addon Entry Points, Addon Hosted Pages, Addon Configuration Schema,
  Secret Reference fields, Addon Event Subscriptions, and Addon Task
  declarations where they can be modeled without runtime breadth.
  Validation: `cargo check -p taru-addon-protocol -p taru-reference-addon -p
  taru-server --tests`; `cargo nextest run -p taru-addon-protocol
  --no-fail-fast`; `cargo nextest run -p taru-server register_addon
  --no-fail-fast`; `git diff --check`.
  Review: Compatible additions should not require an Addon Protocol Version
  bump. Breaking contract changes must update the protocol version plan before
  implementation.
  Progress: Added compatible manifest declaration fields and protocol DTOs for
  Addon Entry Points, Addon Hosted Pages, Addon Configuration Schema, Secret
  Reference fields, Addon Event Subscriptions, and Addon Tasks. Deepened
  manifest validation for duplicate declaration IDs, absolute declaration
  paths, undeclared declaration scopes, entry points that reference missing
  hosted pages, non-object configuration schemas, and task timeout/retry
  bounds. Updated reference Addon manifest and Addon docs, and added server
  registration coverage for valid and invalid declaration contracts.
  Validation: `cargo check -p taru-addon-protocol -p taru-reference-addon -p
  taru-server --tests`; `cargo nextest run -p taru-addon-protocol
  --no-fail-fast`; `cargo nextest run -p taru-server register_addon
  --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Handoff: Continue with AAD-060 Library File Write runtime.

## M4 — Library File Write Runtime

- [x] AAD-060 [owner=codex] [deps=AAD-040] [scope=crates/taru-server/src/app/addons/library_file_write.rs,crates/taru-server/src/app/nfo.rs,crates/taru-nfo,crates/taru-vfs,crates/taru-core/src/addon.rs,crates/taru-server/src/http/tests/addons.rs,docs/workstreams/addon-library-file-write-policy,docs/api/HTTP_API.md]
  Goal: Deepen Library File Write into a Taru-owned runtime Module that owns
  target derivation, file role dispatch, storage writability, atomic
  replace/backup policy, idempotency context, and redacted reporting. Keep NFO
  Export as the first Adapter behind the seam.
  Validation: `cargo check -p taru-core -p taru-nfo -p taru-vfs -p
  taru-server --tests`; `cargo nextest run -p taru-server
  library_file_write --no-fail-fast`; focused NFO export nextest; `git diff
  --check`.
  Review: Do not add broad subtitle or arbitrary sidecar asset behavior unless
  the runtime seam is already proven. No raw Source Locator, storage URI, local
  path, backup URI, or payload content may leak into Addon responses.
  Progress: Replaced the NFO-only Adapter body with an
  `AddonLibraryFileWriteRuntime` seam plus normalized command, resolved target,
  and redacted outcome shapes. The runtime owns payload-to-command
  normalization, file-role dispatch, MediaSource target resolution, library
  lookup, file-write permit acquisition, writable-backend validation, NFO/VFS
  delegation, and safe apply-report creation. NFO Export remains the only
  Adapter behind the seam; no subtitle, arbitrary sidecar asset, or queued
  execution breadth was added. HTTP tests now assert safe target IDs,
  write-mode, and backup-policy facts in the report.
  Validation: `cargo check -p taru-core -p taru-nfo -p taru-vfs -p
  taru-server --tests`; `cargo nextest run -p taru-server
  library_file_write --no-fail-fast`; `cargo nextest run -p taru-nfo
  nfo_service_export --no-fail-fast`; `cargo fmt --all -- --check`; `git diff
  --check`.
  Handoff: Continue with AAD-070 Admin Addon API.

## M5 — Admin Addon API And DTO Shielding

- [x] AAD-070 [owner=codex] [deps=AAD-010,AAD-020] [scope=crates/taru-api/src/extension.rs,crates/taru-server/src/http/addons.rs,crates/taru-server/src/app/addons.rs,crates/taru-server/src/http/tests/addons.rs,docs/api/HTTP_API.md,docs/workstreams/admin-web-console]
  Goal: Finish the Admin Addon API seam by migrating
  `/admin/v1/addons` registration/list/detail behavior and shielding Admin DTOs
  from persistence records. Remove the old root `/addons` management surface
  instead of keeping a compatibility wrapper.
  Validation: `cargo check -p taru-api -p taru-server --tests`; `cargo
  nextest run -p taru-server addons --no-fail-fast`; `cargo nextest run -p
  taru-api --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Admin DTOs must not expose token hashes, raw token values,
  unredacted payload/provenance, raw storage paths, or persistence-only fields.
  Handoff: Continue with AAD-080 protocol/client crate boundary audit.
  Evidence: `/admin/v1/addons` is the only registration/list/detail HTTP
  surface; root `/addons` returns `404`; Admin responses use summary/detail
  DTOs and omit `manifest_json`.

## M6 — Protocol Crate Boundary And Storage Parity

- [x] AAD-080 [owner=codex] [deps=AAD-050] [scope=crates/taru-addon-protocol,crates/taru-addon-client,Cargo.toml,crates/taru-reference-addon,docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md,docs/workstreams/addon-architecture-deepening]
  Goal: Audit whether `taru-addon-protocol` should remain both wire contract
  and HTTP caller helper. The deletion test showed real Interface cost:
  protocol consumers inherited `reqwest`/`async-trait` transport dependencies
  even when they only needed wire DTOs. Split the HTTP caller helper into
  `taru-addon-client`.
  Validation: `cargo check -p taru-addon-protocol -p taru-addon-client -p
  taru-reference-addon -p taru-server --tests`; `cargo nextest run -p
  taru-addon-protocol -p taru-addon-client --no-fail-fast`; `cargo nextest run
  -p taru-server reference_addon_registers_queries_and_handles_resource_call
  --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: `taru-addon-protocol` now owns wire contracts and validation without
  `reqwest` or `async-trait`; `taru-addon-client` owns `AddonTransport`,
  `ReqwestAddonTransport`, `AddonClientError`, and `call_addon_resource`.
  Evidence: The protocol crate `Cargo.toml` has only `serde`/`serde_json`
  dependencies; HTTP caller tests moved to `taru-addon-client`.
  Handoff: Continue with AAD-090 Addon persistence parity.

- [x] AAD-090 [owner=codex] [deps=AAD-030,AAD-070] [scope=crates/taru-db/migrations,crates/taru-db/migrations/postgres,crates/taru-db/src/sqlite/addons.rs,crates/taru-db/src/postgres.rs,crates/taru-db/src/contract_tests.rs,docs/workstreams/postgresql-production-readiness]
  Goal: Verify and repair SQLite/PostgreSQL parity for Addon registrations,
  Addon Tokens, Library-Scoped Addon Grants, Addon Side Effects, fingerprinted
  idempotency, and apply outcomes.
  Validation: `cargo check -p taru-db --tests`; `cargo nextest run -p taru-db
  addon --no-fail-fast`; PostgreSQL opt-in `cargo nextest run -p taru-db
  addon --run-ignored ignored-only --no-fail-fast` when
  `TARU_TEST_POSTGRES_URL` is available; `git diff --check`.
  Review: Any schema change must have backend-neutral contract evidence. Taru
  has no deployed users, so prefer clean base-schema constraints over
  compatibility migrations or nullable historical fallbacks.
  Handoff: Continue with AAD-100 closeout.
  Progress: Reworked the Addon Side Effect fingerprint persistence from a
  compatibility-style incremental migration into the clean schema shape. SQLite
  migration `0022_addon_side_effects.sql` and PostgreSQL proof schema
  `migrations/postgres/0001_contract_jobs.sql` now define
  `request_fingerprint` as `NOT NULL` directly on `addon_side_effects`.
  Removed the temporary `0031` Addon fingerprint migration, removed nullable
  row-decoding fallback, and added SQLite schema evidence that
  `request_fingerprint` is non-null after migration. Repository inserts still
  derive the fingerprint once from permission/library/target/provenance/payload
  and idempotency remains unique by `(addon_id, idempotency_key)`.
  Validation: `cargo check -p taru-db --tests`; `cargo nextest run -p taru-db
  addon --no-fail-fast`; searched for obsolete `0031`/fallback/index patterns;
  `cargo fmt --all -- --check`; `git diff --check`. PostgreSQL opt-in
  contracts were skipped because `TARU_TEST_POSTGRES_URL` was not set.

## M7 — Closeout Or Split

- [x] AAD-100 [owner=planner] [deps=AAD-090] [scope=docs/workstreams/addon-architecture-deepening,docs/GOALS.md,docs/ROADMAP.md,docs/workstreams/README.md]
  Goal: Review, verify, and close the Addon architecture deepening lane, or
  split remaining independent tails into named workstreams.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace
  --tests`; focused Addon nextest gates; `cargo nextest run --workspace
  --no-fail-fast` when practical; PostgreSQL opt-in contracts when
  `TARU_TEST_POSTGRES_URL` is available; `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream discipline before
  declaring the lane complete. No vague follow-up buckets.
  Handoff: Mark this workstream complete only after evidence and gates are
  fresh.
  Progress: Reviewed the lane against its Design target state and found no
  blocking workstream-compliance or code-quality issues. No hidden tail needed
  a new follow-on workstream: Addon Manager, full Addon Task runtime, Event
  Subscription delivery, subtitle breadth, Native Plugin ABI, and Jellyfin
  Plugin Compatibility remain explicit non-goals. Closed the lane after fresh
  formatting, workspace check, focused Addon gates, full workspace nextest, and
  diff gates passed.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run -p taru-addon-protocol -p taru-addon-client
  --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`;
  `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo nextest run
  -p taru-server addon_side_effect --no-fail-fast`; `cargo nextest run -p
  taru-server library_file_write --no-fail-fast`; `cargo nextest run -p
  taru-api --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`;
  `git diff --check`. PostgreSQL opt-in contracts were skipped because
  `TARU_TEST_POSTGRES_URL` was not set.
