# Bangumi Relations And Episode Depth

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-provider-depth-and-precision` split Bangumi relation and episode
depth into a focused follow-on. TMDB now has safe preview graph depth through
series -> season and season -> episode, with refresh guards proving those
graphs are non-mutating. Bangumi needs the same discipline before Nako exposes
provider capability facts to Admin/Web governance or durable candidate review.

The current Bangumi adapter is subject-oriented:

- search uses `/v0/search/subjects`;
- fetch uses `/v0/subjects/{id}`;
- capabilities currently include `Season` and `Episode`;
- graph output is root-only.

Official Bangumi OpenAPI evidence shows separate relation and episode
endpoints exist. Nako should use those endpoints before claiming hierarchy or
episode depth.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Media Item, and Candidate Graph terms. |
| Provider depth split | Covered | `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md` | Selects this lane and its non-goals. |
| TMDB depth closeout | Covered | `docs/workstreams/tmdb-season-episode-graph-depth/CLOSEOUT.md` | Reuses preview-only, root-only persistence boundary. |
| Library pipeline map | Covered | `docs/architecture/LIBRARY_PIPELINE.md` | Routes Bangumi depth through library-metadata-control-plane. |
| Bangumi adapter code | Covered | `crates/nako-metadata/src/providers/bangumi.rs`; `crates/nako-metadata/src/mapping/bangumi.rs` | Current adapter fetches subjects only and overclaims season/episode capability. |
| Official Bangumi API | Covered | `https://raw.githubusercontent.com/bangumi/api/master/open-api/v0.yaml` | Endpoint evidence for subject relations and episodes. |

## Target State

When this lane closes:

1. Bangumi provider capabilities describe only endpoint-backed behavior.
2. `Season` and `Episode` support is not advertised until implemented through
   Bangumi relation or episode endpoints.
3. Bangumi relation or episode graph preview exists only for facts the adapter
   actually fetches.
4. Refresh persists only the root Bangumi Provider Subject and raw response.
5. Durable candidate review and Admin/Web confirmation stay split follow-ons.

## In Scope

- Bangumi provider capability narrowing.
- Focused tests for Bangumi supported media/subject kinds.
- Bangumi episode endpoint reconnaissance against existing adapter shape.
- Episode graph preview under a Bangumi series root when endpoint-backed.
- Refresh guard coverage for root-only Bangumi graph persistence.
- Workstream and architecture evidence updates.

## Out Of Scope

- Generic movie/TV breadth without Bangumi endpoint evidence.
- Schema migrations.
- Public Client API, Admin API, or Web changes.
- Generated Artifact apply changes.
- Automatic episode Media Item creation.
- Child Provider Subject or Provider Mapping writes from preview graph nodes.
- Raw Bangumi response, token, header, or proxy URL exposure.

## Architecture Direction

### Capabilities Must Be Executable

Provider capabilities should be a contract operators can trust. If a provider
cannot fetch an episode through the correct endpoint, it should not advertise
episode fetch support.

### Bangumi Subject Graph Is Anime-First

Bangumi relation and episode evidence should map into Provider Subjects and
Candidate Graph relationships without forcing TMDB's season hierarchy shape
onto Bangumi.

### Preview Does Not Mean Acceptance

Related episode nodes communicate evidence. They do not imply accepted Provider
Mappings, canonical hierarchy, or confirmed Media Items.

## First Executable Task

Start with `BRED-020`: narrow Bangumi capability claims and tests to
endpoint-backed subject behavior before adding relation/episode graph depth.

This task should answer:

- which current Bangumi capability claims are executable today;
- which unsupported search/fetch requests should fail explicitly;
- which tests should guard future episode endpoint work from reintroducing
  overclaims.
