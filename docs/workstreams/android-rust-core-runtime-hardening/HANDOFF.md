# Android Rust Core Runtime Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open. `RCR-010`, `RCR-020`, `RCR-030`, `RCR-040`, and `RCR-050`
are complete.

## Active Task

`RCR-090`: close the lane with fresh evidence, residual risks, and follow-ons.

## Guardrails

- Keep Rust-owned Android networking out of scope.
- Keep Android profile/token/security/UI/Media3 ownership in Android.
- Keep `taru-client-uniffi` as a binding layer, not a runtime policy crate.
- Preserve raw unknown public wire strings in Rust before moving more Android
  decode behind UniFFI.

## Recommended Next Step

Run the closeout gates, write `CLOSEOUT.md`, and make `TODO.md`,
`MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and `WORKSTREAM.json`
agree on the final lane state.
