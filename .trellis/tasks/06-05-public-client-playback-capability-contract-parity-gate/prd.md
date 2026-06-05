# Public Client playback capability contract parity gate

## Goal

Add a serial-first contract gate that keeps the current Public Client playback
capability fields aligned across protocol DTOs, OpenAPI/SDK generation, Rust
client request builders, server query/body mapping, renderer capability
mapping, and HTTP API documentation.

This task should close the known flat-field drift before Nako starts playback
profile v2, HEVC/AV1 output execution, hardware tone mapping, image subtitle
burn-in, renderer/device profile fixtures, or Admin effective-profile support
evidence.

## Requirements

- Inventory the current Public Client playback capability/request-preference
  fields across:
  - `nako-client-protocol` browser ticket, playback, HLS, remux, and renderer
    DTOs;
  - `nako-api` Public Client OpenAPI and SDK generation;
  - `nako-client-core` playback request builders and safe previews;
  - `nako-client` async Rust SDK methods and streaming request builders;
  - `nako-server` playback query mapping, browser ticket body mapping, and
    renderer media capability mapping;
  - generated Public Client SDK surfaces when they are produced by the current
    generator;
  - `docs/api/HTTP_API.md`.
- Treat current flat fields as the compatibility baseline, including direct
  play request preference, container/video/audio codec fields, remux output
  container, bitrate/resolution/audio-channel limits, HDR/subtitle booleans,
  HLS variant policy, and HLS segment container.
- Add parity tests or schema assertions that fail when supported Public Client
  playback capability fields drift between protocol DTOs, OpenAPI/SDK output,
  Rust client builders, server mapping, and docs.
- Preserve the audience boundary: Public Client capability fields describe
  client/player facts and request preferences only.
- Keep Admin-only diagnostics, FFmpeg command facts, hardware probe facts,
  GPU/device paths, runtime resource pressure, operator fallback policy, bearer
  tokens, principal IDs, raw source locators, local paths, and transcode
  internals out of Public Client capability DTOs and generated public outputs.
- Keep playback behavior unchanged. This gate must not alter planner decisions,
  transcode runtime behavior, HLS artifact identity, ticket semantics, auth,
  or renderer control behavior.
- If the implementation discovers that a field can change planner output but is
  missing from `PlaybackTargetProfile::identity` or planner tests, stop and
  either narrow this task back to parity-only work or split a dedicated playback
  identity task.

## Acceptance Criteria

- [ ] A field inventory identifies the current Public Client playback
      capability/request-preference fields and the surfaces that expose,
      generate, send, parse, document, or map them.
- [ ] Current supported fields are aligned across protocol DTOs,
      OpenAPI/SDK generation, Rust client/client-core builders, server
      playback/renderer mapping, and HTTP API docs, or any intentionally
      unsupported generated surface is explicitly documented and protected by a
      failing/guarding test.
- [ ] Parity tests or generator assertions fail when a server-accepted playback
      capability field is missing from protocol/OpenAPI/client/docs surfaces.
- [ ] Public Client route inventory and generated public outputs remain free of
      Admin-only routes, DTOs, diagnostics, and governance concepts.
- [ ] Tests prove no Public Client playback capability DTO or generated public
      output exposes FFmpeg, GPU/device, operator policy, resource pressure,
      bearer token, principal/source identity, local path, or raw locator facts.
- [ ] Existing playback decision, ticket, remux, HLS, renderer, and client
      request behavior remains stable except for the intended contract parity
      coverage.
- [ ] The task result records the next allowed follow-on:
      `playback-output-profile-v2-skeleton-contract-only`.

## Definition Of Done

- The gate is implemented with focused contract tests and any necessary
  generator/client/doc updates.
- Generated outputs, if touched, are regenerated from their source generator
  rather than hand-edited.
- Current flat playback capability fields round-trip through the supported
  Public Client contract surfaces.
- The PR or task notes identify any unsupported generated SDK surface that is
  intentionally deferred and the test that protects that decision.
- Required validation commands pass, including whitespace checks.
- No profile v2, HEVC/AV1 execution, hardware tone-map execution, image
  subtitle burn-in, Admin support evidence expansion, or runtime policy change
  is included in this task.

## Out Of Scope

- No new profile v2 fields such as `profile_id`, `profile_version`,
  `device_family`, `player_engine`, profile rows, subtitle delivery matrices,
  audio output matrices, color pipeline matrices, or HLS output codec matrices.
- No HEVC/AV1 executable output path.
- No hardware tone-map execution.
- No image subtitle burn-in execution.
- No Admin playback effective-profile support evidence.
- No Addon, remote access, source fingerprint hash, storage/VFS, durable job,
  schema, auth, or network endpoint discovery changes.
- No generated Admin contract work.
- No broad refactor of playback, transcode, API generation, or client crates
  beyond what the parity gate requires.

## Technical Notes

- Parent task:
  `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Primary parent research:
  - `research/next-lane-synthesis.md`
  - `research/next-parallel-contract-gates.md`
  - `research/next-product-development-lanes.md`
- Completed playback audit:
  `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/`
- Relevant specs:
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/nako-client-protocol/backend/index.md`
  - `.trellis/spec/nako-client-core/backend/index.md`
  - `.trellis/spec/nako-client/backend/index.md`
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-playback/backend/quality-guidelines.md`
  - `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
- `nako-client-protocol` owns Public Client wire DTOs and route inventory.
- `nako-api` owns OpenAPI/SDK generation from the public contract.
- `nako-client-core` owns transport-neutral request builders and redacted safe
  previews.
- `nako-client` owns the async Rust SDK surface.
- `nako-server` owns HTTP query/body mapping, access/auth, renderer mapping,
  and app-service orchestration.
- `nako-playback` owns pure planner facts and profile identity. It should only
  change in this task if parity work proves a currently supported field already
  affects planner output but is not covered by identity/tests.
- `nako-transcode` owns runtime capability, HLS profile, artifact, and FFmpeg
  command planning. It should not change in this parity-only task.

## Validation Commands

Run focused gates first, then broaden only if the touched files require it:

```powershell
cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-client-core -p nako-client --no-fail-fast
cargo check -p nako-client-protocol -p nako-client-core -p nako-client --tests
cargo check -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

If Public Client SDK or OpenAPI generated outputs are touched, also run the
matching generator/example commands from `crates/nako-api` and compare the
committed outputs. If server playback or renderer mapping changes, add and run
focused `nako-server` playback/renderer route tests. If planner output can
change, add and run focused `nako-playback` profile identity tests before
continuing.

## Stop Conditions

- Another active worker is editing the same Public Client playback DTOs,
  generated Public Client SDK surfaces, server playback/renderer mapping, or
  HTTP API docs.
- A required fix needs profile v2, HEVC/AV1, hardware tone-map, image subtitle,
  Admin support evidence, schema, config, Addon, or durable-job changes.
- A proposed Public Client field carries Admin/runtime/operator facts rather
  than client/player facts.
- A generated output must be hand-edited because the generator cannot produce
  the desired contract.
- The current parity gate cannot be expressed without changing playback
  behavior.
