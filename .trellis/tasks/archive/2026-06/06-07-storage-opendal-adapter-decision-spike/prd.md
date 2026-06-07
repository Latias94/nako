# OpenDAL Adapter Decision Spike

## Goal

Decide whether Nako should adopt OpenDAL behind `StorageBackend`, or defer
the dependency and keep Nako-owned storage semantics intact for now.

## Context

- Existing storage semantics already include `StorageUri`, Source Locator
  redaction, Source Fingerprint evidence, storage health, cache repair
  authority, deterministic staging, and Admin-safe diagnostics.
- OpenDAL 0.57.0 provides broad backend support, including filesystem,
  WebDAV, and S3-style services plus retry and timeout layers.
- Nako's current WebDAV and local filesystem backends already enforce product
  policy that OpenDAL does not own.

## Decision Criteria

1. Preserve Nako's storage domain model and redaction guarantees.
2. Preserve capability narrowing, especially for WebDAV read-only behavior.
3. Preserve byte-range and streaming semantics without hidden whole-object
   buffering.
4. Preserve error mapping so raw backend details do not leak.
5. Preserve runtime policy around retry, timeout, and resource budgets.

## Deliverables

- A written decision: reject, defer, or adopt narrowly.
- Updated architecture evidence if the decision changes the next storage lane.
- Curated task context and research files for future follow-on work.

## Non-Goals

- No production dependency change in the first spike.
- No schema, public API, or config shape change.
- No replacement of Nako's storage product model.

## Exit Criteria

- The decision is explicit and recorded in task evidence.
- If the answer is "adopt narrowly," the follow-on shape is named and bounded.
- If the answer is "defer" or "reject," the reason is specific enough to stop
  repeated re-litigation.

