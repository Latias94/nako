# Admin Web V2 Media Browsing And Item Detail Governance - TODO

Status: Closed
Last updated: 2026-05-25

Task IDs use the `MBG` prefix.

## M0 - Scope And Evidence Freeze

- [x] MBG-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance,docs/workstreams/README.md]
  Goal: Open the lane, freeze route scope, non-goals, bridge policy, task order,
  validation gates, and first executable task.
  Validation: Workstream docs exist and agree with `CONTEXT.md`, `PRODUCT.md`,
  `DESIGN.md`, ADR 0027, and the closed library-management lane.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. Lane opened from
  `admin-web-v2-library-management-and-localization` closeout.
  Handoff: Route/API readiness audit completed in MBG-020.

## M1 - Route/API Readiness And Bridge Plan

- [x] MBG-020 [owner=codex] [deps=MBG-010] [scope=docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance,apps/admin-web/src/adminApi,docs/api/HTTP_API.md]
  Goal: Audit current public/admin item browse, detail, source, artwork,
  metadata, generated artifact, and playback-support routes; decide which reads
  can be bridged now and which gaps must split before implementation.
  Validation: `git diff --check`; readiness notes added to this workstream.
  Review: Public-read bridges must be explicitly named and must not imply Admin
  API ownership. Any mutation or unsafe field gap must be split.
  Evidence: updated `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and optional
  readiness note.
  Result: DONE 2026-05-25. `ROUTE_API_READINESS.md` accepts public-read bridges
  for `/items`, `/search`, `/items/{item_id}`, item credits/images, and bounded
  source probes; metadata diagnostics, per-item Generated Artifacts, Admin
  artwork decisions, provider mapping, Local Inference, NFO status, and repair
  actions remain split.
  Handoff: Continue with MBG-030 using the accepted `/catalog` bridge plan.

## M2 - Catalog Browse Route

- [x] MBG-030 [owner=codex] [deps=MBG-020] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/catalog,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add `/catalog` as a route-owned governance-oriented browse/search page
  with URL-owned filters, safe fallback, redaction checks, and navigation to
  item detail.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: The route must not become a watch-first poster wall or expose unsafe
  source/provider/artifact text.
  Evidence: route/data-source tests and browser smoke notes.
  Result: DONE 2026-05-25. `/catalog` now uses explicit public read bridges for
  `/items` and `/search`, URL-owned `q`/`facet`/`limit`/`offset`, safe route
  summaries, deterministic fallback, redaction tests, and stable links to the
  reserved `/items/:itemId` detail route.
  Handoff: Continue with MBG-040 to replace the reserved item route with the
  full governance item detail read model.

## M3 - Item Detail Route

- [x] MBG-040 [owner=codex] [deps=MBG-030] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/items,apps/admin-web/src/App.test.tsx,apps/admin-web/src/adminApi/client.test.ts,apps/admin-web/src/adminApi/dataSource.test.ts]
  Goal: Add `/items/:itemId` as an administration-supporting detail page with
  Media Item facts, Media Sources, Canonical Metadata summary, artwork and
  Generated Artifact readiness, NFO/provider/local-inference readiness, and
  support links.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`.
  Review: No playback controls, watch-state, raw Source Locators, local paths,
  raw provider bodies, artifact storage handles, or playback output paths.
  Evidence: item detail route tests, bridge tests, redaction tests, and browser
  smoke notes.
  Result: DONE 2026-05-25. `/items/:itemId` now renders a governance detail
  page backed by `getPublicItemDetailBridge`, bounded source probe summaries
  for at most three live item sources, safe route-local projections, readiness
  placeholders for split workflows, deterministic fallback, and redaction
  tests.
  Handoff: Continue with MBG-050 for repair/action split.

## M4 - Repair And Action Split

- [x] MBG-050 [owner=codex] [deps=MBG-040] [scope=docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance,docs/workstreams/README.md]
  Goal: Re-score item-scoped repair/action gaps after browse/detail lands and
  split Catalog repair, Generated Artifact review/apply, Artwork selection, NFO
  sidecar apply, and playback support detail into bounded follow-ons.
  Validation: `git diff --check`.
  Review: Follow-ons must be vertical workflows with confirmation, audit,
  dry-run/review, and redaction requirements when mutations are involved.
  Evidence: updated `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and
  `WORKSTREAM.json`.
  Result: DONE 2026-05-25. `FOLLOW_ON_SPLIT.md` re-scored item-scoped
  repair/action gaps and split Generated Artifact review/actions, item artwork
  selection, catalog repair/actions, safe metadata diagnostics, item NFO
  status/actions, and playback support detail into bounded follow-ons. The
  recommended next lane is `admin-web-v2-generated-artifact-review-actions`.
  Handoff: Continue with MBG-060 closeout before opening the next lane.

## M5 - Closeout

- [x] MBG-060 [owner=codex] [deps=MBG-050] [scope=docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance]
  Goal: Verify final gates, close the lane or explicitly split blockers, and
  update status fields.
  Validation: focused Admin Web gates plus browser smoke evidence; `git diff
  --check`.
  Review: `review-workstream` and `verify-rust-workstream` before completion
  claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, optional
  `CLOSEOUT.md`.
  Result: DONE 2026-05-25. Final Admin Web gates and browser smoke passed,
  closeout review found no blocking issues, remaining repair/action breadth is
  split, and this lane is closed.
  Handoff: Open `admin-web-v2-generated-artifact-review-actions` next.
