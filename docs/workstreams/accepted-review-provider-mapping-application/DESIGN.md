# Accepted Review Provider Mapping Application - Design

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-candidate-durable-review` made provider Candidate Graph previews
durable and added accept/reject status transitions. It deliberately did not
turn an accepted review into Provider Mapping writes.

That separation is still correct. The next backend step is not an Admin/Web
screen; it is a named application boundary that says exactly when an accepted
review may create or promote Provider Subject / Provider Mapping state.

## Target State

When this lane closes:

1. Accepted Metadata Candidate Reviews can produce a read-only Provider Mapping
   application plan.
2. Application plans explain why a review is ready, skipped, or a noop before
   mutation.
3. Applying an accepted review upserts only the root Provider Subject and root
   accepted Provider Mapping idempotently.
4. Unsupported review sources such as `Automation` and `Other` do not silently
   become `MetadataSource::User`.
5. Related graph nodes remain preview evidence until a future hierarchy/Admin
   governance lane owns them.

## In Scope

- Application plan records and reason vocabulary for accepted reviews.
- Source conversion from `MetadataCandidateSource` to `MetadataSource` when
  safe.
- Root Provider Subject and root Provider Mapping application only.
- Idempotent service behavior over `MetadataCandidateReviewRepository` and
  `ProviderMappingRepository`.
- Focused metadata crate tests proving no writes during planning and no related
  graph node writes during application.

## Out Of Scope

- Admin API, Public Client API, generated client, or Web route work in the
  first task.
- Automatic application during metadata refresh.
- Applying related Provider Subjects or Media Item hierarchy changes.
- Reusing Generated Artifact apply outcome tables.
- Raw provider payload retention or diagnostics exposure.
- A second Generated Artifact metadata apply executor.

## Architecture Direction

### Plan Before Apply

`ARPMA-020` starts with a read-only plan. The plan should inspect an accepted
`MetadataCandidateReviewRecord`, its root Provider Subject, review source, and
existing Provider Mapping state.

The plan should be explicit about:

- review status not accepted;
- missing root Provider Subject;
- unsupported source conversion;
- existing accepted mapping;
- existing rejected mapping;
- ready-to-apply root mapping.

Planning must not upsert Provider Subjects or Provider Mappings.

### Root-Only Application

`ARPMA-030` may apply only the root Provider Subject and Provider Mapping from
an accepted review. Related review nodes and relationships remain evidence for
Admin/Web governance and future hierarchy repair.

Application should:

- require an accepted review;
- keep stale guards from candidate review decisions;
- upsert or reuse the root Provider Subject by provider/subject kind/key;
- upsert or promote the root Provider Mapping to `Accepted`;
- keep rejected mappings protected unless a later explicit override task
  changes that policy;
- return a summary that reports changed/noop/skipped behavior.

### Reuse Existing Semantics

Generated Artifact Provider Mapping apply already proved host-owned Provider
Subject / Provider Mapping application and idempotent replay. This lane should
reuse the same repository semantics and avoid a second hidden apply executor.
If implementation exposes a small shared helper, it belongs in
`nako-metadata`, not in server HTTP code.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Metadata Candidate Review, Canonical Metadata, Admin API, and Public Client API terms. |
| Metadata merge ADR | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md` | Keeps provider evidence separate from canonical metadata merge. |
| Provider diagnostics ADR | Covered | `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md` | Keeps raw provider payloads and secrets out of diagnostics/application records. |
| Video-first domain ADR | Covered | `docs/adr/0021-video-first-media-server-domain-model.md` | Provider-specific concepts map through Provider Mapping, not item identity. |
| Durable candidate review closeout | Covered | `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md` | Requires accepted-review Provider Mapping application to be explicit follow-on scope. |
| Generated Artifact Provider Mapping closeout | Covered | `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md` | Proves idempotent host-owned Provider Mapping apply semantics and warns against duplicate executors. |

## Executable Sequence

`ARPMA-020` defines the read-only application plan contract and tests before any
mutation service is added. `ARPMA-030` adds the root-only application service.
`ARPMA-040` split Admin API/Web mutation scope to
`docs/workstreams/admin-web-provider-depth-governance/` before any surface is
exposed.
