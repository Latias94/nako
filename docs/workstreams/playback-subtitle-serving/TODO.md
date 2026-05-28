# Playback Subtitle Serving Task Ledger

## PSS-010 - Workstream Boundary

- [x] PSS-010 [owner=codex] [deps=none] [scope=docs/workstreams/playback-subtitle-serving]
  Goal: Open the lane and define host-owned subtitle serving as a playback
  concern rather than addon resource proxying.
  Validation: documentation review.

## PSS-020 - Shared Sidecar Resolution

- [x] PSS-020 [owner=codex] [deps=PSS-010] [scope=crates/nako-server/src/app]
  Goal: Extract safe subtitle sidecar leaf/URI derivation so import write and
  playback read use the same rules.
  Validation: focused unit/API tests covering imported sidecar file names.
  Evidence: `subtitle_sidecar` owns safe leaf, content type, stream fact, and
  storage URI derivation; addon import write now reuses the same helpers.

## PSS-030 - Subtitle Playback API

- [x] PSS-030 [owner=codex] [deps=PSS-020] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/http/playback.rs]
  Goal: Serve sidecar subtitle text by source and subtitle stream index with
  source play access, playback policy checks, and redacted storage errors.
  Validation: `cargo nextest run -p nako-server subtitle --no-fail-fast`.
  Evidence: subtitle route serves sidecar text with content type/length,
  rejects browse-only access, caps sidecar reads, and redacts locator details.

## PSS-040 - Browser Ticket Scope

- [x] PSS-040 [owner=codex] [deps=PSS-030] [scope=crates/nako-client-protocol,crates/nako-api,crates/nako-server/src/app/playback_ticket.rs]
  Goal: Add a browser ticket mode/URL kind for a specific subtitle stream so
  browser track URLs can fetch subtitles without bearer headers.
  Validation: `cargo nextest run -p nako-server browser_playback_ticket --no-fail-fast`;
  `cargo nextest run -p nako-client-protocol browser_playback --no-fail-fast`.
  Evidence: browser ticket mode `subtitle` is scoped to source plus stream
  index, and the public SDK/OpenAPI expose only safe subtitle URLs.

## PSS-050 - Closeout

- [x] PSS-050 [owner=codex] [deps=PSS-020,PSS-030,PSS-040] [scope=docs/workstreams/playback-subtitle-serving]
  Goal: Record evidence and commit the bounded slice.
  Validation: `cargo check -p nako-api -p nako-client -p nako-client-protocol -p nako-server --tests`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: closeout evidence recorded in `EVIDENCE_AND_GATES.md`.
