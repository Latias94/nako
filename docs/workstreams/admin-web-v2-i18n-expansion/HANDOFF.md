# Admin Web V2 I18n Expansion - Handoff

Status: Closed
Last updated: 2026-05-26

## Current State

I18N-010 through I18N-080 are complete. The existing Admin Web i18n boundary is
small and dependency-free:

- `I18nProvider` owns locale state and `document.documentElement.lang`;
- `messages.ts` owns English and Simplified Chinese catalogs;
- `AdminShell` owns the locale selector;
- `SourceLabel` and Media Library management routes already consume message ids.
- `/overview` now sources route title, description, refresh action, fallback
  copy, loading label, metrics, static badges, table headers, and panel
  descriptions from message ids.
- `/access` now sources route title, description, refresh action, fallback
  copy, loading label, summary cards, readiness labels, panel copy, empty
  state, and mutation-readiness copy from message ids.
- `/settings` now sources route title, description, summary cards, diagnostic
  labels, raw-cache editor copy, confirmation controls, fallback copy, and
  provider-row labels from message ids.
- `/catalog/governance` now sources route title, queue copy, filters, table
  headers, issue/inference labels, review links, loading, empty state, and
  fallback copy from message ids.
- `/catalog/governance/:itemId` now sources route title, Media Item context,
  Provider Mapping selection, review-plan copy, repair boundaries, confirmed
  action copy, result labels, loading, empty state, and fallback copy from
  message ids.
- `/items/:itemId` now sources Media Item detail facts, Canonical Metadata
  panel copy, Media Source probe labels, artwork readiness labels, support
  links, fallback copy, loading, and empty state from message ids.
- `/items/:itemId/artwork` now sources Managed Artwork route copy,
  pagination labels, summary panels, Selected Artwork and candidate empty
  states, guarded select/unpublish confirmations, mutation-result labels,
  fallback copy, and loading state from message ids.
- `/automation/generated-artifacts/:artifactId/review` now sources Generated
  Artifact review route copy, review-plan fields, safe-summary labels, review
  boundaries, readiness labels, confirmation copy, mutation-result labels,
  fallback copy, loading, and empty state from message ids.
- `/jobs`, `/playback/sessions`, and `/storage/staging` now source table-route
  copy, filters, table headers, fallback text, loading/empty states, and panel
  copy from message ids.
- `/catalog`, `/acquisition/intake`, and `/automation/generated-artifacts`
  now source queue/list route copy, filters, table headers, action links,
  fallback text, loading/empty states, count labels, and panel copy from
  message ids.
- `/addons` now sources route-visible Addons copy, including title, filters,
  summary cards, table headers, panel copy, install-boundary text, fallback
  text, loading state, and empty state from message ids.

## Next Task

The lane is closed. Continue the broader Admin Web V2 goal with
users/permissions/Library Access, or open a separate follow-on if future
route-visible i18n work appears. LegacyDashboard hard-coded English remains a
legacy route outside V2 route-local closeout.

## Notes

- Do not translate API enum values, ids, provider names, Media Library names,
  job ids, route query values, timestamps, or status payloads that operators
  may need to match against logs.
- Keep live/mock/planned source labels routed through `SourceLabel`.
- Keep the route batch small enough to review without hiding functional changes.
- Browser smoke for I18N-020 used Playwright CLI because the in-app browser
  plugin's required Node REPL execution tool was not exposed in this session.
- Browser smoke for I18N-030 also used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized mock fallback layout.
- Browser smoke for I18N-040 used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized deterministic mock fallback
  layout for the Catalog Governance list and repair routes.
- Browser smoke for I18N-050 used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized deterministic mock fallback
  layout for Media Item detail, Managed Artwork, and Generated Artifact review
  routes.
- Browser smoke for I18N-060 used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized deterministic mock fallback
  layout for Jobs, Playback Sessions, and Storage Staging.
- Browser smoke for I18N-070 used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized deterministic mock fallback
  layout for Media Catalog, Acquisition Intake, and Generated Artifacts list.
- Browser smoke for I18N-080 used Playwright CLI. Local Vite had no live
  backend Admin API, so smoke verified localized deterministic mock fallback
  layout for Addons and observed the expected 404 on the mock health-check
  request.
