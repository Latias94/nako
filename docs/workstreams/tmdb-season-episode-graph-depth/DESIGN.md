# TMDB Season Episode Graph Depth

Status: Closed
Last updated: 2026-06-02

## Why This Lane Exists

`metadata-provider-depth-and-precision` proved that TMDB series fetch can expose
season Provider Subjects in `MetadataCandidateGraph` while refresh persists
only the root Provider Subject. TMDB season fetch is the next narrow depth
slice because the existing adapter already supports season and episode fetches,
but season details do not yet project episode summaries into the candidate
graph.

This lane keeps provider depth evidence useful without crossing into hierarchy
creation, Admin review, or Provider Mapping acceptance.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Media Item, and Canonical Metadata terms. |
| Provider depth closeout | Covered | `docs/workstreams/metadata-provider-depth-and-precision/CLOSEOUT.md`; `FOLLOW_ONS.md` | Selects this lane and freezes root-only persistence as the baseline. |
| Library pipeline map | Covered | `docs/architecture/LIBRARY_PIPELINE.md` | Routes metadata provider graph depth through the library-metadata lane. |
| TMDB adapter code | Covered | `crates/nako-metadata/src/providers/tmdb.rs`; `crates/nako-metadata/src/mapping/tmdb.rs` | Season details parse root facts but not episode summaries yet. |
| Refresh persistence guard | Covered | `crates/nako-metadata/src/tests.rs` | Existing guard proves related nodes are not persisted as child Provider Mappings. |

## Target State

When this lane closes:

1. TMDB season fetch parses endpoint-backed episode summaries.
2. TMDB season fetch adds related episode Provider Subjects and `contains`
   relationships under the season root graph.
3. Episode graph nodes use existing TMDB episode compound key semantics:
   `{series_id}/{season_number}/{episode_number}`.
4. Refresh persists only the root season Provider Subject and raw response.
5. Follow-ons remain split for durable candidate review and Admin/Web
   confirmation.

## In Scope

- TMDB season details episode summary parsing.
- Episode graph preview nodes for `MetadataCandidateGraph`.
- Focused tests through `TmdbMetadataProvider::fetch(Season)`.
- Refresh guard coverage for root-only season persistence.
- Workstream and architecture evidence updates.

## Out Of Scope

- Automatic episode Media Item creation.
- Child Provider Subject or Provider Mapping writes from preview graph nodes.
- Schema migrations.
- Public Client API, Admin API, or Web changes.
- Generated Artifact apply changes.
- Bangumi, Douban, or generic candidate durability work.

## Architecture Direction

### Provider Adapter Owns Key Parsing

TMDB compound key parsing stays in the TMDB adapter. The Nako-domain graph only
sees Provider Subjects with precise subject kinds and subject keys.

### Graph Preview Does Not Mean Acceptance

Related episode nodes communicate evidence. They do not imply accepted Provider
Mappings, canonical hierarchy, or confirmed Media Items.

### Tests Prove Public Behavior

Tests should exercise `TmdbMetadataProvider::fetch(Season)` and
`MetadataRefreshService` rather than private helper functions.

## Closeout

Closed after `TSEG-040`.

This lane answered:

- how TMDB season details expose episode summary facts;
- how episode Provider Subjects are keyed;
- how existing season fetch behavior remains compatible;
- how season refresh remains root-only when episode preview nodes are present.
