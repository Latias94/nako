# Android UniFFI Boundary Hardening — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. UBF-010, UBF-020, UBF-030, UBF-040, UBF-050, and UBF-090
are complete.

## Active Task

None.

## Decisions Since Last Update

- Treat the current ADR 0032 Rust core / app-supplied transport boundary as
  correct.
- Harden the seam before moving browse/catalog route construction through Rust
  core.
- Keep native smoke script opt-in; ordinary JVM validation must not require a
  connected device.
- `TaruConnectionClient` should depend on Android-owned `ConnectionCoreOutcome`
  values, not generated UniFFI outcome/request records.
- `taru-client-core` keeps a stable public API through `lib.rs`; implementation
  locality now lives in focused modules.
- `scripts/guard-uniffi-boundary.ps1` protects `taru-client-uniffi` from direct
  dependency drift and forbidden runtime/platform transitive dependencies.
- Native smoke is now scriptable with `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`.
  OPPO was disconnected during UBF-050, so the fresh runtime script proof used
  `emulator-5554` with `-Abi x86_64`; previous OPPO arm64 proof remains in
  `android-arm64-uniffi-release-smoke`.

## Blockers

- None.

## Next Recommended Action

Start a new workstream for browse/catalog route construction through
`taru-client-core`, or wire the boundary guard/native smoke script into CI or a
release recipe if release automation is the priority.
