# Android Rust Core Runtime Hardening — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane completed the four follow-ons from
`generated-sdk-runtime-ownership`:

1. Android Rust/UniFFI build ergonomics.
2. `nako-client` reuse of `nako-client-core`.
3. Rust public wire tolerance.
4. Android playback core tracer.

The resulting boundary is deliberately strict:

- `nako-client-core` owns portable request construction, response/version/error
  policy, redaction-safe previews, playback target selection, and playback
  route/query construction.
- `nako-client-uniffi` remains a binding adapter over `nako-client-core`.
- `nako-client` remains the Rust async/reqwest adapter and consumes shared core
  request/response policy instead of duplicating it.
- Android keeps app/runtime ownership: transport execution, profile/token
  state, diagnostics, DTO-to-product mapping, public session header handling,
  player launch, and Media3.

## Commits

- `5234feb build(android): split rust uniffi build artifacts`
- `c302436 refactor(client): reuse rust core request policy`
- `3914d82 feat(protocol): preserve unknown public wire values`
- `4fe7ad8 feat(android): route playback through rust core`

## Final Verification

Fresh closeout gates run on 2026-05-21:

```powershell
cargo fmt --all --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo nextest run -p nako-client-uniffi --no-fail-fast
cargo nextest run -p nako-client-protocol --no-fail-fast
cargo nextest run -p nako-client --no-fail-fast
cargo nextest run -p nako-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon
python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null
git diff --check
```

All gates passed.

## Residual Risks

- Android playback coverage is JVM-level. It proves request construction and
  UniFFI host loading, but not device/emulator playback launch with packaged
  native libraries.
- Browse/catalog/user-playback Android route construction still uses the
  generated Kotlin SDK in places. That is acceptable after this lane, but future
  high-value flows should move portable request/response rules into
  `nako-client-core` instead of adding new Kotlin-only policy.
- Rust tolerant public wire values now preserve unknown strings, but generated
  Android app models intentionally still collapse unknowns to product-safe
  `Unknown` values where raw strings are not yet needed by the UI.

## Recommended Follow-ons

1. Add a device/emulator native-library smoke lane for Android UniFFI loading
   and one playback launch path.
2. Move browse/catalog route construction through `nako-client-core` once the
   playback boundary has settled.
3. Review generated Kotlin SDK responsibilities and shrink it toward DTOs plus
   route inventory if Android continues to consume Rust core request builders.
