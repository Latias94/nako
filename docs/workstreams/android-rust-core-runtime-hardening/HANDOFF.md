# Android Rust Core Runtime Hardening — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. `RCR-010`, `RCR-020`, `RCR-030`, `RCR-040`, `RCR-050`, and
`RCR-090` are complete.

## Active Task

None.

## Guardrails

- Keep Rust-owned Android networking out of scope.
- Keep Android profile/token/security/UI/Media3 ownership in Android.
- Keep `nako-client-uniffi` as a binding layer, not a runtime policy crate.
- Preserve raw unknown public wire strings in Rust before moving more Android
  decode behind UniFFI.

## Recommended Next Step

Start a new workstream for one of the closeout follow-ons if the product needs
more Android routes behind UniFFI, device/emulator native-library smoke tests,
or Kotlin generated SDK shrinkage.
