# Admin Web V2 Library Management And Localization - TODO

Status: Closed
Last updated: 2026-05-25

Task IDs use the `AWL` prefix.

## M0 - Scope And Evidence Freeze

- [x] AWL-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-library-management-and-localization,docs/workstreams/README.md]
  Goal: Open the lane, freeze scope, target state, non-goals, task order, and
  validation gates for library management plus Admin Web localization
  foundation.
  Validation: Workstream docs exist and agree with `CONTEXT.md`, `PRODUCT.md`,
  `DESIGN.md`, Admin Web V2 closeouts, and the Admin API matrix.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. Lane opened from the completed read-only
  `/libraries` closeout and current Jellyfin/Plex parity audit.
  Handoff: Continue with AWL-020.

## M1 - Library Detail Read Model And Route

- [x] AWL-020 [owner=codex] [deps=AWL-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/libraries,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi]
  Goal: Add `/libraries/:libraryId` as the first route-owned Media Library
  management detail entry. It should show safe library facts, a source/inventory
  placeholder or live bridge, metadata-profile summary, scan/NFO readiness, and
  navigation back to `/libraries`.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`.
  Review: Verify route ownership, safe fallback behavior, no unsafe local path,
  source locator, secret, token, or raw provider text rendering.
  Evidence: route/data-source tests and browser smoke notes in
  `EVIDENCE_AND_GATES.md`.
  Result: DONE 2026-05-25. `/libraries/:libraryId` is route-owned, reachable
  from `/libraries`, backed by system config plus metadata-profile data-source
  composition, and renders source/jobs/failure readiness as safe placeholders
  without page-load mutations.
  Handoff: Public source inventory bridge decision resolved by AWL-030.

## M2 - Metadata Profile And Library Actions

- [x] AWL-030 [owner=codex] [deps=AWL-020] [scope=apps/admin-web/src/adminApi,apps/admin-web/src/features/libraries,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts,crates/nako-api/src/admin_contract.rs,crates/nako-server/src/http/admin.rs,crates/nako-server/src/http/tests/library.rs]
  Goal: Surface existing metadata profile GET/PUT semantics and design or wire
  first scan/NFO actions with confirmation, loading, success, failure, and
  fallback states.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: Full-profile replacement must be clear to operators; scan/NFO actions
  must not fire on page load and must not hide unsafe backend responses.
  Evidence: tests proving metadata profile route use, command boundaries, and
  unsafe text exclusions.
  Result: DONE 2026-05-25. Library detail now exposes Metadata Profile as an
  explicit full-replacement GET/PUT workflow, summarizes Source inventory
  through a public-read bridge, and wires scan/NFO import/export as confirmed
  Admin API command routes that enqueue redaction-safe job summaries.
  Handoff: Split field-specific profile patching or runtime library config
  authority if full replacement is too blunt; parity gap split completed in
  AWL-050.

## M3 - Admin Web Localization Foundation

- [x] AWL-040 [owner=codex] [deps=AWL-020] [scope=apps/admin-web/src/i18n,apps/admin-web/src/components/layout,apps/admin-web/src/features/libraries,apps/admin-web/src/App.test.tsx]
  Goal: Add a small Admin Web localization boundary with English and Simplified
  Chinese catalogs, locale selection defaults, formatting helpers, and migration
  of the app shell plus library management visible copy.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx`.
  Review: Avoid translating domain identifiers, API enum values, safe
  fingerprints, IDs, or source statuses that operators must compare with
  backend diagnostics.
  Evidence: tests proving both locales render expected route labels without
  changing API query values.
  Result: DONE 2026-05-25. Added a dependency-free `i18n` provider, English and
  Simplified Chinese catalogs, shell locale selector, localized SourceLabel
  text, and message-id migration for the app shell plus library list/detail
  management surfaces.
  Handoff: Split broader route translation after the pattern is accepted.

## M4 - Parity Gap Split

- [x] AWL-050 [owner=codex] [deps=AWL-030,AWL-040] [scope=docs/workstreams/admin-web-v2-library-management-and-localization,docs/workstreams/README.md]
  Goal: Re-score Jellyfin/Plex-style Admin Web parity after library management
  and localization foundation, then split media browsing/detail, users/library
  access, settings mutation, playback runtime controls, artwork management, and
  catalog repair into dedicated follow-ons.
  Validation: `git diff --check`; workstream docs updated and internally
  consistent.
  Review: Ensure follow-ons are vertical workflows, not vague buckets.
  Evidence: updated `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and
  `WORKSTREAM.json`.
  Result: DONE 2026-05-25. Added `PARITY_GAP_SPLIT.md` with current Admin Web
  V2 baseline, gap scoring, and dedicated follow-on lane candidates. The
  recommended next execution lane is
  `admin-web-v2-media-browsing-and-item-detail-governance`.
  Handoff: Closeout completed in AWL-060; open the bounded media browsing/item
  detail governance lane next.

## M5 - Closeout

- [x] AWL-060 [owner=codex] [deps=AWL-050] [scope=docs/workstreams/admin-web-v2-library-management-and-localization]
  Goal: Verify final gates, close the lane or explicitly split remaining work,
  and update status fields.
  Validation: focused Admin Web gates plus browser smoke evidence; `git diff
  --check`.
  Review: `review-workstream` and `verify-rust-workstream` before completion
  claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`,
  optional `CLOSEOUT.md`.
  Result: DONE 2026-05-25. Final Admin Web, Rust, browser, review, and
  whitespace gates are recorded; `CLOSEOUT.md` closes this lane and hands off
  to `admin-web-v2-media-browsing-and-item-detail-governance`.
  Handoff: Open the recommended follow-on lane before implementing media
  browsing/item detail work.
