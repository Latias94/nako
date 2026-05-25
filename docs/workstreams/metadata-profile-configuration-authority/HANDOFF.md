# Metadata Profile Configuration Authority - Handoff

Status: Completed
Last updated: 2026-05-25

## Current Focus

Lane closed on 2026-05-25.

The source-of-truth slice is implemented and verified.

## Current State

ALMPC shipped Admin API read/update routes for `MetadataProfile`. MPCA-020
closed the restart source-of-truth gap:

- config still defines configured Media Library identity/root/preset;
- `metadata.library_profiles.<library_id>` is an explicit TOML profile override;
- Admin updates now persist `LibraryOptions.metadata_profile_source = admin`;
- TOML overrides set `metadata_profile_source = configured`;
- preset-generated profiles use `metadata_profile_source = preset`;
- startup reconciliation preserves persisted Admin-owned profiles when desired
  config only has preset defaults.

## Verification

- `cargo nextest run -p nako-server metadata_profile_restart --no-fail-fast`
  passed.
- `cargo nextest run -p nako-server app_startup_overwrites_persisted_library_with_configured_desired_state --no-fail-fast`
  passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed.
- `cargo nextest run -p nako-db library_media --no-fail-fast` passed.
- `cargo nextest run -p nako-core metadata_profile --no-fail-fast` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

## Guardrails

- Do not add a database migration for the first slice.
- Do not expose internal source tracking through Public Client DTOs.
- Do not weaken configured library name/root/backend/preset reconciliation.
- Keep explicit TOML profile overrides authoritative.
- Treat dirty files outside this lane as user/other-session changes.

## Follow-Ons

1. Admin Web V2 should explain whether a profile is preset-derived,
   TOML-configured, or Admin-owned.
2. A future config export/import or writeback workflow can help operators move
   Admin changes into TOML-managed deployments.
3. Field-specific profile patch commands may be safer than full-profile
   replacement for form-based UI.
4. Addon scrape/writeback controls still need capability/grant/health-aware UX.
