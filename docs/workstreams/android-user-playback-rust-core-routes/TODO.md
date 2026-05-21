# Android User Playback Rust Core Routes — TODO

Status: Closed
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

- [x] UPC-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-user-playback-rust-core-routes]
  Goal: Freeze user-playback route migration scope, non-goals, and gate set.
  Validation: `python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null`.
  Review: Confirm this lane moves request construction only, not DTO decode,
  Android transport/UI ownership, or request-body ownership.
  Evidence: `DESIGN.md`
  Handoff: DONE. Start with UPC-020.

## M1 — Rust Core User Playback Builders

- [x] UPC-020 [owner=codex] [deps=UPC-010] [scope=crates/taru-client-core]
  Goal: Add explicit User Playback State request-builder inputs and functions
  for get state, continue watching, update progress, and set watched state.
  Validation: `cargo fmt --package taru-client-core --check`; `cargo nextest run -p taru-client-core --no-fail-fast`
  Review: Builders should return complete `CoreHttpRequest` values, include
  auth, preserve safe previews, encode item IDs, add pagination for Continue
  Watching, and attach JSON bodies/content type for write routes without owning
  body serialization.
  Evidence: `crates/taru-client-core/src/user_playback.rs`; tests.
  Handoff: DONE. Added `crates/taru-client-core/src/user_playback.rs`
  with explicit builders for get state, Continue Watching, update progress,
  and set watched state. Core tests cover stable paths, pagination, auth,
  safe previews, JSON content type, body passthrough, and item ID encoding.

## M2 — UniFFI User Playback Binding Surface

- [x] UPC-030 [owner=codex] [deps=UPC-020] [scope=crates/taru-client-uniffi,scripts]
  Goal: Expose FFI-safe user-playback request builders over `taru-client-core`
  and keep `taru-client-uniffi` a thin binding adapter.
  Validation: `cargo fmt --package taru-client-uniffi --check`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `./scripts/guard-uniffi-boundary.ps1`
  Review: Binding records must not expose transport, reqwest/Tokio/platform
  types, DTO decode, or Android-specific diagnostics.
  Evidence: `crates/taru-client-uniffi/src/lib.rs`; boundary guard output.
  Handoff: DONE. Added FFI-safe user-playback request input records and
  explicit UniFFI builder functions over `taru-client-core`. UniFFI tests now
  cover Continue Watching and write routes, and the boundary guard passes.

## M3 — Android User Playback Adapter Migration

- [x] UPC-040 [owner=codex] [deps=UPC-030] [scope=apps/android/app/src/main/java/dev/taru/android/userplayback,apps/android/app/src/test/java/dev/taru/android/userplayback]
  Goal: Add Android `UserPlaybackCore` adapter over UniFFI and migrate
  `TaruUserPlaybackClient` route construction from `TaruPublicClientRequests`
  to Rust core descriptors while keeping generated SDK DTO/body mapping and
  Android diagnostics.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.userplayback.TaruUserPlaybackClientTest --no-daemon --rerun-tasks`
  Review: Generated Kotlin SDK route descriptor calls should be absent from
  runtime user-playback request construction after migration; DTO/body aliases
  may remain.
  Evidence: `TaruUserPlaybackClient.kt`; `RustUserPlaybackCore.kt`; tests.
  Handoff: DONE. Added Android `UserPlaybackCore`/`RustUserPlaybackCore`,
  migrated `TaruUserPlaybackClient` runtime route construction to Rust/UniFFI
  request descriptors, kept generated SDK DTO/body mapping, and preserved local
  missing-token behavior before transport. User-playback JVM tests and the
  route-owner scan pass.

## M4 — Integration Verification And Docs

- [x] UPC-050 [owner=codex] [deps=UPC-040] [scope=apps/android/README.md,docs/workstreams/android-user-playback-rust-core-routes]
  Goal: Update docs and run combined Rust/Android/boundary verification for
  the migrated user-playback route family.
  Validation: Rust core + UniFFI + boundary guard + Android user-playback test gates + route-owner scan.
  Review: Confirm generated SDK role is documented as DTO/body contract
  transition, not runtime user-playback route owner.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. README now documents Rust core user-playback route ownership and generated SDK DTO/body contract transition role. Fresh Rust core, UniFFI, boundary guard, Android user-playback JVM, and route-owner scan gates passed.

## M5 — Closeout

- [x] UPC-090 [owner=planner] [deps=UPC-050] [scope=docs/workstreams/android-user-playback-rust-core-routes]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-user-playback-rust-core-routes/WORKSTREAM.json > $null`; `git diff --check`.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `CLOSEOUT.md`
  Handoff: DONE. Lane closed with fresh evidence, residual risks, and follow-ons in `CLOSEOUT.md`.
