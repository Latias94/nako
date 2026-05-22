# Phase 13.0: Metadata Maintenance Job Boundary

## Goal

Promote metadata refresh from item-level diagnostics to a library maintenance
operation that can be queued, summarized, inspected, and cleaned up without
leaking provider secrets.

## Architecture

- `nako-server` owns job creation, library/item scope resolution, profile
  override application, and outbox events.
- `nako-metadata` still owns provider runtime, provider registry, fallback
  strategy, merge policy, raw response caching, and attempt persistence.
- `nako-db` exposes repository methods for library item listing, attempt
  filtering, and raw cache cleanup.
- `nako-api` exposes request and response envelopes; it does not carry resolved
  provider credentials.

## Job Semantics

`metadata_maintenance` jobs accept either:

- `library_id`: refresh all indexed items that belong to one library; or
- `item_ids`: refresh an explicit item set.

The request may override:

- providers;
- full metadata profile;
- item kinds;
- language;
- refresh mode;
- force behavior.

Force maps to `full_refresh` when no explicit refresh mode is provided.

The job succeeds when the maintenance pass completes, even when individual
items fail. Per-item failures are counted in the job summary and kept out of
outbox payloads.

## Diagnostics

Provider attempts can be filtered by provider and status. Raw provider responses
can be filtered by provider and cleaned by provider plus cutoff timestamp. If a
cleanup call does not provide a cutoff, the server computes one from the
configured raw cache retention.

Provider diagnostics expose process-local runtime state:

- circuit breaker open/closed;
- consecutive failure count;
- last error;
- last rate-limit wait;
- runtime scope.

The scope is explicitly `process_local`; it is not a distributed health view.

## Secret Handling

Resolved tokens, API keys, header values, and proxy URLs must not appear in:

- job inputs;
- job summaries;
- outbox payloads;
- diagnostics responses;
- raw cache cleanup responses.

Diagnostics may expose that a proxy is configured, but not the proxy URL.

## Validation

Required tests:

- maintenance job summary for a library scope;
- job input does not include provider secrets;
- HTTP route enqueues a maintenance job;
- attempts can be filtered by provider/status;
- raw cache can be filtered and cleaned;
- provider diagnostics include process-local runtime health.
