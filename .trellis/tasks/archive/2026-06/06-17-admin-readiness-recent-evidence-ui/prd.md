# Admin Readiness Recent Evidence UI

## Goal

Render the new redaction-safe `recent_evidence` block from
`GET /admin/v1/operator-readiness` inside the existing Admin Web operator
readiness drilldown. Product operators should see what happened recently for
the Media Library Scan intake components without opening raw durable job
payloads or storage/source details.

## Requirements

- Extend only the existing `/operator-readiness` Admin Web route.
- Do not add new Admin API calls, route params, mutations, or client-side job
  queries.
- Display `details.media_library_scan.recent_evidence` near the existing intake
  action plan so operators can compare "what to do" with "what happened".
- Preserve the backend component order and render all three components:
  `library_scan`, `source_fingerprint_hash`, and `watch_folder`.
- For latest job evidence, show only safe fields already exposed by the
  contract: kind, status, resource class, queued/start/completed timestamps,
  and `has_error`.
- For latest watch-folder tick evidence, show only safe booleans, enum values,
  and counters from the contract. Render `scan_job_present`, not a scan job id.
- If a component has no latest job or latest tick, render a deterministic
  "none" state.
- Render localized English and zh-Hans labels through the existing
  `operatorReadiness` catalog.
- Redact unsafe display values with the existing operator readiness safe display
  helpers. Do not render raw paths, locators, URLs with secrets, tokens,
  fingerprints, etags, durable `input_json`, durable `summary_json`, raw error
  strings, or job ids.
- Keep the page read-only. Do not turn recent evidence into an executable
  command surface.

## Acceptance Criteria

- [ ] `/operator-readiness` renders a "Recent intake evidence" section.
- [ ] The section shows three component entries with localized component names.
- [ ] Source fingerprint hash mock data renders queued job evidence, including
  safe resource class and `has_error = No`.
- [ ] Watch-folder mock data renders tick evidence, including
  `scan_job_present = Yes` and safe candidate/failure counts.
- [ ] Empty evidence facts render `None` rather than disappearing silently.
- [ ] Existing mock fallback and live data-source behavior stay unchanged.
- [ ] Route tests prove unsafe recent evidence fields are not rendered.
- [ ] `npm run check --prefix apps/admin-web` and focused Vitest route tests
  pass.

## Design Notes

- Visual thesis: extend the existing dense, operations-oriented drilldown
  surface with a second compact read-only evidence grid that matches the intake
  action plan styling.
- Content order: action plan first, recent evidence second, because operators
  should see the recommended inspection path before the supporting execution
  facts.
- Interaction plan: no new interaction beyond existing route refresh; hover and
  focus behavior inherit existing card/button styles.

## Out Of Scope

- Backend DTO or route changes.
- Generated Admin API contract changes.
- New job history pages or Admin Jobs filters.
- Executing scans, retries, watch-folder ticks, or repairs.
- Rendering raw durable job payloads, raw failures, or storage/source identity.

## Relevant Context

- Existing page: `apps/admin-web/src/features/overview/OperatorReadinessPage.tsx`
- Formatter helpers:
  `apps/admin-web/src/features/overview/operatorReadinessFormatters.ts`
- Localization: `apps/admin-web/src/i18n/catalogs/operatorReadiness.ts`
- Existing route tests: `apps/admin-web/src/App.test.tsx`
- Contract authority:
  `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- Backend recent evidence contract:
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
