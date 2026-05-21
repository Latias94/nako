# Playback Transcode Ops Hardening — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is active as the next mainline child of `post-rpd-product-hardening`
after NFO sidecar apply closed the remaining local metadata/library-file
mutation risk. Existing playback/transcode baselines are already complete in
`playback-streaming`, `transcode-runtime`, and
`admin-playback-runtime-diagnostics`.

PTOH-020 is complete. It adds a typed readiness contract across
`taru-transcode`, Admin API DTOs, Admin HTTP runtime diagnostics, and the Admin
TypeScript contract. `GET /admin/v1/playback/runtime` now reports a top-level
readiness state plus stable check entries for FFmpeg probe, hardware
acceleration, selected fallback, transcode budget, remote playback budget, and
staging.

PTOH-030 is complete. Playback transcode profile/plan validation now uses
typed reason categories in `taru-transcode`, Result-returning construction
seams in `taru-streaming`, and pre-session propagation in
`taru-server::app::playback`.

## Active Task

- Task ID: PTOH-040
- Owner: unassigned
- Files:
  - `crates/taru-core`
  - `crates/taru-transcode`
  - `crates/taru-streaming/src`
  - `crates/taru-server/src/app/playback`
  - `crates/taru-server/src/http/tests`
- Validation:
  - `cargo nextest run -p taru-server playback --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - package-specific DB tests only if persisted categories change
- Status: READY
- Review: check redaction, persistence compatibility, and client error
  stability.

## Decisions Since Opening

- Use this lane for playback/transcode supportability, not new streaming
  features.
- Treat `GET /admin/v1/playback/runtime` as the starting surface for
  server-wide readiness.
- Keep Public Client API behavior stable unless a dedicated client-contract
  follow-on is opened.
- Split full Transcode Profile product work if validation grows beyond
  playback transcode request safety.
- Split downloadable/exported support bundles if the first read model needs
  persistence or file export.
- PTOH-020 keeps the route Admin-only and read-only. Admin web generated
  contract/mocks were updated only because the Admin contract generator
  requires app-local generated output to match; no UI behavior was added.
- Public Client API and `taru-client-protocol` were not changed.
- PTOH-030 keeps validation in the narrowest owning crate:
  `taru-transcode` validates profile/plan facts, `taru-streaming` composes
  playback profiles through Result-returning seams, and `taru-server` uses
  those seams before request identity, staging, or session creation.

## Blockers

- None for PTOH-040.

## Next Recommended Action

Implement PTOH-040 with tests first:

1. identify current playback failure surfaces across validation, staging,
   budget, runner startup, timeout, cancellation, and hardware fallback;
2. add stable support categories without leaking paths, locators, command argv,
   raw stderr, or credentials;
3. preserve existing public client DTO behavior unless a separate client
   contract lane is explicitly opened;
4. split persistence or support bundle export if failure taxonomy grows into a
   broader evidence-retention feature.
