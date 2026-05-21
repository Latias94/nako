# Metadata Provider Breadth Design

Status: Completed
Last updated: 2026-05-21

## Why This Lane Exists

RPD recommends Metadata Provider Breadth as the next product lane. Taru already
has a strong foundation: **Provider Subject**, **Provider Mapping**, raw
provider response cache, provider attempts, shared HTTP runtime, field-lock
merge policy, and hierarchy confirmation. However, the current provider
workflow still behaves like "try providers in order until one succeeds".

That is not enough for a self-hosted operator with mixed movie, TV, anime, NFO,
and local inference data. Taru needs to explain what each provider can do,
which candidates it found, how safe each match is, and when conflicts should
wait for manual confirmation instead of silently becoming **Canonical
Metadata**.

## Relevant Authority

- ADRs:
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
- Existing workstreams:
  - `docs/workstreams/metadata-catalog`
  - `docs/workstreams/metadata-provider-attempt-runtime`
  - `docs/workstreams/metadata-refresh-seam`
  - `docs/workstreams/metadata-merge-policy-unification`
  - `docs/workstreams/post-rpd-product-hardening`

## Problem

The current provider surface lacks these product-grade boundaries:

- no explicit provider capability contract for supported **Media Kind**,
  subject kind, search support, fetch support, external-ID matching, hierarchy
  support, and credential requirement;
- provider diagnostics expose runtime health but not capability shape;
- search candidates have only a raw score and do not distinguish exact,
  strong, ambiguous, weak, or conflict decisions;
- strategy refresh accepts the best candidate per provider without a reusable
  matching policy;
- cross-provider search is not represented as a reviewable candidate set;
- there is no first-class "manual confirmation required" result for ambiguous
  provider conflicts.

## Target State

- `taru-metadata` exposes a provider capability model for built-in providers.
- TMDB, Douban, and Bangumi report capabilities without leaking secrets.
- Matching policy converts candidate scores and evidence into explicit
  decisions.
- Refresh keeps automatic external-ID matches safe and remains compatible.
- Search-based ambiguity can produce a non-destructive manual-confirmation
  signal rather than writing questionable canonical metadata.
- Server diagnostics include provider capabilities alongside runtime state.
- The first cross-provider conflict slice is documented and test-covered, with
  broader Admin UI confirmation split if needed.

## In Scope

- `taru-metadata` capability and matching policy types.
- Built-in provider capability implementations for TMDB, Douban, and Bangumi.
- Metadata provider diagnostics DTO/API additions for capability reporting.
- Focused tests for capabilities, match decisions, and non-destructive
  ambiguity handling.
- Workstream evidence and follow-on split notes.

## Out Of Scope

- Adding new providers beyond TMDB, Douban, and Bangumi.
- Provider-specific API feature breadth beyond capability reporting.
- Full Admin UI candidate confirmation workflow.
- New database schema unless the first implementation proves candidate sets
  must be durable immediately.
- NFO round-trip or link management changes.
- Managed import/download staging.
- AI suggestions or local model runtime.
- Addon metadata provider protocol changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Capability reporting can be in-memory and diagnostics-only first. | High | Provider configs and registry already exist in server composition. | If clients need durable capabilities, split a schema/API follow-on. |
| Search-based ambiguity can be represented before durable candidate queues. | Medium | Provider attempts already record skipped/failure outcomes, but not candidate sets. | If non-destructive review needs persistence, split a candidate-review persistence task before UI. |
| External-ID matches should remain auto-acceptable. | High | Existing refresh behavior and tests rely on direct provider keys from external IDs. | If provider IDs conflict, matching policy must mark conflicts before commit. |
| `taru-api` can carry admin diagnostics additions without touching public client protocol. | High | Metadata provider diagnostics already live in `taru-api::metadata_diagnostics`. | If API clients require versioning, update route docs and admin contract tests. |

## Architecture Direction

### Capability Model

Capabilities belong to the provider abstraction, not server config. A provider
can be available at runtime but still not support a requested **Media Kind** or
match path. The registry should be able to describe:

- provider identity and display name;
- supported media kinds;
- supported subject kinds;
- whether search is supported;
- whether fetch is supported;
- whether external-ID matching is supported;
- whether hierarchy fetch/confirmation is supported;
- credential requirement shape;
- notes/limitations.

This is diagnostics-safe because it contains no secret values, proxy URLs, raw
paths, or provider payloads.

### Matching Policy

Provider scores should be normalized into an explicit match decision:

- `accepted`: safe automatic refresh;
- `needs_confirmation`: plausible but ambiguous or conflicting;
- `rejected`: too weak or provider/kind mismatch.

The first slice should not invent a complex ML ranking model. It should encode
clear thresholds and reasons so future AI/addon/import workflows can consume a
stable decision vocabulary.

### Refresh Compatibility

Existing external-ID refresh behavior is preserved. Search-based refresh can
become stricter only when ambiguity is visible through attempts/tests and does
not silently write lower-confidence metadata.

### Cross-Provider Conflict

This lane should introduce a conflict vocabulary before building a full review
UI. The first durable output can be a service result or Admin/API diagnostic
that says "these provider candidates disagree and require confirmation" while
leaving **Canonical Metadata** untouched.

## First Implementation Slice

`MPB-020` should implement provider capabilities and expose them through
diagnostics. It is small, safe, and immediately useful:

- add capability types to `taru-metadata`;
- default `MetadataProvider::capabilities`;
- implement capabilities for TMDB, Douban, Bangumi;
- extend registry diagnostics;
- extend `taru-api` metadata provider diagnostics;
- test that `/metadata/providers` returns capabilities without secrets.



## Shipped Implementation

- `MetadataProviderCapabilities` now belongs to the provider abstraction and is
  reported by registry diagnostics for available providers.
- `MetadataCandidateMatchingPolicy` translates provider candidates into
  `accepted`, `needs_confirmation`, or `rejected` decisions with stable reason
  vocabulary.
- Search-based refresh uses the matching policy before provider fetch. Only
  accepted search candidates continue to fetch/cache/commit; ambiguous or weak
  candidates become explainable non-success attempts.
- Cross-provider candidate review is exposed through
  `/items/{item_id}/metadata/candidates`; it returns a review set and leaves
  **Canonical Metadata**, provider raw responses, and provider mappings
  untouched.

## Follow-On Split

Durable candidate persistence, manual accept/reject workflows, and Admin UI
confirmation are intentionally not part of this lane. They should be opened as
follow-ons after NFO/link authority and managed import staging decide how
candidate acceptance should interact with local file writes.

## Closeout Condition

This lane can close when:

- provider capabilities are explicit and exposed through diagnostics;
- matching policy decisions are test-covered;
- ambiguous search/provider conflicts do not silently mutate canonical state;
- refresh compatibility for safe external-ID matches remains covered;
- follow-on durable candidate review/UI work is split if not implemented;
- fresh gates in `EVIDENCE_AND_GATES.md` pass.
