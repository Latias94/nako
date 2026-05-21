# Android Browse/Catalog Rust Core Routes — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane moved Android browse/catalog runtime route construction to the shared
Rust client core while preserving the ADR 0032 boundary: Android still owns
transport, DTO decoding, diagnostics, product mapping, UI, profile/token state,
and Media3.

Completed slices:

1. `taru-client-core` now exposes explicit browse/catalog request builders for
   libraries, library sources, items, item images, people, person items, genres,
   genre items, tags, tag items, and search.
2. `taru-client-uniffi` now exposes FFI-safe browse builder records/functions
   as a thin adapter over `taru-client-core`.
3. Android now uses `BrowseCore`/`RustBrowseCore` to convert product browse
   inputs into Rust-built request descriptors before executing Android-owned
   transport.
4. `TaruBrowseClient` no longer calls generated Kotlin SDK route descriptors for
   migrated runtime browse routes. The generated SDK remains in use as the
   DTO/contract transition layer.
5. Android docs now state the route ownership split explicitly.

## Final Verification

Fresh closeout gates run on 2026-05-21:

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo fmt --package taru-client-uniffi --check
cargo nextest run -p taru-client-uniffi --no-fail-fast
./scripts/guard-uniffi-boundary.ps1
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon --rerun-tasks
if (rg -n "TaruPublicClientRequests|pathAndQuery" apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt apps/android/app/src/main/java/dev/taru/android/browse/RustBrowseCore.kt) { exit 1 } else { 'PASS: TaruBrowseClient/RustBrowseCore have no generated SDK route descriptor calls.' }
python -m json.tool docs/workstreams/android-browse-catalog-rust-core-routes/WORKSTREAM.json > $null
git diff --check
```

All gates passed. Core ran 13 tests, UniFFI ran 3 tests, and Android browse ran
29 JVM tests with `--rerun-tasks`. The UniFFI boundary guard reported only
`taru-client-core` and `uniffi` as direct dependencies and no forbidden
runtime/platform dependency.

## Residual Risks

- Browse DTO decoding is still Kotlin-side through the generated public-client
  SDK. That is intentional for this lane, but a future lane may move DTO decode
  or selected parsers into Rust if cross-platform cache/offline semantics need
  a single owner.
- The route builders currently mirror the existing Android browse route family;
  newly added Public Client API browse routes need new core + UniFFI builders,
  not ad-hoc Kotlin route construction.
- Android browse JVM tests prove route construction, DTO decode, diagnostics,
  and error mapping. Device smoke remains covered by broader Android validation
  lanes and was not expanded here.
- Rust core encodes spaces as `%20`; old Kotlin route descriptors encoded search
  spaces as `+`. Both are valid query encodings, but snapshot expectations now
  follow the Rust core route owner.

## Recommended Follow-ons

1. Move remaining user-playback route construction through Rust core once the
   browse migration has settled.
2. Decide whether DTO decode should remain Kotlin SDK-owned or move selected
   read models into Rust for future offline/cache use cases.
3. Add the UniFFI boundary guard and targeted Android browse JVM gate to CI if
   this seam becomes a release-blocking invariant.
4. Keep generated SDK request descriptors available for contract tests and API
   inventory, but avoid reintroducing them into Android runtime route policy.
