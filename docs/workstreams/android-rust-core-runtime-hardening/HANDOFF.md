# Android Rust Core Runtime Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open. `RCR-010`, `RCR-020`, `RCR-030`, and `RCR-040` are complete.

## Active Task

`RCR-050`: use the Rust core / UniFFI boundary for playback decision request
construction and playback target interpretation while keeping Android-owned
transport, diagnostics, and Media3.

## Guardrails

- Keep Rust-owned Android networking out of scope.
- Keep Android profile/token/security/UI/Media3 ownership in Android.
- Keep `taru-client-uniffi` as a binding layer, not a runtime policy crate.
- Preserve raw unknown public wire strings in Rust before moving more Android
  decode behind UniFFI.

## Recommended Next Step

Implement `RCR-050` across `taru-client-core`, `taru-client-uniffi`, and Android
playback. Keep Android DTO mapping and product diagnostics app-owned unless the
task explicitly moves a portable rule into Rust.
