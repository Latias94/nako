# Admin Web V2 I18n Expansion - TODO

Status: Closed
Last updated: 2026-05-26

## M0 - Scope And Audit

- [x] I18N-010 [owner=codex] [deps=none] [scope=apps/admin-web/src,docs/workstreams/admin-web-v2-library-management-and-localization]
  Goal: Confirm the existing i18n boundary and identify the first route batch.
  Validation: `rg -n "i18n|locale|messages|t\\(" apps/admin-web/src docs/workstreams/admin-web-v2-library-management-and-localization docs/workstreams/README.md`
  Review: Do not translate API enum/query values or diagnostic facts.
  Evidence: `DESIGN.md`
  Handoff: DONE. Existing boundary covers shell and library-management copy.
  First route batch is `/overview` and `/access`.

## M1 - Default And Access Route Copy

- [x] I18N-020 [owner=codex] [deps=I18N-010] [scope=apps/admin-web/src/features/overview,apps/admin-web/src/features/access,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Move Overview and Users & Access visible UI copy into the English and
  Simplified Chinese catalogs.
  Validation: `cd apps/admin-web && npm run test -- App.test.tsx`; `cd apps/admin-web && npm run check`
  Review: Keep API values, ids, status strings, library names, provider names,
  and redaction-safe facts stable.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/overview` and `/access` now source route-visible copy from
  message ids, focused and broad Admin Web gates pass, and Chinese browser
  smoke confirms desktop/mobile rendering without horizontal overflow.

## M2 - Next Route Batch

- [x] I18N-030 [owner=codex] [deps=I18N-020] [scope=apps/admin-web/src/features/settings,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Select and migrate the next route batch based on current Admin Web V2
  management priority. This batch migrates `/settings`.
  Validation: route-local tests plus `cd apps/admin-web && npm run check`
  Review: Keep each batch small enough for route-local review.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/settings` now sources route-visible diagnostics and raw-cache
  mutation copy from message ids, focused/full Admin Web gates pass, and
  Chinese desktop/mobile browser smoke shows no horizontal overflow.

## M3 - Repair And Governance Route Batch

- [x] I18N-040 [owner=codex] [deps=I18N-030] [scope=apps/admin-web/src/features/catalog,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Migrate Catalog Governance list and repair routes in a bounded
  route-local batch.
  Validation: route-local tests plus `cd apps/admin-web && npm run check`
  Review: Confirmation controls, review-plan copy, and mutation-result copy
  must remain explicit; API enum values and ids stay untranslated.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/catalog/governance` and
  `/catalog/governance/:itemId` now source route-visible queue, filter,
  review-plan, repair-boundary, and confirmed-action copy from message ids.

## M4 - Remaining Action Route Batch

- [x] I18N-050 [owner=codex] [deps=I18N-040] [scope=apps/admin-web/src/features/items,apps/admin-web/src/features/automation,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Migrate item detail/artwork and Generated Artifact review routes in
  bounded route-local batches.
  Validation: route-local tests plus `cd apps/admin-web && npm run check`
  Review: Keep review-plan, confirmation, generated-artifact, and artwork
  mutation result copy explicit; API enum values, ids, provider keys, and
  artifact names stay untranslated.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/items/:itemId`, `/items/:itemId/artwork`, and
  `/automation/generated-artifacts/:artifactId/review` now source visible
  detail, artwork confirmation, Generated Artifact review-plan, boundary, and
  mutation-result copy from message ids.

## M5 - Residual I18n Audit

- [x] I18N-060 [owner=codex] [deps=I18N-050] [scope=apps/admin-web/src/features/jobs,apps/admin-web/src/features/playback,apps/admin-web/src/features/storage,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Audit remaining Admin Web V2 route-visible hard-coded copy and migrate
  the first residual table-route batch: Jobs, Playback Sessions, and Storage
  Staging.
  Validation: hard-coded UI-copy scan plus `cd apps/admin-web && npm run check`
  Review: Do not translate raw API values, ids, timestamps, file names,
  provider keys, status payloads, or diagnostic facts.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/jobs`, `/playback/sessions`, and `/storage/staging` now
  source route-visible table, filter, fallback, loading, empty-state, and
  panel copy from message ids.

- [x] I18N-070 [owner=codex] [deps=I18N-060] [scope=apps/admin-web/src/features/catalog,apps/admin-web/src/features/acquisition,apps/admin-web/src/features/automation,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Migrate the remaining queue/list route batch: Media Catalog,
  Acquisition Intake, and Generated Artifacts list.
  Validation: hard-coded UI-copy scan plus `cd apps/admin-web && npm run check`
  Review: Keep search query values, facet strings, candidate ids, source kinds,
  proposal ids, payload shapes, provider names, and status payloads stable.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/catalog`, `/acquisition/intake`, and
  `/automation/generated-artifacts` now source route-visible list, filter,
  table header, action-link, fallback, loading, empty-state, and panel copy
  from message ids.

- [x] I18N-080 [owner=codex] [deps=I18N-070] [scope=apps/admin-web/src/features/addons,apps/admin-web/src/i18n,apps/admin-web/src/App.test.tsx]
  Goal: Migrate the Addons route copy or split a dedicated Addons i18n
  follow-on if its breadth should not block this lane closeout.
  Validation: hard-coded UI-copy scan plus `cd apps/admin-web && npm run check`
  Review: Keep addon ids, versions, permission strings, hosted-page ids,
  protocol values, token prefixes, and health/status payloads stable.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `/addons` now sources route-visible title, filter, summary,
  table, panel, fallback, loading, empty-state, and install-boundary copy from
  message ids.
