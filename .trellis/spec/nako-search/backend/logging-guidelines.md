# Logging Guidelines

`nako-search` currently does not need routine logging because evaluation is
pure. Add tracing only around future indexing or expensive evaluation paths.

## Required Patterns

- Prefer counters such as document count, candidate count, result count, and
  elapsed time.
- Log projection version mismatches as version diagnostics, not data dumps.
- Keep query text redacted or sampled carefully if user-entered terms can be
  sensitive.
- Keep facet labels bounded and normalized before logging.

## Forbidden Patterns

- Do not log full search documents.
- Do not log raw user search text by default.
- Do not emit one log line per candidate document in normal operation.
- Do not hide ranking determinism issues behind debug-only output.

## Useful Fields

- `search.document_count`
- `search.candidate_count`
- `search.result_count`
- `search.projection_version`
- `search.required_facet_count`
