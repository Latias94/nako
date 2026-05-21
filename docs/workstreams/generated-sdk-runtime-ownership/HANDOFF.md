# Generated SDK Runtime Ownership — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

`SDKRT-010`, `SDKRT-020`, `SDKRT-030`, `SDKRT-035`, `SDKRT-040`, `SDKRT-050`,
and `SDKRT-090` are complete. The workstream is closed.

The prior SDK lanes are closed:

- `android-generated-public-client-sdk` moved Android DTO and route mirrors to
  an OpenAPI-backed Kotlin/JVM SDK.
- `generated-sdk-forward-compat-tolerance` made generated public string values
  tolerant of unknown future wire values.

The runtime ownership question is resolved for this lane: the durable target is
an early shared Rust client core with app-supplied Android transport, and this
lane proved the first Android connection tracer through UniFFI.

Android connection checks now use the Rust core / UniFFI boundary for request
construction and response interpretation. Android still owns HTTP execution,
base URL normalization, cleartext/TLS policy, profile and token storage,
failure categories, user copy, Compose/navigation, UI, and Media3.

## Active Task

None. The workstream is closed.

## Decisions Already Inherited

- ADR 0025: Public Client API OpenAPI v1 is the SDK contract authority.
- ADR 0026: native shells with shared Rust client core are the long-term
  flagship direction.
- ADR 0031: generated client SDK work was sequenced before mobile Rust/UniFFI;
  ADR 0032 now supersedes its post-generated-SDK mobile Rust/UniFFI sequencing.
- ADR 0032: pull shared Rust client core forward behind app-supplied Android
  transport.
- Android UI, navigation, Media3, media sessions, token storage, product copy,
  and platform security policy stay app-owned.
- Generated DTO/request code must remain synchronized from `taru-api` if it is
  generator-owned.

## Frozen Decisions From SDKRT-010

- Select Option E: pull shared Rust client core forward now.
- Do not add a Kotlin SDK runtime layer unless Rust core is later rejected.
- Do not make the first tracer Rust-owned Android networking.
- Start with an FFI-safe no-socket Rust client core and app-supplied Android
  transport.
- Keep `taru-client` as the existing reqwest/async Rust adapter; define whether
  a new `taru-client-core` and `taru-client-uniffi` split is needed in
  `SDKRT-020`.
- Rust core should own protocol-level request construction, API-version
  observation, public error parsing, JSON decode classification, redaction
  primitives, and eventually playback decision/request interpretation.
- Android should own token vaults, profile persistence, cleartext/TLS policy,
  product failure categories, user copy, Compose/navigation, and Media3.

## Frozen Decisions From SDKRT-020

- Create ADR 0032:
  `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`.
- Mark ADR 0031 as superseded for post-generated-SDK mobile Rust/UniFFI
  sequencing.
- Introduce/define `taru-client-core` as the new permissive, no-socket,
  FFI-safe core.
- Keep `taru-client` as the reqwest/async adapter that should later reuse the
  core.
- Put UniFFI scaffolding in a thin `taru-client-uniffi` binding crate later,
  not in the core.
- First tracer: connection health plus authenticated library auth probe with
  Android-supplied transport.
- First tracer may skip library-list DTO decode to avoid strict Rust enum
  tolerance blockers.

## Closeout Decision

`SDKRT-050` selected split-not-broaden. The connection tracer is enough to prove
the first Rust core / UniFFI / Android-supplied transport boundary. Broader
route-family migrations now require separate lanes because they mix Rust wire
tolerance, Gradle/native packaging, product diagnostics, playback semantics,
browse/search semantics, and SDK publishing/KMP policy.

## Blockers

None for the closed lane.

## Recommended Follow-Ons

Recommended order:

1. `android-rust-uniffi-build-ergonomics`: make the Gradle/NDK/UniFFI pipeline
   incremental and package-aware; decide debug/release ABI strategy and symbol
   stripping.
2. `rust-client-core-adapter-reuse`: make `taru-client` reuse
   `taru-client-core` so Rust request/error/version/redaction policy does not
   fork.
3. `rust-public-wire-tolerance`: preserve unknown additive public string values
   in Rust before moving browse/playback DTO decode behind UniFFI.
4. `android-playback-core-tracer`: move only playback decision/request
   interpretation into Rust core while keeping Media3 and player presentation
   Android-owned.
5. `android-browse-core-tracer`: move only browse route interpretation that
   removes duplication without hiding Android product taxonomy.
6. `mobile-sdk-publishing-kmp`: decide Maven publishing, KMP/iOS posture, and
   multi-SDK runtime policy.

Do not silently move profile persistence, token vaults, UI, Media3, or
Rust-owned networking in any of these follow-ons.

## Residual Risks

- Android ordinary app builds now invoke the UniFFI/native-library path. This is
  intentional, but build ergonomics and release packaging need a dedicated lane.
- The connection tracer avoids strict-enum browse/playback decode; Rust wire
  tolerance must be solved before broader Android decode moves to Rust.
- `taru-client` and `taru-client-core` can drift until the adapter-reuse lane
  consolidates request/error/version/redaction policy.
- Android still maps core runtime failures into product categories. That is the
  desired boundary, but each new route-family tracer needs focused tests.

## Verification

Fresh closeout evidence is recorded in
`docs/workstreams/generated-sdk-runtime-ownership/EVIDENCE_AND_GATES.md`.

Closeout docs: `docs/workstreams/generated-sdk-runtime-ownership/CLOSEOUT.md`.
