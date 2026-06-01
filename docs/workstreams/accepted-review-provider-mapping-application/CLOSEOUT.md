# Accepted Review Provider Mapping Application - Closeout

Status: Closed
Closed: 2026-06-02

## Shipped Scope

This lane turned accepted Metadata Candidate Reviews into an explicit backend
application boundary for root Provider Subject / Provider Mapping state.

Shipped behavior:

- `ARPMA-020` added redaction-safe read-only application plans with explicit
  action, reason, source conversion, and existing mapping status semantics.
- `ARPMA-030` added `MetadataCandidateReviewApplicationService`, which applies
  only the root Provider Subject and accepted Provider Mapping idempotently.
- Application keeps review status changes separate from Provider Mapping
  mutation, protects rejected mappings, rejects stale or wrong-item requests,
  and leaves related graph nodes as preview evidence.
- `ARPMA-040` split Admin API/Web exposure into a follow-on instead of growing
  this backend lane.
- `ARPMA-050` closes the lane with follow-ons for Admin/Web governance and
  related-node hierarchy application.

## Confirmed Boundaries

- No Admin API, Public Client API, generated contract, or Web route was added.
- No automatic metadata refresh side effect applies a durable review.
- No related Provider Subject, child Provider Mapping, or Media Item hierarchy
  mutation was added.
- Candidate Review application does not reuse Generated Artifact apply outcome
  tables or create a second Generated Artifact metadata executor.
- Raw provider payloads, paths, tokens, headers, proxy URLs, and provider
  bodies remain outside the review/application records.

## Surface Split Decision

Admin API/Web mutation scope is split to
`proposed:admin-web-provider-depth-governance`.

Rationale:

- existing Admin Catalog Governance Provider Mapping review routes operate on
  already persisted Provider Mapping rows;
- durable Metadata Candidate Reviews carry provider graph preview evidence,
  review status, application plan reasons, and accepted-review application
  semantics that need a distinct operator workflow;
- exposing Candidate Review mutation should require fresh idempotency keys,
  stale guards, redaction-safe graph summaries, and explicit separation between
  preview evidence and accepted Provider Mapping facts;
- combining those concerns into this backend lane would blur the root-only
  persistence boundary that ARPMA just proved.

## Validation

Fresh implementation gates from `ARPMA-030`:

- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 6 tests run, 6 passed.
- `cargo nextest run -p nako-metadata --no-fail-fast` passed: 51 tests run,
  51 passed.
- `cargo nextest run -p nako-db candidate_review provider_mapping --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo fmt --all -- --check` passed.

Fresh closeout gates:

- `python -m json.tool docs/workstreams/accepted-review-provider-mapping-application/WORKSTREAM.json`
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
- `git diff --check`

## Follow-Ons

- `proposed:admin-web-provider-depth-governance`: expose durable review
  evidence, decision state, application plans, and explicit apply mutations
  through Admin API/Web without leaking raw provider data or adding Public
  Client API.
- `proposed:provider-review-related-hierarchy-application`: apply related
  graph nodes, child Provider Subjects, child Provider Mappings, or Media Item
  hierarchy changes only after Admin/Web governance proves the operator model.
- `proposed:douban-tv-episode-endpoint-depth`: prove Douban TV/episode
  endpoint semantics before broadening Douban provider capabilities.

## Residual Risks

- There is no user-facing way yet to inspect or apply durable Metadata
  Candidate Reviews. The backend boundary is ready, but product exposure is
  intentionally split.
- Related graph evidence remains preview-only. This protects item identity, but
  it means accepted provider depth still cannot confirm seasons/episodes
  without a future hierarchy governance lane.
