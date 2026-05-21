# Playback Transcode Ops Hardening — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is newly opened as the next mainline child of
`post-rpd-product-hardening` after NFO sidecar apply closed the remaining
local metadata/library-file mutation risk. Existing playback/transcode
baselines are already complete in `playback-streaming`, `transcode-runtime`,
and `admin-playback-runtime-diagnostics`.

The lane must therefore avoid redoing M7/M25/M56. Its purpose is to harden the
operator support contract: readiness, typed fallback reasons, validation,
failure taxonomy, and bounded redacted support evidence.

## Active Task

- Task ID: PTOH-020
- Owner: unassigned
- Files:
  - `crates/taru-transcode/src/hardware.rs`
  - `crates/taru-server/src/app/playback`
  - `crates/taru-server/src/http/admin.rs`
  - `crates/taru-server/src/http/tests/system.rs`
  - `crates/taru-api/src/admin.rs`
- Validation:
  - `cargo nextest run -p taru-transcode hardware --no-fail-fast`
  - `cargo nextest run -p taru-server admin_v1_playback_runtime --no-fail-fast`
  - `cargo nextest run -p taru-api admin_playback --no-fail-fast`
- Status: READY
- Review: keep the route Admin-only, read-only, and redaction-safe.

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

## Blockers

- None for PTOH-020.

## Next Recommended Action

Implement PTOH-020 with tests first:

1. define the readiness categories and redaction contract;
2. prove them in `taru-transcode` hardware/readiness tests;
3. expose them through Admin DTO/app snapshots;
4. add route-level tests that prove useful readiness and no path/command leak.
