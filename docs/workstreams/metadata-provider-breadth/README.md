# Metadata Provider Breadth

## Status

Completed execution lane.

This workstream is the first post-RPD product-hardening lane. It makes built-in
metadata matching across TMDB, Douban, and Bangumi capability-aware,
explainable, and safe before Taru adds broader NFO/link mutation, managed
import/download staging, AI suggestions, or addon metadata contributions.



## Shipped Boundary

This lane shipped four product boundaries:

- provider capabilities for TMDB, Douban, and Bangumi in registry diagnostics;
- deterministic candidate match decisions with explainable reasons;
- non-destructive ambiguous search refresh that stops before fetch/cache/commit;
- a first cross-provider candidate review API at
  `/items/{item_id}/metadata/candidates`.

Durable candidate queues and Admin UI confirmation are deliberately split as
follow-on work.

## Purpose

Existing provider refresh can call TMDB, Douban, and Bangumi and persist
accepted provider mappings. The remaining product risk is that provider
selection and conflicts are still too implicit for real operators:

- a provider can be registered but unsupported for a requested media kind;
- search-based matches do not explain why they are safe enough to accept;
- cross-provider candidates are not exposed as a conflict set before the first
  successful refresh writes canonical metadata;
- diagnostics describe runtime health but not provider capabilities.

This lane keeps the current refresh behavior compatible while adding the
capability and decision model needed for safer manual confirmation and future
import/AI/addon flows.

## Authoritative Docs

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Related Workstreams

- [post-rpd-product-hardening](../post-rpd-product-hardening/README.md)
- [metadata-catalog](../metadata-catalog/README.md)
- [metadata-provider-attempt-runtime](../metadata-provider-attempt-runtime/DESIGN.md)
- [metadata-refresh-seam](../metadata-refresh-seam/DESIGN.md)
- [metadata-merge-policy-unification](../metadata-merge-policy-unification/README.md)
