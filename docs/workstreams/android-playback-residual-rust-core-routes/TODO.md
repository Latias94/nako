# Android Playback Residual Rust Core Routes — TODO

Status: Closed
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

- [x] PRR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-playback-residual-rust-core-routes]
  Goal: Freeze residual playback route migration scope, non-goals, cleanup
  candidates, and gate set.
  Validation: `python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null`.
  Review: Confirm this lane moves playback request construction only, not DTO
  decode, Android transport/UI ownership, or server API shape.
  Evidence: `DESIGN.md`
  Handoff: DONE. Start with PRR-020.

## M1 — Rust Core Residual Playback Builders

- [x] PRR-020 [owner=codex] [deps=PRR-010] [scope=crates/taru-client-core]
  Goal: Add explicit request builders for source probe, playback session
  inspection, and playback session cancellation.
  Validation: `cargo fmt --package taru-client-core --check`; `cargo nextest run -p taru-client-core --no-fail-fast`
  Review: Builders should return complete `CoreHttpRequest` values, include
  auth, preserve safe previews, encode source/session IDs, and use stable
  methods (`GET`, `GET`, `POST`).
  Evidence: `crates/taru-client-core/src/playback.rs`; tests.
  Handoff: DONE. Added core builders for source probe, playback session
  inspection, and playback session cancellation. Core tests cover stable
  methods, encoded source/session IDs, auth injection, and safe previews.

## M2 — UniFFI Residual Playback Surface

- [x] PRR-030 [owner=codex] [deps=PRR-020] [scope=crates/taru-client-uniffi,scripts]
  Goal: Expose FFI-safe residual playback request builders over
  `taru-client-core` and keep `taru-client-uniffi` thin.
  Validation: `cargo fmt --package taru-client-uniffi --check`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `./scripts/guard-uniffi-boundary.ps1`
  Review: Binding records must not expose transport, reqwest/Tokio/platform
  types, DTO decode, or Android-specific diagnostics.
  Evidence: `crates/taru-client-uniffi/src/lib.rs`; boundary guard output.
  Handoff: DONE. Added FFI-safe residual playback request input records and
  explicit UniFFI builder functions over `taru-client-core`. UniFFI tests now
  cover source probe, session inspect, and session cancel routes, and the
  boundary guard passes.

## M3 — Android Playback Migration And Cleanup

- [x] PRR-040 [owner=codex] [deps=PRR-030] [scope=apps/android/app/src/main/java/dev/taru/android/playback,apps/android/app/src/main/java/dev/taru/android/browse,apps/android/app/src/test/java/dev/taru/android/playback]
  Goal: Extend Android `PlaybackCore`/`RustPlaybackCore`, migrate
  `TaruPlaybackClient` residual runtime route construction away from generated
  SDK descriptors, and delete confirmed dead compatibility helpers.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon --rerun-tasks`
  Review: Generated Kotlin SDK route descriptor calls should be absent from
  runtime playback request construction after migration; DTO aliases may remain.
  Evidence: `TaruPlaybackClient.kt`; `RustPlaybackCore.kt`; cleanup diff; tests.
  Handoff: DONE. Extended Android `PlaybackCore`/`RustPlaybackCore`, migrated
  `TaruPlaybackClient` residual route construction to Rust/UniFFI request
  descriptors, and deleted the confirmed-dead `toSdkPageQuery` helper.
  Playback JVM tests, playback route-owner scan, and dead-helper scan pass.

## M4 — Integration Verification And Docs

- [x] PRR-050 [owner=codex] [deps=PRR-040] [scope=apps/android/README.md,docs/workstreams/android-playback-residual-rust-core-routes]
  Goal: Update docs and run combined Rust/Android/boundary verification for the
  migrated residual playback route family.
  Validation: Rust core + UniFFI + boundary guard + Android playback test gates + route-owner/dead-helper scans.
  Review: Confirm generated SDK role is documented as DTO/contract transition,
  not runtime playback route owner.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. README now documents complete Rust core playback runtime route ownership. Fresh Rust core, UniFFI, boundary guard, Android playback JVM, route-owner scan, and dead-helper scan gates passed.

## M5 — Closeout

- [x] PRR-090 [owner=planner] [deps=PRR-050] [scope=docs/workstreams/android-playback-residual-rust-core-routes]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-playback-residual-rust-core-routes/WORKSTREAM.json > $null`; `git diff --check`.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `CLOSEOUT.md`
  Handoff: DONE. Lane closed with fresh evidence, residual risks, and follow-ons in `CLOSEOUT.md`.
