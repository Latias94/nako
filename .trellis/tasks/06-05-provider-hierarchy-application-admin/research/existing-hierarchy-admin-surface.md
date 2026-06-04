# Existing Hierarchy Admin Surface Research

## Existing Capability

- `nako-metadata::MetadataCandidateReviewApplicationService::apply_related_hierarchy`
  already applies safe related hierarchy from accepted Metadata Candidate
  Reviews.
- The operation requires the review to be accepted and the root Provider Mapping
  to be accepted before related hierarchy mutation.
- Existing metadata tests cover pending-review rejection, missing root mapping,
  safe child mapping confirmation, ambiguous target rejection, and unsafe
  relationship rejection.

## Current Gap

- Admin Candidate Review detail exposes the root Provider Mapping application
  plan, but that plan becomes `noop` when the accepted root mapping already
  exists.
- There is no related-hierarchy-specific read-only plan or Admin API route.
- Existing Admin apply route only commits the root Provider Subject and Provider
  Mapping; it keeps related Provider Subject, related Provider Mapping, and Media
  Item Hierarchy application out of scope.

## Required Boundary

- Add a related hierarchy plan before apply.
- Keep response fields redaction-safe and provider-neutral.
- Keep the route Admin-only and do not add Public Client API, Admin Web UI,
  durable batch execution, schema changes, or provider endpoint depth.

## Validation Targets

- `cargo nextest run -p nako-metadata related_hierarchy --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server related_hierarchy --no-fail-fast`
- `cargo check -p nako-core -p nako-metadata -p nako-api -p nako-server --tests`
