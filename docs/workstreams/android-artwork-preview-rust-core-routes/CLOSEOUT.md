# Android Artwork And Preview Rust Core Routes — Closeout

Status: Closed
Closed on: 2026-05-22

## Outcome

This lane removed the remaining Android `src/main` generated SDK route
descriptor use that could be mistaken for runtime route ownership.

Completed slices:

1. `nako-client-core` now exposes selected artwork image request construction
   for `GET /images/{image_id}` with optional `width` and `height` variant query
   parameters.
2. `nako-client-uniffi` exposes a thin FFI-safe artwork image request builder
   over `nako-client-core`.
3. Android `PublicArtworkSource` now uses an injectable `ArtworkCore` seam backed
   by Rust/UniFFI descriptors.
4. Android selected artwork validation still rejects blank tokens, blank image
   ids, admin/absolute/query/fragment/traversal URLs, and stale DTO URLs whose
   route does not match the Rust-built canonical image route.
5. `NakoBrowseShellPreview` no longer imports generated SDK route descriptors or
   `PageQuery`; it uses preview-local route helpers.
6. The dead `PublicApiRequestDescriptors.kt` generated-descriptor adapter was
   deleted.
7. Android docs now state selected artwork route ownership and preview fixture
   policy.

## Final Verification

Fresh closeout gates run on 2026-05-22:

```powershell
cargo fmt --package nako-client-core --check
cargo nextest run -p nako-client-core --no-fail-fast
cargo fmt --package nako-client-uniffi --check
cargo nextest run -p nako-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.artwork.PublicArtworkTest --tests dev.nako.android.ui.artwork.ArtworkRequestResolverTest --no-daemon --rerun-tasks
apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon
if (rg -n "NakoPublicClientRequests|NakoRequestDescriptor|PublicApiRequestDescriptors|urlOn\\(" apps/android/app/src/main/java) { exit 1 } else { 'PASS: Android main has no generated SDK route descriptor use.' }
if (rg -n "NakoPublicClientRequests|PageQuery|pathAndQuery" apps/android/app/src/main/java/dev/nako/android/ui/browse/NakoBrowseShellPreview.kt) { exit 1 } else { 'PASS: browse preview has no generated SDK route matching.' }
python -m json.tool docs/workstreams/android-artwork-preview-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

All gates passed. Core ran 18 tests, UniFFI ran 6 tests, targeted Android
artwork/resolver JVM tests passed, Android debug Kotlin compile passed, route
owner scans passed, and the UniFFI boundary guard reported only
`nako-client-core` and `uniffi` as direct dependencies with no forbidden
runtime/platform dependency.

## Residual Risks

- Android still decodes Public Client API DTOs through the generated Kotlin SDK.
  That remains intentional: the generated SDK is the DTO/contract transition
  layer, not runtime route policy.
- Generated SDK route descriptors remain in Android tests that assert SDK
  contract inventory. That is acceptable because tests are not app runtime
  policy.
- Preview route helpers are intentionally local fixtures. They should not grow
  into a second production route-construction layer.

## Follow-ons

1. Add a CI route-owner scan if Android `src/main` must never regain generated
   SDK route descriptor imports.
2. Consider moving selected artwork variant selection policy to product UI once
   the app actually requests bounded artwork dimensions.
3. Continue migrating DTO decode to Rust only when Rust wire tolerance and
   cross-platform read-model reuse justify it.