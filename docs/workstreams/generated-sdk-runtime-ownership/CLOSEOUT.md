# Generated SDK Runtime Ownership — Closeout

Status: Closed
Closed: 2026-05-21

## Closeout Claim

The Generated SDK Runtime Ownership lane is complete. Nako has pulled the
shared Rust client-core direction forward behind an app-supplied Android
transport, proved the first Android connection tracer through UniFFI, and
closed without broadening unrelated route families into this lane.

ADR 0032 is now the target-state authority. ADR 0031 remains historical context
for why generated SDK adoption happened before mobile Rust/UniFFI work.

## Delivered

- Frozen ownership matrix for generated SDK, Android app, Rust client core,
  UniFFI binding, `nako-client`, and follow-on SDK/runtime surfaces.
- ADR 0032:
  `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`.
- `crates/nako-client-core`:
  - no-socket, FFI-safe connection probe state machine;
  - health request construction;
  - authenticated library auth-probe request construction;
  - API-version observation;
  - public error-envelope parsing;
  - invalid-response classification;
  - bearer-token redaction in safe request previews.
- `crates/nako-client-uniffi`:
  - thin UniFFI binding surface over `nako-client-core`;
  - no runtime policy in the binding crate.
- `crates/nako-uniffi-bindgen`:
  - repository-pinned bindgen entrypoint for Gradle and local validation.
- Android connection consumption:
  - Gradle generates UniFFI Kotlin bindings and Android ABI native libraries;
  - `RustConnectionCore` adapts generated bindings to Android connection code;
  - `NakoConnectionClient` uses the Rust core for connection probe
    request/response interpretation.
- Android still owns:
  - HTTP execution;
  - base URL normalization;
  - cleartext/TLS policy;
  - token vaults and profile storage;
  - connection failure categories and user copy;
  - diagnostics presentation;
  - Compose/navigation/UI;
  - Media3 and playback sessions.

## SDKRT-050 Decision: Split, Not Broaden

The lane intentionally stops after the connection tracer.

The tracer has already proved the architectural seam that mattered:

1. Rust can own portable protocol-level request construction and response
   interpretation.
2. UniFFI can stay thin and policy-free.
3. Android can supply transport and keep platform/product policy.
4. Ordinary Android build/test/assemble commands can consume generated bindings
   and native libraries.

Broader migration should not be hidden here. Browse and playback are not just
"more of the same"; they require additional decisions about Rust wire-value
tolerance, product diagnostics, playback presentation boundaries, route-family
test matrices, `nako-client` reuse, and build packaging.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `SDKRT-010` through `SDKRT-090` are complete.
- ADR 0032 resolves the ADR impact required by `SDKRT-020`.
- Android product/platform ownership remained outside the Rust core and UniFFI
  binding.
- Remaining work is split into follow-ons rather than hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- The core is no-socket and FFI-safe; it does not expose `reqwest`, async
  traits, platform exceptions, borrowed Rust data, or Android types across
  UniFFI.
- The UniFFI crate delegates to `nako-client-core` and does not become a policy
  owner.
- Android maps core outcomes into existing connection diagnostics rather than
  moving user-facing taxonomy into Rust.
- The first tracer avoids browse/playback strict-enum decode until Rust public
  wire tolerance is solved.

### Missing Gates

- None for the closeout claim. Fresh command evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Fresh Closeout Gates

Passed on 2026-05-21:

- `cargo fmt --package nako-client-core --package nako-client-uniffi --check`
- `cargo nextest run -p nako-client-core --no-fail-fast`
- `cargo nextest run -p nako-client-uniffi --no-fail-fast`
- `cargo run -p nako-uniffi-bindgen -- --help`
- `apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon`
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon`
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
- `cargo nextest run -p nako-client --no-fail-fast`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo fmt --package nako-api --check`
- `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`
- `npm run check --prefix sdk/typescript`
- `cargo nextest run -p nako-api --no-fail-fast`
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `cargo fmt --package nako-client --package nako-client-protocol --check`
- `python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null`
- `git diff --check`

## Follow-Ons Split From This Lane

Recommended order:

1. **Android Rust/UniFFI build ergonomics**
   - Make Gradle/native builds incremental and package-aware.
   - Decide debug/release ABI matrix, CI/device gates, stripping, and native
     artifact packaging.
2. **Rust client-core adapter reuse**
   - Refactor `nako-client` to reuse `nako-client-core` so Rust
     request/error/version/redaction policy does not fork.
3. **Rust public wire tolerance**
   - Preserve unknown additive public string values in Rust protocol/core
     surfaces before mobile browse/playback DTO decode moves to Rust.
4. **Android playback core tracer**
   - Move playback decision/request interpretation into Rust core only after
     the tolerance and adapter-reuse seams are ready. Keep Media3, media
     sessions, PiP, route UI, and player presentation Android-owned.
5. **Android browse core tracer**
   - Move only browse route interpretation that removes duplicated portable
     behavior. Keep Android product taxonomy and presentation state app-owned.
6. **Mobile SDK publishing / KMP / iOS posture**
   - Decide Maven publishing, Kotlin Multiplatform, iOS binding strategy, and
     multi-SDK runtime policy separately.
7. **Rust-owned networking**
   - Consider only in a later lane with explicit TLS, proxy, certificate,
     cleartext, retry, and platform-diagnostics decisions.

## Residual Risks

- Android ordinary app builds now include Rust/UniFFI/NDK prerequisites. This
  is intentional, but build speed, cacheability, ABI packaging, release
  stripping, and CI ergonomics need dedicated work.
- The connection tracer intentionally avoids strict-enum browse/playback DTO
  decode. Rust wire tolerance must be solved before those route bodies move
  behind UniFFI.
- `nako-client` and `nako-client-core` can drift until adapter reuse is
  implemented.
- Android connection categories and user messages remain app-owned. Each future
  route-family tracer must prove that core failures map cleanly without moving
  product taxonomy into Rust.
- SDK publishing and KMP remain undesigned. That is out of scope for this lane.

## Evidence Anchors

- `docs/workstreams/generated-sdk-runtime-ownership/EVIDENCE_AND_GATES.md`
- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `crates/nako-client-core/src/lib.rs`
- `crates/nako-client-uniffi/src/lib.rs`
- `crates/nako-uniffi-bindgen/src/main.rs`
- `apps/android/app/src/main/java/dev/nako/android/connection/RustConnectionCore.kt`
- `apps/android/app/src/main/java/dev/nako/android/connection/NakoConnectionClient.kt`
- `apps/android/app/build.gradle.kts`
