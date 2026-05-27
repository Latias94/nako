# Casting Renderer Runtime - Milestones

Status: Planned
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- ADR 0040 exists and states the Renderer Session/Adapter boundary.
- Workstream docs agree that casting starts after playback policy/target
  readiness.
- Nako-to-Nako cast is chosen as the first implementation target.

Primary evidence:

- `docs/adr/0040-casting-as-renderer-session-adapter.md`
- `docs/workstreams/casting-renderer-runtime/DESIGN.md`
- `docs/workstreams/casting-renderer-runtime/TODO.md`

## M1 - Readiness And Characterization

Exit criteria:

- Current Playback Session behavior is characterized.
- Missing Renderer Session/control API surface is proven.
- Cast-safe URL and command delivery gaps are explicit.

Primary gates:

- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-client-protocol public --no-fail-fast`

## M2 - Renderer Session Domain

Exit criteria:

- Renderer Session and Renderer Command records exist.
- Persistence decision is explicit: durable repository or process-local first
  with a documented extraction path.
- Records do not conflate renderer, playback, and transcode identities.

Primary gates:

- `cargo nextest run -p nako-core renderer --no-fail-fast`
- `cargo nextest run -p nako-db renderer --no-fail-fast`

## M3 - Nako Remote Client Adapter

Exit criteria:

- Nako client targets can register/heartbeat/update capabilities.
- Public Client API can list controllable targets safely.
- Commands can be delivered or polled without exposing secrets.

Primary gates:

- `cargo nextest run -p nako-server renderer --no-fail-fast`
- `cargo nextest run -p nako-client-protocol public --no-fail-fast`

## M4 - Cast Play Command Flow

Exit criteria:

- Authorized play command creates a Playback Session through the normal
  policy-aware playback app service.
- Denied command creates no Playback Session, Transcode Session, artifact, or
  ticket.
- Progress/heartbeat remains owned by the playback client/renderer session.

Primary gates:

- `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`

## M5 - Diagnostics And External Adapter Follow-Ons

Exit criteria:

- Admin diagnostics show renderer runtime readiness and active target summary.
- Chromecast, DLNA, and AirPlay follow-ons are split with adapter boundaries.
- Cast-safe ticket lifecycle is implemented or explicitly deferred until the
  first no-bearer external adapter.

Primary gates:

- `cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast`
- `cargo nextest run -p nako-server renderer --no-fail-fast`

## M6 - Closeout

Exit criteria:

- Workstream evidence is current.
- `WORKSTREAM.json` status and completed tasks are current.
- Remaining protocol adapter work is split or deliberately deferred.

Primary gates:

- `cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast`
- `cargo nextest run -p nako-client-protocol public --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
