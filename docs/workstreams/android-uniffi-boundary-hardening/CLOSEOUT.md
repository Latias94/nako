# Android UniFFI Boundary Hardening — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane hardened the existing ADR 0032 Rust core / Android app-supplied
transport seam without expanding scope into Rust-owned networking, browse DTO
migration, or Media3 ownership.

Completed slices:

1. Android adapter encapsulation: connection product logic now consumes
   Android-owned `ConnectionCoreOutcome`, `ConnectionCoreRequest`, and
   `ConnectionCoreSuccess` values instead of generated UniFFI probe types.
2. Core module locality: `taru-client-core` now has focused modules for ids,
   encoding, redaction, request, response, connection, and playback while
   keeping `lib.rs` as the stable public re-export surface.
3. Boundary drift guard: `scripts/guard-uniffi-boundary.ps1` checks that
   `taru-client-uniffi` direct dependencies remain allowlisted and that
   forbidden runtime/platform dependencies do not appear in its dependency tree.
4. Native smoke repeatability: `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`
   builds selected ABI APKs, installs them on a selected device, and runs the
   packaged UniFFI native smoke test.

## Final Verification

Fresh closeout gates run on 2026-05-21:

```powershell
cargo fmt --package taru-client-core --check
./scripts/guard-uniffi-boundary.ps1
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
./apps/android/scripts/Validate-UniFfiNativeSmoke.ps1 -Serial emulator-5554 -Abi x86_64
python -m json.tool docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json > $null
git diff --check
```

All gates passed. The OPPO arm64 device used in the earlier release-smoke lane
was not connected during this closeout; the new script intentionally failed
fast for that stale serial and then passed on the connected x86_64 emulator.
The earlier OPPO arm64 evidence remains recorded in
`docs/workstreams/android-arm64-uniffi-release-smoke/`.

## Residual Risks

- Browse/catalog/user-playback route construction still uses the generated
  Kotlin SDK in places. That remains an accepted follow-on; this lane only
  hardened the existing connection/playback UniFFI seam.
- The new native smoke script proves one selected ABI/device per run. A broader
  physical-device matrix is still a release-management choice, not a default
  local gate.
- The boundary guard is dependency-focused. It catches transport/platform crate
  creep, but it does not fully snapshot every generated Kotlin binding symbol.

## Recommended Follow-ons

1. Move browse/catalog route construction through `taru-client-core` when the
   product needs to reduce remaining Kotlin runtime-policy ownership.
2. Add the UniFFI boundary guard and native-smoke script to CI/release recipes
   once the desired device/ABI matrix is decided.
3. Rename playback `sessionProbeRequest` toward a more precise preparation or
   session-start concept before expanding playback target shapes further.
