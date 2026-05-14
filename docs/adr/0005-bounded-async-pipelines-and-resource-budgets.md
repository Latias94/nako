# 0005: Use Bounded Async Pipelines and Resource Budgets

## Status

Proposed

## Context

Taru will run work that has very different resource profiles:

- directory scanning and remote listing
- file probing through ffprobe
- metadata provider calls
- image downloads and caching
- webhook delivery
- automation calls to external API providers
- FFmpeg remux and transcode sessions
- future remote-drive staging and byte-range cache work

All of these are naturally asynchronous, but unconstrained async fan-out can
make a self-hosted media server unstable. Too many concurrent probes can thrash
disk. Too many remote reads can hit provider rate limits. Too many transcodes
can exhaust CPU, GPU, memory, or temporary storage. A strong foundation needs
explicit backpressure and resource ownership from the first implementation.

## Decision

Use bounded async pipelines throughout Taru. Async is the default execution
model for I/O and external process orchestration, but every concurrent pipeline
must have an explicit limit, a failure policy, and a persistence boundary.

Core rules:

- Never spawn unbounded work from user input, scan results, provider results, or
  addon responses.
- Every fan-out pipeline must use a configured concurrency limit.
- External work must be cancel-safe at the orchestration boundary.
- Single-item failure should not fail a whole batch unless the caller requested
  strict behavior.
- Job progress must be representable as persisted state or reproducible from
  persisted inputs.
- Repository writes must be idempotent where repeated jobs are expected.
- Resource classes should be named explicitly, for example `disk.scan`,
  `media.probe`, `network.metadata`, `network.webhook`, `cpu.transcode`,
  `gpu.transcode`, `storage.remote`, and `automation.external_api`.
- Defaults should be conservative for self-hosted systems, with configuration
  available later for users with stronger hardware.

Initial defaults:

- media probing: bounded, default 2 concurrent ffprobe processes
- library scanning: sequential per root until directory cache and provider
  limits are implemented
- metadata provider calls: bounded per provider when providers are added
- transcode sessions: bounded separately from probe and scan work

## Consequences

- Throughput is more predictable under large libraries.
- Remote storage and API provider integrations can share the same backpressure
  model instead of inventing per-feature throttling.
- The code has more plumbing than naive async fan-out.
- Tests must verify idempotency, partial failure behavior, and concurrency
  limits for every new pipeline.
- Configuration and observability become first-class follow-up work.

## Alternatives Considered

- Unbounded `join_all` or task spawning: simple but unsafe for large libraries
  and remote providers.
- Fully serial execution: safest but too slow for probing, downloads, metadata
  refresh, and webhook delivery.
- A full distributed job queue immediately: powerful but premature for the
  modular monolith MVP.

## Related Workstreams

- `docs/workstreams/server-foundation/`
