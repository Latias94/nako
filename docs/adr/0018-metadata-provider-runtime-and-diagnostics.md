# 0018: Use a Shared Metadata Provider Runtime and Diagnostics Boundary

## Status

Accepted.

## Context

Nako metadata refresh started with TMDB and later grew provider ordering,
field-lock-aware merge policy, raw response caching, and fallback attempts for
Bangumi and Douban. That direction is useful, but provider code becomes fragile
if each implementation owns its own HTTP client, timeout behavior, retry policy,
secret handling, and failure vocabulary.

The server also needs an operator-facing way to answer why metadata did or did
not refresh for an item. Job summaries alone are not enough: they are tied to a
single job, do not expose raw cache entries, and cannot describe provider
availability when a provider is disabled, missing credentials, or rate limited.

## Decision

All network metadata providers must use the shared `MetadataHttpRuntime`. The
runtime owns HTTP client construction, timeout, bounded retry, per-provider
minimum request interval, provider concurrency, User-Agent, optional proxy
configuration, and a process-local circuit breaker.

Server configuration stores provider secret references, not secret values.
Provider clients are constructed through the server metadata provider factory,
which resolves secrets from environment variables and sanitized header
configuration. Resolved tokens, API keys, custom header values, and proxy URLs
must not be written to jobs, events, raw response cache records, logs, or
diagnostics API responses.

Metadata refresh records one durable provider attempt per tried provider.
Attempt status has a stable operational vocabulary:

- `succeeded`: provider selected and item metadata was merged;
- `skipped_disabled`: provider was explicitly disabled in config;
- `skipped_unavailable`: provider could not be constructed, usually because
  credentials or configuration were missing;
- `not_implemented`: the profile requested a provider with no registered
  implementation;
- `no_match`: provider ran but found no usable candidate;
- `rate_limited`: provider failed with rate-limit evidence such as HTTP 429;
- `failed`: provider failed for another provider-level reason.

Provider error classes describe retryability more precisely than status alone.
Timeout, rate-limited, network, provider unavailable, and unknown failures are
retry candidates. Auth, unsupported, no-match, and parse failures are not
retry candidates. HTTP status failures are considered retry candidates only
when the caller has status evidence that the provider failure is transient.

Raw provider responses are cached by `(item_id, provider, provider_key)` after
a successful fetch. The cache is an audit and debugging artifact, not the
canonical metadata model. Canonical media, catalog graph tables, and search
documents remain updated through the merge and hydration path.

The HTTP diagnostics boundary exposes:

- `GET /items/{item_id}/metadata/attempts`
- `GET /items/{item_id}/metadata/raw`
- `GET /metadata/providers`

These routes are for operational visibility. They may expose provider names,
status, failure class, failure messages, runtime budgets, whether a proxy is
configured, and raw provider bodies, but must not expose resolved secrets or
proxy URLs.

Future providers must implement `MetadataProvider`, use
`MetadataHttpRuntime`, return provider-specific raw JSON, map provider payloads
into `CanonicalMetadata`, and add tests that prove runtime usage, credential
handling, raw-cache behavior, and attempt persistence.

## Consequences

- TMDB, Bangumi, and Douban share one timeout/retry/rate-limit/circuit-breaker
  implementation.
- Provider failures become diagnosable without inspecting background task logs.
- Clients can show whether an item metadata failure is likely retryable.
- Raw provider cache can support auditing and future refresh diff tools without
  becoming the canonical metadata source.
- Provider construction remains server-owned, which keeps secret resolution out
  of `nako-metadata` and avoids leaking deployment details into provider code.
- Circuit breaker state is currently process-local; multi-process deployments
  would need a separate durable or distributed health boundary.

## Alternatives Considered

- Let every provider use its own HTTP client: rejected because timeout, retry,
  User-Agent, proxy, and circuit behavior would diverge quickly.
- Store provider secrets in SQLite or raw cache rows: rejected for the same
  reasons as ADR 0009.
- Put diagnostics only in job summaries: rejected because summaries are scoped
  to one job and cannot list cached raw responses or provider build status.
- Make raw cache canonical metadata: rejected because provider payloads are not
  stable Nako domain state and should not bypass merge policy or field locks.

## Related Workstreams

- `docs/workstreams/server-foundation/PHASE3_4_METADATA_STRATEGY_EXECUTOR.md`
- `docs/workstreams/server-foundation/PHASE3_6_CATALOG_GRAPH_ARTWORK_SEARCH_SCAN.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0009-resolve-provider-secrets-from-environment.md`
