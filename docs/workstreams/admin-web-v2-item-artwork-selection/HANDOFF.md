# Admin Web V2 Item Artwork Selection - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The lane is closed. AWA-010 through AWA-070 are complete:

- The lane was opened from GAR closeout and the MBG follow-on map.
- Scope is item-scoped Managed Artwork gallery, select/replace, and unpublish.
- Backend Admin routes and HTTP docs already exist for gallery/select/unpublish.
- AWA-020 accepted the backend item artwork gallery/select/unpublish routes for
  the first Admin Web slice and found no backend DTO hardening blocker.
- AWA-030 added generated Admin Web contract route constants and DTOs for item
  artwork gallery/select/unpublish and explicit `AdminApiClient` methods.
- AWA-040 added a read-only item-scoped Managed Artwork gallery route at
  `/items/:itemId/artwork`, linked it from Media Item detail, and added safe
  data-source projection with deterministic fallback and redaction tests.
- AWA-050 added explicit prepare/confirm select and unpublish controls to the
  item artwork gallery, wired live-only Admin Web mutation wrappers, rendered
  redaction-safe results, and covered visible mutation failures with no fake
  fallback success.
- AWA-060 ran full Admin Web check/test/build and browser smoke for item detail,
  artwork gallery, select confirmation, and unpublish confirmation. Desktop
  and mobile checks found no console errors, no document horizontal overflow,
  and no unsafe artwork source/storage/path/hash/token text.
- AWA-070 reviewed and closed the lane with no blocking findings. Fresh
  closeout gates passed: Admin Web check/test/build, focused `nako-api` admin
  contract tests, `cargo fmt --all --check`, and `git diff --check`.
- Candidate accept, ingest processing/requeue, artifact lifecycle cleanup,
  storage drift/remediation, provider search, uploads, catalog repair,
  Generated Artifact review, NFO writes, settings mutation,
  users/permissions/Library Access, and full-site i18n are out of scope.

## Closed Task

- Task ID: AWA-070
- Owner: planner
- Files:
  - `docs/workstreams/admin-web-v2-item-artwork-selection`
- Validation: final evidence review, review-workstream, verify-rust-workstream, `git diff --check`.
- Status: DONE 2026-05-25
- Review: no blocking findings
- Evidence: `CLOSEOUT.md`, closeout rows in `EVIDENCE_AND_GATES.md`, and
  AWA-060 browser smoke evidence.

## Decisions

- Start with item-scoped gallery/select/unpublish only.
- Mutations require explicit confirmation and visible error states.
- Gallery reads may use deterministic fallback, but select/unpublish mutations
  must not fake success.
- Admin Web must not render artwork source/storage/cache/path/hash/token data.
- Generated Admin Web contract output must come from Rust contract source.
- AWA-040 was gallery-only and intentionally did not post select/unpublish
  mutations.
- AWA-050 added select/unpublish UI only through an explicit prepare/confirm
  flow with visible failure states.
- AWA-050 now posts select/unpublish only after explicit confirmation and does
  not convert mutation failures into mock success.

## Blockers

- None. The lane is closed.

## Next Recommended Action

Open `admin-web-v2-catalog-repair-actions` next if continuing the media
governance sequence from MBG. Pull settings mutation or users/permissions/
Library Access forward only if product priority changes.
