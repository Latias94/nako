# Playback Transcode Ops Hardening — Handoff

Status: Complete
Last updated: 2026-05-22

## Current State

This lane is complete as the post-NFO-sidecar playback/transcode
supportability child of `post-rpd-product-hardening`. Existing
playback/transcode baselines remain owned by `playback-streaming`,
`transcode-runtime`, and `admin-playback-runtime-diagnostics`.

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

PTOH-050 is complete. `GET /admin/v1/playback/support` now returns a bounded
Admin-only support evidence read model for runtime, session, and source
contexts. The response composes existing readiness, session, source, staging,
remote playback, and hardware facts while excluding raw local paths, Source
Locators, FFmpeg paths, command argv, output paths, raw stderr, fingerprints,
secrets, and credentials. The route rejects mismatched session/source query
contexts. Admin TypeScript contract and Admin web typed client/mocks were
updated; Public Client API and `taru-client-protocol` remain unchanged.

PTOH-060 is complete. The lane status, task ledger, milestone evidence, final
JSON/diff gates, parent umbrella re-score, and workstream index now agree.

## Closeout State

- Task ID: PTOH-060
- Status: DONE
- Final scope:
  - `docs/workstreams/playback-transcode-ops-hardening`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Review result: no blocking findings. The target state is met, and remaining
  work is split into follow-ons rather than hidden in this lane.

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
- PTOH-050 keeps support evidence read-only and non-persistent. It exposes a
  stable `request_key_fingerprint` rather than the raw request key in support
  evidence, uses `source_scheme` rather than raw Source Locator data, narrows
  hardware support facts to selected acceleration, fallback, counts, and
  unavailable capability status categories, and moves downloadable bundles,
  retention, or UI workflow beyond this lane unless opened explicitly.

## Blockers

- None.

## Next Recommended Action

`post-rpd-product-hardening` PRPH-120 opened
`downloads-watch-folder-intake`. Continue with DWI-020: durable intake
candidate domain/persistence. That lane should consume Managed Import Staging,
Link Apply, NFO Sidecar Apply, and Playback support evidence instead of
introducing direct library writes or protocol-specific downloader behavior in
core Taru.

Split follow-ons if needed:

- downloadable support bundles, retention, and Admin UI workflows;
- adaptive HLS ladders and durable Optimized Versions;
- direct remote FFmpeg credentials or distributed transcode workers.
