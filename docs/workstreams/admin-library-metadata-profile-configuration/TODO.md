# Admin Library Metadata Profile Configuration - TODO

Status: Completed
Last updated: 2026-05-25

Task IDs use the `ALMPC` prefix.

## M0 - Scope And Evidence Freeze

- [x] ALMPC-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-library-metadata-profile-configuration]
  Goal: Freeze the Admin API configuration problem, target state, non-goals,
  first vertical slice, and evidence gates.
  Validation: Workstream docs exist and agree.
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. New narrow lane opened for Admin Library Metadata
  Profile configuration.
  Handoff: Continue with ALMPC-020.

## M1 - Admin Profile Read/Update

- [x] ALMPC-020 [owner=codex] [deps=ALMPC-010] [scope=crates/nako-api,crates/nako-server/src/app/library.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests]
  Goal: Add Admin API read/update routes for a Media Library's
  `MetadataProfile`, persist updates through existing library options, and prove
  the next scan uses the updated scan policy.
  Validation: `cargo nextest run -p nako-server admin_library_metadata_profile --no-fail-fast`;
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
  Review: Do not add schema migrations. Do not bypass `LibraryRepository`.
  Preserve default Addon writeback false and keep Public Client API unchanged.
  Evidence: focused HTTP/app tests and generated Admin TS contract diff.
  Result: DONE 2026-05-25. Added Admin DTOs, generated Admin TypeScript
  contract entries, `GET`/`PUT /admin/v1/libraries/{library_id}/metadata-profile`,
  repository-backed profile replacement, and HTTP tests for persisted read/write
  plus next-scan behavior.
  Handoff: Continue with ALMPC-030 closeout/follow-on split. `cargo fmt --all
  -- --check` is blocked by pre-existing addon-event scheduler test formatting
  drift outside this lane; see `EVIDENCE_AND_GATES.md`.

## M2 - Follow-On Split Or Closeout

- [x] ALMPC-030 [owner=codex] [deps=ALMPC-020] [scope=docs/workstreams/admin-library-metadata-profile-configuration]
  Goal: Record verification evidence, close this API slice, and split Admin Web
  UI, config-file persistence, source ordering UX, or capability-aware Addon
  controls as follow-ons.
  Validation: `cargo fmt --all -- --check`; focused nextest gates;
  `git diff --check`.
  Review: No completion claim without fresh evidence and a clear handoff.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. Fresh focused API/server nextest gates, formatting,
  and whitespace checks pass. Lane closed with follow-ons split for
  restart-proof configuration authority, Admin Web V2/product design,
  field-specific patch commands, and capability-aware Addon controls.
