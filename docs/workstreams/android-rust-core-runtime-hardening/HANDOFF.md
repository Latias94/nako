# Android Rust Core Runtime Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open. `RCR-010` and `RCR-020` are complete.

## Active Task

`RCR-030`: move reusable request/response policy into `taru-client-core` and
make `taru-client` consume that policy as a reqwest/async adapter.

## Guardrails

- Keep Rust-owned Android networking out of scope.
- Keep Android profile/token/security/UI/Media3 ownership in Android.
- Keep `taru-client-uniffi` as a binding layer, not a runtime policy crate.
- Preserve raw unknown public wire strings in Rust before moving more Android
  decode behind UniFFI.

## Recommended Next Step

Implement `RCR-030` in `crates/taru-client-core` and `crates/taru-client`, then
run the focused Rust gates.
