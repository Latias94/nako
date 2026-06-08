# Live mutation fallback risk

## Question

Where can Admin Web display deterministic mock fallback read data while still
calling a live mutation method?

## Findings

- `GeneratedArtifactReviewPage` loads a review plan through
  `loadGeneratedArtifactReviewPlan`, but `reviewMutation` only checks whether
  `reviewGeneratedArtifact` exists.
- `CatalogGovernanceRepairPage` combines item detail and review-plan sources,
  but its review mutation only checks selection and method availability.
- `ItemArtworkGalleryPage` loads gallery data with fallback and then exposes
  select/unpublish controls if the mutation methods exist.
- `SourceDuplicateReconciliationPage` uses a feature adapter. The adapter can
  return fallback plans, and the page can still call `applySuggestion`.
- `AddonsPage`, Settings, Jobs, and Storage already provide useful positive
  examples: writes are blocked when the relevant source is not `live`.

## Risk

The data source does not fabricate mutation success by itself. The higher-risk
path is that a route renders fallback plan data and the operator confirms a real
mutation based on that fallback plan. That mixes mock authority with live writes.

## Desired Contract

- Read fallback remains useful for layout, diagnostics, and redaction testing.
- Mutation availability depends on both:
  - the mutation method existing, and
  - the governing read result being `source === "live"`.
- Page controls should communicate unavailability by being disabled; mutation
  functions should still reject if invoked programmatically.

## Test Targets

- App route tests for generated artifact review, catalog governance repair, and
  item artwork gallery.
- Feature adapter/page tests for source duplicate reconciliation.
- A small data-source helper test so future pages reuse the same source guard.
