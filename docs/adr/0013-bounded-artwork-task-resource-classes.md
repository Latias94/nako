# 0013: Use Bounded Artwork Task Resource Classes

Status: accepted

## Context

Artwork fetching, image resizing, and video preview extraction stress different
resources. Fetching is network-bound, resizing is CPU-bound, and preview frame
extraction can start ffmpeg processes. Treating all image work as one unbounded
async queue would make poster-heavy pages and large scans unstable.

## Decision

Taru persists artwork work in `artwork_tasks` with explicit kind, status,
resource class, attempts, max attempts, and error state. Core defaults define
separate concurrency limits for fetch, resize, preview, and cleanup work.

The current phase stores the queue and resource policy. Worker execution,
thumbnail generation, ffmpeg preview extraction, and cache eviction are future
phases.

## Consequences

- UI image loading and preview generation have a bounded queue foundation.
- Retries are explicit and auditable instead of hidden in transient tasks.
- Future workers can use semaphores per resource class without changing schema.
- This does not yet generate thumbnails or preview frames.

## Alternatives Considered

- Generate thumbnails synchronously during list routes: rejected because it
  would make browsing latency depend on ffmpeg and image CPU work.
- Reuse only the generic job table: rejected because artwork tasks need
  image-specific ownership and retry state at a finer granularity.

## Related Workstreams

- Server Foundation Phase 3.6
- Server Foundation Phase 4.0
