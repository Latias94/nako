# Metadata Profile Configuration Authority

Status: Completed
Last updated: 2026-05-25

This workstream defines the source-of-truth rules for Media Library Metadata
Profile configuration after Admin API updates are introduced.

Admin API can now persist a library's effective `MetadataProfile` into
repository-backed `LibraryOptions`. Startup reconciliation still rebuilds
configured libraries from TOML desired state, so Nako needs explicit semantics
for when TOML should overwrite a profile and when an Admin-edited profile should
survive restart.

Closed on 2026-05-25 after adding internal profile source tracking and proving
Admin-updated profiles survive restart unless TOML explicitly provides a
profile override.

## Goals

- Make Metadata Profile authority explicit for preset defaults, TOML overrides,
  and Admin API updates.
- Preserve Admin-updated profiles across restart when TOML does not explicitly
  own that library's profile.
- Keep explicit `metadata.library_profiles` TOML overrides authoritative and
  visible.
- Prove scan-time metadata acquisition uses the restart-preserved profile.

## Non-Goals

- Writing Admin updates back to TOML.
- Admin Web V2 page implementation.
- Profile field-level patch commands.
- Addon capability/grant-aware UI.
- Schema migrations for a separate profile table.

## Related Work

- `docs/workstreams/admin-library-metadata-profile-configuration`
- `docs/workstreams/library-metadata-scan-policy`
- `docs/workstreams/metadata-acquisition-pipeline`
- `docs/workstreams/multi-library-hardening`
- `docs/adr/0010-library-presets-are-configuration-templates.md`
