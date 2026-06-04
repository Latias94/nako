# Douban Series Completion Evidence

## Question

What has already shipped for Douban TV/episode depth, and what must remain a
future follow-on?

## Local Evidence

- Archived Trellis task:
  `.trellis/tasks/archive/2026-06/06-05-douban-tv-episode-endpoint-depth/`.
- Commit evidence from recent history:
  `d2f1c367 feat(metadata): support douban series subjects`.
- The archived PRD says the shipped slice extends Douban from movie-only to
  endpoint-backed TV series subjects without overclaiming Season/Episode or
  hierarchy mutation support.
- Current tests include:
  `douban_provider_supports_series_subject_endpoint_without_hierarchy` and
  `douban_provider_rejects_season_episode_until_endpoint_backed`.
- Current provider diagnostics in `crates/nako-metadata/src/providers/douban.rs`
  describe movie and TV series subject-level metadata only.

## Interpretation

The old `proposed:douban-tv-episode-endpoint-depth` label is now too broad. The
Series subject-level portion has shipped. The remaining legitimate future work
is a narrower Douban Season/Episode graph-depth task that proves endpoint
semantics, relationship mapping, and hierarchy preview behavior before claiming
those capabilities.

## Planning Boundary

Docs should say:

- shipped: Douban Movie and TV Series subject-level search/fetch;
- not shipped: Douban Season/Episode direct search/fetch, child graph preview,
  hierarchy mutation, Admin/Web governance, and Public Client exposure.
