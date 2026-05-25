# Metadata Profile Configuration Authority - TODO

Status: Completed
Last updated: 2026-05-25

Task IDs use the `MPCA` prefix.

## M0 - Scope And Model

- [x] MPCA-010 [owner=codex] [deps=none] [scope=docs/workstreams/metadata-profile-configuration-authority]
  Goal: Open the workstream and freeze source-of-truth semantics for Metadata
  Profile configuration.
  Validation: Workstream docs exist and describe preset/configured/admin
  authority.
  Evidence: `README.md`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. New lane opened for restart-proof Metadata Profile
  configuration authority.
  Handoff: Continue with MPCA-020.

## M1 - Admin Profile Restart Persistence

- [x] MPCA-020 [owner=codex] [deps=MPCA-010] [scope=crates/nako-core/src/media/library.rs,crates/nako-server/src/config.rs,crates/nako-server/src/app/library.rs,crates/nako-server/src/app/library_reconciliation.rs,crates/nako-server/src/app/tests/startup.rs]
  Goal: Preserve Admin-updated Metadata Profiles across startup reconciliation
  when TOML does not explicitly provide `metadata.library_profiles` for the
  library, while keeping explicit TOML profile overrides authoritative.
  Validation: `cargo nextest run -p nako-server metadata_profile_restart --no-fail-fast`;
  `cargo nextest run -p nako-server app_startup_overwrites_persisted_library_with_configured_desired_state --no-fail-fast`.
  Review: Do not add database migrations or Public Client DTO fields. Do not
  weaken configured library name/root/preset reconciliation.
  Evidence: Startup tests and focused server nextest output.
  Result: DONE 2026-05-25. Added internal `MetadataProfileSource` tracking,
  marked Admin updates as `admin`, TOML profile overrides as `configured`, and
  preserved Admin-owned profiles during startup reconciliation when desired
  config only supplies preset defaults. New restart tests pass.
  Handoff: Continue with MPCA-030 closeout after verification.

## M2 - Evidence And Closeout

- [x] MPCA-030 [owner=codex] [deps=MPCA-020] [scope=docs/workstreams/metadata-profile-configuration-authority]
  Goal: Record verification evidence, close or split remaining UI/API follow-ons,
  and update handoff.
  Validation: focused nextest gates; `cargo fmt --all -- --check`;
  `git diff --check`.
  Review: No completion claim without fresh evidence.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: DONE 2026-05-25. Lane closed with follow-ons split for Admin Web V2
  source explanations, config export/import or writeback UX, and safer
  field-specific patch commands.
