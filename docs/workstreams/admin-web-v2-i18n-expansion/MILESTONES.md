# Admin Web V2 I18n Expansion - Milestones

Status: Closed
Last updated: 2026-05-26

## M0 - Boundary Confirmed

Exit criteria:

- Existing i18n provider, catalogs, locale selector, and shell integration are
  understood.
- The first route batch is selected from current Admin Web V2 priorities.
- Non-translatable API and diagnostic values are named explicitly.

## M1 - Overview And Access Localized

Exit criteria:

- `/overview` uses message ids for route copy, metric labels, static badges,
  table headers, loading labels, and fallback copy.
- `/access` uses message ids for route copy, cards, readiness labels, panel
  copy, loading labels, fallback copy, and mutation-readiness copy.
- English output remains stable for existing route tests.
- Simplified Chinese tests prove both routes render localized copy.

## M2 - Route Batch Expansion

Exit criteria:

- The next route batch is selected with bounded scope.
- Each migrated route has localized rendering tests.
- Browser smoke covers localized layout at desktop and mobile widths.

## M3 - Governance Routes Localized

Exit criteria:

- `/catalog/governance` uses message ids for route copy, queue copy, filters,
  table headers, review links, loading labels, empty state, and fallback copy.
- `/catalog/governance/:itemId` uses message ids for Media Item context,
  Provider Mapping selection, review-plan copy, repair boundaries,
  confirmation controls, mutation-result labels, loading labels, empty state,
  and fallback copy.
- English output remains stable for existing route tests.
- Simplified Chinese tests prove both Catalog Governance routes render
  localized copy.

## M4 - Remaining Action Routes

Exit criteria:

- Item detail/artwork routes migrate visible copy to message ids without
  translating API enum values, ids, provider keys, or artwork names.
- Generated Artifact review routes migrate visible copy to message ids without
  translating artifact ids, job ids, status payloads, or generated file names.
- Browser smoke covers localized layout at desktop and mobile widths.

## M5 - Residual Audit

Exit criteria:

- Remaining Admin Web V2 route-visible hard-coded copy is audited against the
  message catalog boundary.
- Any remaining high-value route copy is either migrated in a focused batch or
  recorded as an intentional raw diagnostic/API value.
- The lane has enough evidence to close or a named follow-on workstream exists.

## M6 - Residual Table Routes Localized

Exit criteria:

- `/jobs`, `/playback/sessions`, and `/storage/staging` use message ids for
  route copy, filters, table headers, fallback text, loading, empty state, and
  panel copy.
- English tests remain stable and Simplified Chinese tests prove localized
  rendering for all three routes.
- Browser smoke covers localized layout at desktop and mobile widths.

## M7 - Queue/List Routes Localized

Exit criteria:

- `/catalog`, `/acquisition/intake`, and `/automation/generated-artifacts` use
  message ids for route copy, filters, table headers, action links, fallback
  text, loading, empty state, count labels, and panel copy.
- English tests remain stable and Simplified Chinese tests prove localized
  rendering for all three routes.
- Browser smoke covers localized layout at desktop and mobile widths.

## M8 - Addons Route Localized

Exit criteria:

- `/addons` uses message ids for route copy, filters, summary cards, table
  headers, fallback text, loading, empty state, panel copy, and install-boundary
  text.
- English tests remain stable and Simplified Chinese tests prove localized
  rendering for the route.
- Browser smoke covers localized layout at desktop and mobile widths.
