# Android Rust Core Runtime Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open. `RCR-010`, `RCR-020`, and `RCR-030` are complete.

## Active Task

`RCR-040`: make Rust public string-value DTOs preserve unknown additive wire
values instead of failing deserialization.

## Guardrails

- Keep Rust-owned Android networking out of scope.
- Keep Android profile/token/security/UI/Media3 ownership in Android.
- Keep `taru-client-uniffi` as a binding layer, not a runtime policy crate.
- Preserve raw unknown public wire strings in Rust before moving more Android
  decode behind UniFFI.

## Recommended Next Step

Implement `RCR-040` in `crates/taru-client-protocol` first, then update any
Rust client/API generator fallout and run the focused protocol/API/client
gates.
