# Admin Library Metadata Profile Configuration - Handoff

Status: Completed
Last updated: 2026-05-25

## Current Focus

Lane closed on 2026-05-25.

The Admin API slice is implemented, verified, and split from larger product
follow-ons.

## Current State

Closed prerequisite lanes already provide:

- `MetadataProfile::scan_acquisition_plan`;
- scan-time NFO Import;
- scan-triggered Addon Bulk Metadata Scrape;
- opt-in explicit Addon metadata writeback payloads;
- protected Addon `metadata_write` side-effect merge.

The missing product surface is an Admin API mutation path for changing a
library's effective profile after startup.

ALMPC-020 implemented that first product surface:

- `AdminUpdateLibraryMetadataProfileRequest`
- `AdminLibraryMetadataProfileResponse`
- `AdminMetadataScanAcquisitionPlan`
- `GET /admin/v1/libraries/{library_id}/metadata-profile`
- `PUT /admin/v1/libraries/{library_id}/metadata-profile`
- Admin TypeScript contract entries
- focused HTTP tests for persistence and next-scan behavior

## Guardrails

- Do not add schema migrations for this slice.
- Do not mutate metadata directly from Addon task output.
- Keep Addon writeback default false.
- Preserve existing Public Client API behavior.
- Treat existing dirty worktree changes outside this lane as user/other-session
  changes.

## Verification

- `cargo run -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed.
- `cargo nextest run -p nako-server admin_library_metadata_profile --no-fail-fast` passed.
- `git diff --check` passed.
- `cargo fmt --all -- --check` passed during closeout.

## Dirty Worktree Notes

The workspace already had unrelated addon-event scheduler/replay changes. To
make `nako-server` compile, `crates/nako-server/src/app/addons/event_runtime.rs`
was updated at the existing claim site with `forced_replay: false` and
`replay_reason_code: None`, matching the ordinary non-replay delivery path.

## Follow-Ons

1. Define config-file/writeback semantics for restart-proof profile updates.
   The current API persists to library options in the repository; a separately
   configured library may still need explicit source-of-truth rules across
   startup reconciliation.
2. Design Admin Web V2 controls for profile editing, likely as part of a wider
   React 19, Tailwind v4, shadcn/ui, and TanStack product design lane.
3. Add field-specific patch commands with stronger validation if full-profile
   replacement proves too blunt for UI workflows.
4. Make Addon scrape/writeback controls capability-aware once Addon health,
   grant, and capability diagnostics are ready.
