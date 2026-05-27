# Casting Renderer Runtime Closeout

Status: Closed
Closed: 2026-05-27

## Closeout Claim

This lane is complete for the casting renderer runtime target. Nako now models
casting as Renderer Sessions plus typed Renderer Commands, with Nako-to-Nako
casting implemented before external protocols.

The lane does not claim Chromecast, DLNA, AirPlay, or non-direct remux/HLS cast
transport implementation. Those are split to protocol-specific follow-ons in
`ADAPTER_FOLLOW_ONS.md`.

## Delivered

- Durable Renderer Session and Renderer Command domain records and repository
  adapters.
- Public Client renderer registration, heartbeat, controllable target listing,
  command polling, command completion, and policy-checked play command routes.
- `RendererAppService` for session ownership, TTL, capability normalization,
  target validation, and command lifecycle.
- `CastingAppService` for controller-to-renderer play orchestration through
  the policy-aware Playback App Service.
- Direct-play Nako-to-Nako cast flow that creates a Playback Session, queues a
  play command, and attaches the session to the Renderer Session.
- Redaction-safe Admin route `GET /admin/v1/playback/renderers` with runtime
  readiness, session summary, safe session facts, and adapter readiness.
- Admin Web generated contract coverage for renderer diagnostics.
- ADR 0040 and `ADAPTER_FOLLOW_ONS.md` describing the shared adapter boundary
  for non-direct Nako transport, Chromecast, DLNA, and AirPlay.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `TODO.md` tasks CAST-010 through CAST-070 are complete.
- `DESIGN.md` target state is satisfied for Nako-to-Nako casting and adapter
  contracts.
- ADR 0040 is respected: Playback Session, Transcode Session, Renderer Session,
  Renderer Adapter, and Cast Ticket remain separate concepts.
- External protocol adapters are split instead of being mixed into playback
  routes or the Playback Planner.

### Code Quality

- Blocking: none.
- Important: none.
- Runtime complexity is behind app services: `RendererAppService` and
  `CastingAppService`.
- Admin diagnostics expose summaries and readiness, not owner principals,
  command payload JSON, capability JSON payloads, source locators, local paths,
  bearer tokens, or ticket material.
- Tests exercise public HTTP/API seams and denial behavior rather than private
  implementation details.

### Missing Gates

- None for this lane.

## Final Verification

Fresh closeout gates run on 2026-05-27:

```powershell
cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json
```

Results:

- `nako-server playback/renderer`: 82 passed, 282 skipped.
- `nako-client-protocol public`: 11 passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `WORKSTREAM.json`: parsed.

## Residual Risks

- Non-direct Nako renderer transport is not implemented yet. Current play
  command flow intentionally requires direct-play decisions.
- Chromecast requires receiver app configuration, HTTPS/CORS/external endpoint
  readiness, and cast-safe URL behavior.
- DLNA requires local discovery, trusted network exposure, and limited-control
  semantics.
- AirPlay requires platform/protocol discovery, pairing/auth review, and media
  compatibility mapping.

## Recommended Follow-Ons

See `ADAPTER_FOLLOW_ONS.md`.

Recommended order:

1. Nako remote-client non-direct transport.
2. Chromecast adapter.
3. DLNA renderer adapter.
4. AirPlay adapter.

## Evidence Anchors

- `docs/adr/0040-casting-as-renderer-session-adapter.md`
- `docs/workstreams/casting-renderer-runtime/EVIDENCE_AND_GATES.md`
- `docs/workstreams/casting-renderer-runtime/ADAPTER_FOLLOW_ONS.md`
- `crates/nako-server/src/app/renderer.rs`
- `crates/nako-server/src/app/casting.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-api/src/admin_contract.rs`
