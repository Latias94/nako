# Metadata Provider Depth And Precision — Follow-Ons

Status: Proposed split
Last updated: 2026-06-02

`MPDP-020` and `MPDP-030` proved the first safe depth slice: TMDB series fetch
can expose season graph evidence while refresh and Provider Mapping persistence
stay root-only. Deeper provider work should now split by capability instead of
continuing this lane as a broad provider project.

## Split Principles

- Keep provider graph evidence in `nako-metadata` until a product workflow
  proves durable review is required.
- Do not create Media Items, child Provider Subjects, or child Provider
  Mappings from preview graph nodes without an explicit confirmation workflow.
- Keep schema, Public Client API, Admin API, and Web confirmation work in their
  own lanes.
- Tighten provider capability claims before UI depends on them.
- Treat raw provider payloads, paths, tokens, proxy URLs, and headers as
  forbidden diagnostics data unless a dedicated redaction contract owns them.

## Split Lanes

### docs/workstreams/tmdb-season-episode-graph-depth

Goal: Extend TMDB depth from series -> season preview to season -> episode
preview when TMDB season details expose episode summaries.

Status: closed as `docs/workstreams/tmdb-season-episode-graph-depth/`.

First slice:

- parse TMDB season `episodes` summaries;
- add episode Provider Subjects under the season graph when fetching a season;
- keep episode nodes as preview evidence only;
- prove refresh remains root-only for season fetches.

Non-goals:

- no automatic episode Media Item creation;
- no child Provider Mapping writes;
- no Web hierarchy confirmation UI.

### docs/workstreams/bangumi-relations-and-episode-depth

Goal: Make Bangumi depth claims anime-first and endpoint-backed.

Status: opened as `docs/workstreams/bangumi-relations-and-episode-depth/`.

First slice:

- audit Bangumi subject, relation, and episode endpoints in the existing
  adapter;
- tighten `MetadataProviderCapabilities` if current media/subject claims are
  too broad;
- add graph preview only for facts the adapter actually fetches.

Non-goals:

- no generic movie/TV breadth without endpoint evidence;
- no generated artifact apply changes;
- no raw Bangumi response exposure.

### docs/workstreams/douban-subject-kind-precision

Goal: Tighten Douban subject-kind precision around its current movie/generic
subject boundary.

Status: closed as `docs/workstreams/douban-subject-kind-precision/`.

First slice:

- verify which media kinds the current Douban endpoint contract truly supports;
- narrow diagnostics or provider capability notes if needed;
- add subject-key tests that prevent series/season/episode overclaiming.

Non-goals:

- no hierarchy graph preview until endpoint evidence exists;
- no Admin/Web review surface;
- no schema migration.

Follow-on split after closeout:

- `proposed:douban-tv-episode-endpoint-depth`

### docs/workstreams/metadata-candidate-durable-review

Goal: Persist candidate graph previews only when operator review becomes a
product requirement.

Status: closed as `docs/workstreams/metadata-candidate-durable-review/`.

Shipped:

- redaction-safe Candidate Graph review plan contract;
- durable review snapshot persistence for SQLite/PostgreSQL;
- backend-only idempotent accept/reject status transitions with stale guards;
- no Provider Mapping writes from review status transitions.

Non-goals:

- no provider adapter-specific persistence shortcuts;
- no reuse of Generated Artifact apply outcome tables as a generic candidate
  queue.

Follow-on split after closeout:

- `proposed:admin-web-provider-depth-governance`
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed)

### proposed:admin-web-provider-depth-governance

Goal: Show provider depth evidence in Admin/Web after backend review semantics
are stable.

First slice:

- expose redaction-safe provider graph summaries;
- distinguish preview evidence from accepted Provider Mappings;
- keep mutation behind explicit confirmation and fresh idempotency keys.

Non-goals:

- no raw provider payloads;
- no hidden refresh side effects;
- no Public Client API expansion before Admin semantics settle.
