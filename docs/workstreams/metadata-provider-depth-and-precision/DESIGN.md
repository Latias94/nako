# Metadata Provider Depth And Precision

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-provider-breadth` shipped provider capabilities, candidate matching,
and non-mutating candidate review. Later Generated Artifact lanes added
Provider Mapping application and repair safeguards. The provider layer now has
enough infrastructure to become more precise, but it still needs a tighter
contract for provider depth, subject identity, and what each provider can
safely match or fetch.

The main architecture risk is overclaiming provider support. For example,
diagnostics can say a provider supports season or episode fetch while the
adapter is actually using movie or generic subject endpoints. That makes
future Admin repair UX and automatic refresh behavior harder to trust.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Canonical Metadata, Local Inference, and Metadata Authority terms. |
| Merge and provider runtime ADRs | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md`; `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md` | Provider precision must not bypass field locks, raw-cache boundaries, shared runtime, or redaction. |
| Library pipeline map | Covered | `docs/architecture/LIBRARY_PIPELINE.md` | Routes provider depth after Generated Artifact apply closeout. |
| Provider breadth closeout | Covered | `docs/workstreams/metadata-provider-breadth/DESIGN.md`; `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md` | Confirms capabilities/matching exist and deeper provider identity was intentionally split. |
| Admin governance read model | Covered | `docs/workstreams/admin-catalog-governance-read-model/DESIGN.md` | Shows existing Admin needs for Provider Mapping counts and future details without raw evidence leakage. |
| Code seams | Covered | `crates/nako-core/src/media/candidate.rs`; `crates/nako-metadata/src/providers/tmdb.rs`; `crates/nako-metadata/src/provider_attempt.rs`; built-in provider adapters | First task should add TMDB season graph evidence without persistence side effects. |
| Generated Artifact apply | Out of scope | `docs/workstreams/generated-artifact-apply-repair-actions/CLOSEOUT.md` | Provider precision must not reopen apply repair or add another metadata executor. |

## Target State

When this lane closes:

1. Provider capabilities describe depth truth, not optimistic provider breadth.
2. Provider key semantics are explicit for movie, series, season, episode, and
   generic subject cases.
3. Matching/confirmation reasons are precise enough for Admin governance and
   future addon/import workflows.
4. TMDB, Douban, and Bangumi tests prove their supported media kinds, subject
   kinds, search/fetch support, hierarchy support, and credential requirements
   without leaking secrets.
5. Admin diagnostics stay redaction-safe and do not expose raw provider bodies.
6. Any durable candidate review, schema changes, or Web review mutation are
   split unless the first tasks prove they are necessary.

## First Slice Decision

Read-only code audit on 2026-06-02 found the sharpest safe first slice:
TMDB already advertises hierarchy support and can fetch season/episode details,
but provider candidate graphs currently use `MetadataCandidateGraph::for_provider`,
which creates only a root node with empty `related` and `relationships`.

The first implementation should add a TMDB series -> season graph preview:

- parse the season summaries already present in TMDB TV details;
- add related season Provider Subjects to the candidate graph;
- use existing TMDB season compound key semantics;
- preserve root-only refresh/Provider Mapping persistence;
- avoid creating season/episode Media Items or child Provider Mappings.

## In Scope

- Audit and tighten built-in provider capabilities.
- Define a provider-depth capability vocabulary if the current capability
  shape is too coarse.
- Make provider key and Provider Subject kind semantics explicit and tested.
- Improve non-mutating candidate review precision and reason vocabulary when
  it can be proven without schema changes.
- Update metadata provider diagnostics and generated Admin contracts only when
  the server/API shape actually changes.
- Update architecture and workstream evidence.

## Out Of Scope

- No new metadata provider.
- No Public Client API changes.
- No raw provider response exposure in Admin DTOs.
- No durable candidate review table in the first task.
- No automatic hierarchy repair.
- No Generated Artifact apply or recovery mutation changes.
- No Web Admin confirmation UI until backend/API semantics are clear.

## Architecture Direction

### Capability Truth Before UX

Provider depth belongs in `nako-metadata`, close to provider adapters and
tests. Server composition and Web should consume redaction-safe capability
facts; they should not infer provider depth from route names, config flags, or
provider-specific string parsing.

Capability truth remains important, but the first implementation does not need
a new diagnostics matrix. It should prove depth through concrete graph evidence
first, then split a capability-matrix follow-on only if TMDB, Douban, Bangumi,
or Admin diagnostics need more expressiveness.

### Provider Keys Are Provider-Owned, But Semantics Are Nako-Owned

Nako should not treat provider keys as opaque when the key implies a depth
boundary, such as a TMDB season or episode compound key. Parsing and rendering
must stay provider-adapter-owned, while the resulting Provider Subject kind and
capability facts remain Nako-domain concepts.

### Candidate Review Remains Non-Mutating

`MetadataCandidateMatchingPolicy` and candidate review already provide a
non-mutating ambiguity surface. This lane may strengthen decision reasons, but
must not persist candidate sets or change Canonical Metadata unless a later
task deliberately proves that durable review is required.

## First Executable Task

Start with `MPDP-020`: TMDB series -> season provider graph preview.

This task should answer:

- how TMDB hierarchy evidence is represented in `MetadataCandidateGraph`;
- how refresh and Provider Mapping persistence stay root-only;
- which follow-ons should own episode graph depth, Bangumi relations, Douban
  precision, durable candidate review, and Admin/Web confirmation.

## Follow-On Split

`MPDP-040` records the follow-on split in `FOLLOW_ONS.md`:

- `proposed:tmdb-season-episode-graph-depth`
- `proposed:bangumi-relations-and-episode-depth`
- `proposed:douban-subject-kind-precision`
- `proposed:metadata-candidate-durable-review`
- `proposed:admin-web-provider-depth-governance`

These proposed lanes should open separately. This lane should close after the
split is validated because its first safe vertical slice is already proven.
