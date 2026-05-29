# Playback Runtime Resource Scheduler Closeout

Date: 2026-05-29
Status: Completed

## Result

The lane shipped the single-node playback runtime resource scheduler first
slice.

- Playback resource demand is typed in `nako-server`.
- HLS and remux start paths acquire host-owned permits before process-backed
  runtime work starts.
- Browser playback preflight starts keep permit lifetime with the supervised
  background task.
- Active or completed HLS/remux session reuse does not double-acquire process
  permits.
- Admin runtime diagnostics expose redaction-safe resource pressure with
  configured capacity, available permits, in-use permits, resource class, and
  enforcement mode.
- `nako-transcode` remains responsible for FFmpeg command planning and
  low-level runner semaphores.

## Boundaries Preserved

- Public direct/remux/HLS route contracts remain stable.
- Browser and renderer ticket behavior remains stable.
- FFmpeg stays behind the CLI-first media engine boundary from ADR 0052.
- Runtime policy stays server-owned because admission needs user, session,
  storage, route, and diagnostics context.

## Deferred Follow-ons

- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:remote-transcode-worker-runtime`
- `proposed:playback-os-resource-isolation`
- `proposed:playback-device-capacity-tuning`
- `proposed:hls-artifact-io-pressure-enforcement`
- Existing proposed LL-HLS, DASH/CMAF, and DRM/key-delivery lanes should reuse
  this admission vocabulary.

## Residual Risks

- HLS artifact I/O pressure is modeled but not yet enforced. This is acceptable
  for the first slice because process-backed CPU/GPU/remux pressure is now
  guarded and observable; disk-sensitive enforcement should wait for real
  operator pressure evidence or a dedicated artifact I/O lane.
- There is no durable wait queue. Busy hosts reject bounded playback work
  instead of waiting, which keeps the first slice deterministic.
- Remote transcode workers are still out of scope. The current diagnostics are
  local-host oriented and should become worker-aware in the remote runtime lane.

## Verification

Final closeout verification is recorded in `EVIDENCE_AND_GATES.md`.

- `cargo nextest run -p nako-server playback --no-fail-fast` (132 passed, 331
  skipped)
- `cargo nextest run -p nako-server hls --no-fail-fast` (53 passed, 410
  skipped)
- `cargo nextest run -p nako-server playback_resource --no-fail-fast` (6
  passed, 457 skipped)
- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast` (10
  passed, 1 leaky, 453 skipped)
- `cargo nextest run -p nako-api --no-fail-fast` (69 passed)
- `cargo fmt --all -- --check`
- `python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`
- `git diff --check`
