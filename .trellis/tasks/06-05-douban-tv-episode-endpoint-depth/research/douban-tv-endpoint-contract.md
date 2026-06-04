# Douban TV Endpoint Contract

## Question

What is the smallest endpoint-backed Douban TV slice that can be implemented
without overclaiming Season/Episode support?

## Local Findings

- Current `DoubanMetadataProvider` uses:
  - `movie/search` for search;
  - `movie/subject/{id}` for fetch.
- Current capabilities advertise only `Movie` and `Unknown`, and explicitly
  reject `Series`, `Season`, and `Episode` before HTTP.
- Existing tests prove unsupported non-movie requests do not reach the mocked
  Douban runtime.
- TMDB supports true Series/Season/Episode endpoints and hierarchy preview.
- Bangumi supports subject-level Series fetch plus episode preview, but still
  rejects Season/Episode direct fetch.

## External Endpoint Reference

- `https://goddlts.github.io/douban-api-docs/movie.html`
  - The documented movie subject payload includes `subtype`, with sample values
    including `movie` and `tv`.
  - The documented TV subject payload includes TV-specific count fields such as
    seasons and episodes.
  - The simple search subject also carries `subtype`.

## Decision

Use the existing Douban `movie/search` and `movie/subject/{id}` endpoints for
subject-level TV support, but validate `subtype: "tv"` before mapping a result
to Nako `MediaKind::Series` / `ProviderSubjectKind::Series`.

Do not implement Season/Episode support in this task. The external reference
does not prove a Nako-ready episode endpoint contract, and the local
architecture explicitly requires endpoint-backed precision before capability
claims.

## Expected Test Shape

- Series search against mocked HTTP should hit `movie/search`, filter to TV
  subjects, and return a Series candidate.
- Series fetch against mocked HTTP should hit `movie/subject/{id}` and return a
  root-only Series graph.
- Season/Episode unsupported tests should remain, but the loop should exclude
  Series after this task.
- Built-in provider capability diagnostics should include Douban Series and
  still exclude Season/Episode/hierarchy support.
