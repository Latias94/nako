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

PTOH-040 is complete. Playback transcode session failures now use a broader
support taxonomy for probe, plan, staging, budget, hardware fallback, runner,
timeout, stale, cancellation, storage, invalid request, and unknown cases.
Persisted app failure messages are redacted operator summaries, and Public
Client session DTOs preserve the coarse public category contract while
redacting raw persisted failure text.

## Active Task

- Task ID: PTOH-050
- Owner: unassigned
- Files:
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-server/src/app/playback`
  - `crates/taru-server/src/http/admin.rs`
  - `crates/taru-server/src/http/tests`
- Validation:
  - `cargo nextest run -p taru-api admin_playback --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: READY
- Review: check Admin API ownership and redaction.

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
- PTOH-040 expands internal persisted failure categories but maps those new
  internal categories back to existing Public Client coarse categories in
  `taru-api::public_client`, so `taru-client-protocol` remains unchanged.
- Public playback session `failure_message` is now derived from the category
  instead of returning raw persisted text. Admin session list still exposes
  only `has_failure_message`; richer evidence belongs to PTOH-050.

## Blockers

- None for PTOH-050.

## Next Recommended Action

Implement PTOH-050 with tests first:

1. define a bounded Admin-only playback support evidence response around
   runtime/session/source context;
2. compose evidence from existing readiness, session category/state, staging,
   and hardware facts;
3. prove the response excludes paths, Source Locators, FFmpeg argv/stderr,
   output paths, secrets, and credentials;
4. split downloadable bundles, retention, or UI work if operators need export
   beyond this read model.
