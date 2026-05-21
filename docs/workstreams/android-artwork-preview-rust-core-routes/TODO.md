# Android Artwork And Preview Rust Core Routes — TODO

Status: Closed
Last updated: 2026-05-22

## M0 — Scope And Gate Freeze

- [x] APR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-artwork-preview-rust-core-routes]
  Goal: Freeze artwork + preview generated-SDK route cleanup scope, non-goals,
  and gate set.
  Validation: `python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null`.
  Review: Confirm runtime artwork request construction moves to Rust core while
  Android still owns transport, DTOs, token/profile state, and image loading.
  Evidence: `DESIGN.md`.
  Handoff: DONE. Start with APR-020.

## M1 — Rust Core Artwork Request Builder

- [x] APR-020 [owner=codex] [deps=APR-010] [scope=crates/taru-client-core,crates/taru-client-uniffi]
  Goal: Add selected artwork image request construction to `taru-client-core`
  and expose it through thin UniFFI bindings.
  Validation: `cargo fmt --package taru-client-core --check`; `cargo nextest run -p taru-client-core --no-fail-fast`; `cargo fmt --package taru-client-uniffi --check`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `./scripts/guard-uniffi-boundary.ps1`.
  Review: Builder must encode image ids, preserve optional width/height query
  parity, inject bearer auth, and redact safe previews without exposing transport
  or platform types.
  Evidence: Rust code and tests.
  Handoff: DONE. Added Rust core and UniFFI selected artwork image request
  builders with tests covering encoded image ids, optional variant query,
  bearer auth, and redaction-safe previews. Rust core, UniFFI, and boundary
  gates pass.

## M2 — Android Runtime Artwork Migration

- [x] APR-030 [owner=codex] [deps=APR-020] [scope=apps/android/app/src/main/java/dev/taru/android/artwork,apps/android/app/src/test/java/dev/taru/android/artwork,apps/android/app/src/test/java/dev/taru/android/ui/artwork]
  Goal: Replace generated SDK descriptor use in `PublicArtworkSource` with an
  Android `ArtworkCore` seam backed by Rust/UniFFI request descriptors.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.artwork.PublicArtworkTest --tests dev.taru.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks`.
  Review: Runtime artwork code must reject unsafe/stale DTO URLs and must not
  import generated route descriptor APIs.
  Evidence: Android artwork tests.
  Handoff: DONE. Added Android `ArtworkCore`/`RustArtworkCore`, migrated
  `PublicArtworkSource` to Rust/UniFFI request descriptors, and preserved exact
  DTO URL-to-core-route safety validation. Android artwork/resolver tests pass.

## M3 — Preview/Test Fixture Route Cleanup

- [x] APR-040 [owner=codex] [deps=APR-030] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/main/java/dev/taru/android/connection]
  Goal: Remove generated SDK route matching from Compose preview fake transport
  and delete dead generated descriptor adapter code if no callers remain.
  Validation: `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon`; route-owner scans.
  Review: Preview code should use fixture-owned route helpers; app runtime
  `src/main` should no longer import `TaruPublicClientRequests` or
  `TaruRequestDescriptor`.
  Evidence: route-owner scan and compile gate.
  Handoff: DONE. Replaced browse preview generated SDK route matching with
  preview-local fixture route helpers and deleted the dead generated descriptor
  adapter. Android compile gate and route-owner scans pass.

## M4 — Integration Verification And Docs

- [x] APR-050 [owner=codex] [deps=APR-040] [scope=apps/android/README.md,docs/workstreams/android-artwork-preview-rust-core-routes]
  Goal: Update docs and run combined Rust/Android/route-owner verification.
  Validation: Rust core + UniFFI + boundary guard + Android artwork/resolver +
  compile gate + route-owner scans + `git diff --check`.
  Review: Generated SDK role remains DTO/contract/test-only, not Android runtime
  route owner.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE. README documents selected artwork Rust-core route ownership
  and preview fixture route-helper policy. Full Rust, UniFFI, Android,
  route-owner, preview, JSON, and diff gates pass.

## M5 — Closeout

- [x] APR-090 [owner=planner] [deps=APR-050] [scope=docs/workstreams/android-artwork-preview-rust-core-routes]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null`; `git diff --check`.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `CLOSEOUT.md`.
  Handoff: DONE. Lane closed with final evidence, residual risks, and
  follow-ons in `CLOSEOUT.md`.
