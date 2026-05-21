# Android Browse/Catalog Rust Core Routes — TODO

Status: Closed
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

- [x] BCR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-browse-catalog-rust-core-routes]
  Goal: Freeze browse/catalog route migration scope, non-goals, and gate set.
  Validation: `python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null`.
  Review: Confirm this lane moves request construction only, not DTO decode or
  Android transport/UI ownership.
  Evidence: `DESIGN.md`
  Handoff: DONE. Start with BCR-020.

## M1 — Rust Core Browse Request Builders

- [x] BCR-020 [owner=codex] [deps=BCR-010] [scope=crates/taru-client-core]
  Goal: Add explicit browse/catalog request-builder inputs and functions for
  libraries, library sources, items, item images, people, genre/tag facets, and
  search while preserving existing core public behavior.
  Validation: `cargo fmt --package taru-client-core --check`; `cargo nextest run -p taru-client-core --no-fail-fast`
  Review: Builders should return complete `CoreHttpRequest` values and reuse
  generic request/redaction/encoding policy; avoid generic string-helper APIs as
  the Android seam.
  Evidence: `crates/taru-client-core/src/browse.rs`; tests.
  Handoff: DONE. Added `crates/taru-client-core/src/browse.rs` with explicit
  request builders for libraries, library sources, items, item images, people,
  person items, genres/tags, facet items, and search. Core tests cover stable
  URLs, pagination, auth injection, safe previews, facet encoding, and search
  query/facet encoding.

## M2 — UniFFI Browse Binding Surface

- [x] BCR-030 [owner=codex] [deps=BCR-020] [scope=crates/taru-client-uniffi,scripts]
  Goal: Expose FFI-safe browse request builders over `taru-client-core` and keep
  `taru-client-uniffi` a thin binding adapter.
  Validation: `cargo fmt --package taru-client-uniffi --check`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `./scripts/guard-uniffi-boundary.ps1`
  Review: Binding records must not expose reqwest/Tokio/platform types or DTO
  decode policy.
  Evidence: `crates/taru-client-uniffi/src/lib.rs`; boundary guard output.
  Handoff: DONE. Added FFI-safe page/entity/search browse request input
  records and explicit UniFFI browse request builder functions over
  `taru-client-core`. UniFFI tests now cover libraries, search, and tag facet
  routes, and the dependency boundary guard still passes.

## M3 — Android Browse Adapter Migration

- [x] BCR-040 [owner=codex] [deps=BCR-030] [scope=apps/android/app/src/main/java/dev/taru/android/browse,apps/android/app/src/test/java/dev/taru/android/browse]
  Goal: Add Android `BrowseCore` adapter over UniFFI and migrate
  `TaruBrowseClient` route construction from `TaruPublicClientRequests` to Rust
  core descriptors while keeping Kotlin SDK DTO decode and Android diagnostics.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon`
  Review: Generated Kotlin SDK route descriptor calls should be absent from
  runtime browse request construction after migration; DTO aliases may remain.
  Evidence: `TaruBrowseClient.kt`; `RustBrowseCore.kt`; browse tests.
  Handoff: DONE. Added Android `BrowseCore`/`RustBrowseCore`, migrated
  `TaruBrowseClient` runtime route construction to Rust/UniFFI request
  descriptors, and kept Kotlin SDK DTO decode plus Android diagnostics. Browse
  JVM tests pass. `TaruPublicClientRequests` is no longer used by
  `TaruBrowseClient`.

## M4 — Integration Verification And Docs

- [x] BCR-050 [owner=codex] [deps=BCR-040] [scope=apps/android/README.md,docs/workstreams/android-browse-catalog-rust-core-routes]
  Goal: Update docs and run combined Rust/Android/boundary verification for the
  migrated browse route family.
  Validation: Rust core + UniFFI + boundary guard + Android browse test gates.
  Review: Confirm generated SDK role is documented as DTO/contract transition,
  not runtime browse route owner.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. README now documents Rust core browse/catalog route ownership and generated SDK DTO/contract transition role. Fresh Rust core, UniFFI, boundary guard, Android browse JVM, and route-owner scan gates passed.

## M5 — Closeout

- [x] BCR-090 [owner=planner] [deps=BCR-050] [scope=docs/workstreams/android-browse-catalog-rust-core-routes]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null`; `git diff --check`.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `CLOSEOUT.md`
  Handoff: DONE. Lane closed with fresh evidence, residual risks, and follow-ons in `CLOSEOUT.md`.
