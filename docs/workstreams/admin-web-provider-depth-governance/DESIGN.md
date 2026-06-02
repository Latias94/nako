# Admin Web Provider Depth Governance - Design

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-candidate-durable-review` made provider Candidate Graph previews
durable. `accepted-review-provider-mapping-application` then proved backend
root Provider Subject / Provider Mapping application semantics.

The missing product boundary is Admin/Web governance: an operator must be able
to inspect durable review evidence, understand the accepted-review application
plan, and later apply that plan explicitly. This should not be hidden inside
metadata refresh, Generated Artifact apply, or existing Provider Mapping review
routes.

## Target State

When this lane closes:

1. Admin API exposes redaction-safe durable Candidate Review detail and
   accepted-review application plan facts.
2. Admin API can explicitly apply an accepted review with stale guards and an
   idempotency key.
3. Web Admin can show preview graph evidence separately from accepted Provider
   Mapping facts and require confirmation before mutation.
4. Generated Admin TypeScript contracts and Web data-source mapping stay in
   sync with Rust DTOs.
5. Related graph nodes remain preview evidence until a future hierarchy
   application lane owns them.

## In Scope

- Admin DTOs and route inventory for durable Candidate Review read/application
  plan.
- Admin mutation route for accepted-review root Provider Mapping application.
- Generated Admin TypeScript contract sync.
- Web Admin data-source and route-state support for inspect/confirm/apply.
- Redaction, idempotency, stale guard, and no-related-node application tests.

## Out Of Scope

- Public Client API.
- Related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- Raw provider payload retention or diagnostics exposure.
- Generated Artifact apply outcome reuse.
- Provider endpoint breadth such as Douban TV/episode support.

## Architecture Direction

### Existing Surface Is Precedent, Not The Surface

Admin Catalog Governance Provider Mapping review already accepts/rejects
persisted Provider Mapping rows. Durable Metadata Candidate Reviews are
different: they carry review snapshots, provider graph preview evidence,
application plan reasons, and a backend apply service. This lane should reuse
the operator-confirmation style, not collapse Candidate Review into the older
row-review route.

### Read Before Mutate

`AWPDG-020` starts with read-only Admin API detail and application plan routes.
They must prove redaction and no writes before `AWPDG-030` adds mutation.

### Confirmed Mutation

`AWPDG-030` should call `MetadataCandidateReviewApplicationService` through a
server-owned app boundary. It must carry item identity, expected review
freshness, and an idempotency key. Replays should be explicit and
diagnosable.

### Web Shows The Boundary

`AWPDG-040` should keep preview graph evidence visually separate from accepted
Provider Mapping facts. The user should see apply/noop/skip reasons before
confirming, and related nodes must not appear as silently applied hierarchy.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Metadata Candidate Review, Admin API, Public Client API, and Media Item terms. |
| Metadata ADRs | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md`; `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`; `docs/adr/0021-video-first-media-server-domain-model.md` | Keeps provider evidence separate from Canonical Metadata, secrets, and item identity. |
| Durable review closeout | Covered | `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md` | Review snapshots are durable and redaction-safe. |
| Accepted review application closeout | Covered | `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md` | Backend root-only application semantics are ready. |
| Generated Artifact Provider Mapping closeout | Covered | `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md` | Provides plan/apply/result precedent while warning against duplicate executors. |
| Admin Catalog Governance read model | Covered | `docs/workstreams/admin-catalog-governance-read-model/README.md` | Existing Admin governance pattern reviews persisted Provider Mapping rows only. |

## First Executable Task

`AWPDG-020` adds a read-only Admin API surface for durable Candidate Review
detail and application plan evidence. It must not apply Provider Mappings.

## Closeout Decision

The lane closes after `AWPDG-050`. Admin API/Web now expose the root-only
durable Candidate Review governance flow. Related-node hierarchy application,
Douban TV/episode endpoint depth, Candidate Review list/navigation, and broader
provider governance remain separate follow-ons.
