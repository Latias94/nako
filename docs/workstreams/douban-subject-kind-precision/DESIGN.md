# Douban Subject Kind Precision

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-provider-depth-and-precision` split Douban subject-kind precision into
a focused follow-on. TMDB and Bangumi provider-depth lanes now prove a pattern:
provider capability facts must be endpoint-backed before graph depth, durable
review, or Admin/Web governance depends on them.

The current Douban adapter is movie-endpoint oriented:

- search uses `movie/search`;
- fetch uses `movie/subject/{id}`;
- capabilities previously included `Series`, `Season`, and `Episode`;
- graph output is root-only.

Nako should not claim non-movie subject kinds until a future endpoint-backed
lane proves the behavior.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Media Item, and Candidate Graph terms. |
| Provider depth split | Covered | `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md` | Selects this lane and its non-goals. |
| TMDB and Bangumi closeouts | Covered | `docs/workstreams/tmdb-season-episode-graph-depth/CLOSEOUT.md`; `docs/workstreams/bangumi-relations-and-episode-depth/CLOSEOUT.md` | Reuses endpoint-backed capability and preview-only persistence discipline. |
| Library pipeline map | Covered | `docs/architecture/LIBRARY_PIPELINE.md` | Routes provider precision through library-metadata-control-plane. |
| Douban adapter code | Covered | `crates/nako-metadata/src/providers/douban.rs`; `crates/nako-metadata/src/mapping/douban.rs` | Current adapter fetches movie subjects only and overclaims series/season/episode capability. |

## Target State

When this lane closes:

1. Douban provider capabilities describe only endpoint-backed behavior.
2. `Series`, `Season`, and `Episode` support is not advertised while the
   adapter uses only movie endpoints.
3. Unsupported search/fetch requests fail before provider HTTP calls.
4. Existing Douban movie refresh and Provider Mapping behavior stays
   compatible.
5. Durable candidate review and Admin/Web confirmation stay split follow-ons.

## In Scope

- Douban provider capability narrowing.
- Focused tests for Douban supported media/subject kinds.
- Unsupported-kind tests that prevent movie endpoints from masquerading as
  series/season/episode fetches.
- Workstream and architecture evidence updates.

## Out Of Scope

- Douban hierarchy graph preview.
- Generic TV/episode breadth without endpoint evidence.
- Schema migrations.
- Public Client API, Admin API, or Web changes.
- Generated Artifact apply changes.
- Raw Douban response, API key, header, or proxy URL exposure.

## Architecture Direction

### Capabilities Must Be Executable

Provider capabilities should describe the adapter's real endpoint contract. A
future Admin capability table should not need to know that Douban actually
routes every subject kind through movie endpoints.

### Narrow First, Extend Later

Douban can gain broader media kinds later, but only after a focused endpoint
lane proves the source contract and mapping semantics.

### Root Mapping Remains Compatible

Narrowing capability diagnostics must not break current movie refresh,
Provider Subject, or Provider Mapping behavior.

## First Executable Task

Start with `DSKP-020`: narrow Douban capability claims and tests to
endpoint-backed movie behavior.

This task should answer:

- which current Douban capability claims are executable today;
- which unsupported search/fetch requests should fail explicitly;
- which tests should guard future Douban breadth work from reintroducing
  overclaims.

## Closeout

Closed after `DSKP-030`. Douban capabilities now describe only endpoint-backed
movie behavior, unsupported Series/Season/Episode requests fail before HTTP,
and future Douban TV/episode support is split to
`proposed:douban-tv-episode-endpoint-depth`.
