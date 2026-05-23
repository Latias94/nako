# Official Addon E2E Alpha2 - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The workstream has been opened and scoped. Nako `v0.1.0-alpha.1` is published
with release artifacts, public crates, and GHCR images. The official companion
repository has `nako-metadata-scraper@0.1.0-alpha.1`.

OAE2E-020 is complete. The published Nako GHCR image can run
`nako-server --help` and `ffmpeg -version`. The official metadata scraper can
run locally in fixture/default mode and passes its direct sidecar smoke without
provider secrets.

## Next Task

Start OAE2E-030.

Goal: prove Nako can register `nako-metadata-scraper`, check hosted health, and
make one hosted resource diagnostic call through Nako's Addon runtime.

Suggested first steps:

1. Choose a local Nako config that uses SQLite and throwaway temp directories.
2. Start Nako with `NAKO_ADMIN_TOKEN` set.
3. Start `nako-metadata-scraper` on `127.0.0.1:9100`.
4. Run `smoke.local.ps1 -RegisterInNako -Enable -RunResourceCall`.
5. Record redacted output and any host-side diagnostic gaps.

## Known Risks

- CI may need fixture-only mode if real provider APIs require secrets.
- Running both services locally may need stable port assignment and token
  redaction.
- Addon registration may expose a missing host-side admin/smoke seam; split it
  only if it is not a small fix.
