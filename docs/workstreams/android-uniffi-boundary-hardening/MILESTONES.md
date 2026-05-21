# Android UniFFI Boundary Hardening — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

Exit criteria:

- The hardening target is explicit and grounded in ADR 0032.
- Non-goals prevent scope creep into Rust-owned Android networking,
  browse/catalog DTO migration, or Media3 ownership.
- Task ledger names bounded implementation slices.

Primary evidence:

- `docs/workstreams/android-uniffi-boundary-hardening/DESIGN.md`
- `docs/workstreams/android-uniffi-boundary-hardening/TODO.md`
- `docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json`

## M1 — Android Adapter Encapsulation

Exit criteria:

- `TaruConnectionClient` no longer imports or switches over generated UniFFI
  connection records/enums.
- Generated connection types are isolated to `RustConnectionCore.kt` and smoke
  tests.
- Connection JVM tests pass.

Primary gate:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
```

## M2 — Core Module Split

Exit criteria:

- `taru-client-core` has focused modules for request, response, redaction,
  connection, playback, and encoding.
- `lib.rs` stays as the public export surface.
- Current Rust callers and UniFFI tests pass unchanged.

Primary gates:

```powershell
cargo fmt --package taru-client-core --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
cargo nextest run -p taru-client --no-fail-fast
```

## M3 — Boundary Drift Guards

Exit criteria:

- A local guard rejects forbidden UniFFI runtime dependencies.
- Expected UniFFI surface is documented or snapshotted enough to catch drift.
- Guard is suitable for future CI/local validation.

Primary gates:

```powershell
# exact guard command recorded after implementation
cargo nextest run -p taru-client-uniffi --no-fail-fast
```

## M4 — Native Smoke Script

Exit criteria:

- A serial-aware, ABI-aware smoke script exists.
- README documents usage.
- The script is run on OPPO arm64 when available, or device unavailability is
  recorded with dry validation.

Primary gate:

```powershell
apps/android/scripts/Validate-UniFfiNativeSmoke.ps1 -Serial <device> -Abi arm64-v8a
```

## M5 — Closeout

Exit criteria:

- Gate set is recorded with fresh command evidence.
- Residual risks and follow-ons are explicit.
- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and
  `HANDOFF.md` agree on closed state.

Result: complete. See `CLOSEOUT.md`.
