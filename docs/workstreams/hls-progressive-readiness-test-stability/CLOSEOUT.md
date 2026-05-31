# HLS Progressive Readiness Test Stability Closeout

Status: Closed
Closed: 2026-05-31
Task: HPRTS-030

## Closeout Claim

The follow-on stabilized the default full HLS gate that blocked HRLB closeout.
The failure was classified as Windows full-suite process-backed test timing,
not a production HLS runtime bug.

## Delivered

- Repro evidence for two progressive readiness tests that failed in the full
  HLS gate but passed individually.
- A test-only timeout helper for the process-backed progressive readiness
  tests.
- Fresh full HLS verification after the helper change.
- Updated HRLB handoff and evidence so HRLB closeout could be retried.

## Validation

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/hls-progressive-readiness-test-stability/WORKSTREAM.json
python -m json.tool docs/workstreams/hls-runtime-lifecycle-boundary/WORKSTREAM.json
git diff --check
```

Result: passed on 2026-05-31. The full HLS gate ran 71 tests, all passed, with
26 slow tests.

## Follow-Ons

- Broader Windows HLS fixture scheduling or nextest grouping if slow
  process-backed tests continue to grow.
- HRLB follow-ons remain owned by their own proposed lanes, not this
  test-stability workstream.
