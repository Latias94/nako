# AI Assisted Library Ops — TODO

Status: Complete
Last updated: 2026-05-22

Task IDs use the `AILO` prefix.

## M0 — Scope And Evidence Freeze

- [x] AILO-010 [owner=planner] [deps=network-access-boundary NAB-050] [scope=docs/workstreams/ai-assisted-library-ops,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open the AI Assisted Library Ops lane with Generated Artifact,
  acceptance, redaction, non-goal, and follow-on boundaries frozen.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, parent umbrella, and workstream index agree.
  Evidence: `docs/workstreams/ai-assisted-library-ops/DESIGN.md`.
  Handoff: Continue with AILO-020.

## M1 — Generated Artifact Proposal Queue

- [x] AILO-020 [owner=codex] [deps=AILO-010] [scope=crates/taru-core/src/automation.rs,crates/taru-db,crates/taru-automation,crates/taru-server/src/app/automation.rs]
  Goal: Deepen or wrap existing Automation Artifacts into a Generated Artifact
  proposal queue with stable target/provenance/confidence/readiness semantics
  for title-match, metadata-cleanup, summary, and recommendation artifacts.
  Validation: `cargo nextest run -p taru-db automation --no-fail-fast`;
  `cargo nextest run -p taru-automation --no-fail-fast`; focused server
  automation tests; `cargo fmt --all -- --check`; `git diff --check`.
  Review: `review-workstream` must check no canonical metadata, sidecar,
  Managed Import, Media Source, or library file mutation occurs.
  Evidence: `crates/taru-db/src/tests.rs`
  `taru_database_sqlite_lists_generated_artifact_proposals_with_readiness` and
  `taru_database_sqlite_marks_generated_artifact_proposal_stale_after_target_changes`;
  `crates/taru-automation/src/lib.rs`
  `automation_job_runner_persists_proposed_artifact_and_summary`;
  `crates/taru-server/src/app/tests/automation.rs`
  `automation_app_lists_generated_artifact_proposals_without_raw_payloads_or_mutation`.
  Handoff: Add Admin-only proposal diagnostics in AILO-030.

## M2 — Admin Proposal Diagnostics

- [x] AILO-030 [owner=codex] [deps=AILO-020] [scope=crates/taru-api/src/admin.rs,crates/taru-api/src/admin_contract.rs,crates/taru-server/src/http/admin.rs,apps/admin-web/src/adminApi]
  Goal: Expose Admin-only Generated Artifact proposal diagnostics and typed
  Admin web support without exposing prompts, raw generated payloads, provider
  secrets, raw Source Locators, local paths, or Public Client API shape.
  Validation: `cargo nextest run -p taru-api admin_contract --no-fail-fast`;
  `cargo nextest run -p taru-server http::tests::system --no-fail-fast`;
  `npm run check` from `apps/admin-web`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must check Admin boundary ownership and redaction.
  Evidence: Admin DTO/contract, route tests, Admin web contract sync, and Public
  Client protocol boundary check:
  `crates/taru-api/src/admin.rs`
  `admin_generated_artifact_proposals_expose_summaries_not_raw_prompt_or_payload`;
  `crates/taru-api/src/admin_contract.rs`
  `admin_contract_includes_read_model_route_constants`,
  `admin_contract_excludes_generated_fetch_runtime_and_raw_sensitive_fields`,
  `admin_web_generated_contract_matches_generator_output`, and
  `admin_contract_routes_stay_out_of_public_client_inventory`;
  `crates/taru-server/src/http/tests/system.rs`
  `admin_v1_generated_artifact_proposals_are_admin_only_redacted_and_read_only`;
  `apps/admin-web/src/adminApi` typed client/data source/mocks and generated
  contract.
  Handoff: Add acceptance planning in AILO-040.

## M3 — Acceptance Planning Without Autonomous Writes

- [x] AILO-040 [owner=codex] [deps=AILO-030] [scope=crates/taru-server/src/app,crates/taru-core,crates/taru-db,crates/taru-api/src/admin.rs]
  Goal: Add explicit accept/reject planning for at least title-match or
  metadata-cleanup proposals, routing accepted changes through existing
  metadata authority/NFO/apply boundaries and proving no autonomous writes.
  Validation: focused app/db tests for idempotent accept/reject and stale-target
  checks; relevant Admin/system tests; `cargo fmt --all -- --check`; `git diff
  --check`.
  Review: `review-workstream` must check acceptance audit, target revalidation,
  stale evidence, and no direct canonical mutation.
  Evidence: `crates/taru-server/src/app/tests/automation.rs`
  `automation_app_reviews_metadata_cleanup_proposal_without_canonical_mutation`
  and
  `automation_app_blocks_stale_generated_artifact_acceptance_and_allows_reject`;
  `crates/taru-api/src/admin.rs`
  `admin_generated_artifact_review_response_exposes_boundary_not_raw_payload`;
  `crates/taru-server/src/http/tests/system.rs`
  `admin_v1_generated_artifact_review_accepts_without_autonomous_metadata_writes`;
  Admin contract sync includes generated artifact review plan/review routes.
  Handoff: Close or split concrete provider adapters/local runtime in AILO-050.

## M4 — Closeout And Follow-On Split

- [x] AILO-050 [owner=planner] [deps=AILO-040] [scope=docs/workstreams/ai-assisted-library-ops,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split provider adapters, local model
  runtime, embeddings/vector search, Addon distribution, Public Client display,
  and protocol downloader follow-ons, then return the next lane decision to the
  post-RPD umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; workstream
  JSON and parent umbrella JSON validate with `python -m json.tool`; `git diff
  --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  post-AILO re-score notes. Provider-specific adapters, local model runtime,
  embeddings/vector search, Addon distribution, downloader protocol, Public
  Client display, and deeper metadata-authority apply work were split rather
  than hidden in this lane.
  Handoff: DONE. Return to `post-rpd-product-hardening`; Addon Runtime /
  Distribution is the next mainline lane.
