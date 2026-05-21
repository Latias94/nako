# Android arm64 UniFFI Release Smoke — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. `A64-010`, `A64-020`, `A64-030`, and `A64-090` are
complete.

## Active Task

None.

## Guardrails

- Do not claim arm64 runtime execution unless `ro.product.cpu.abi` is arm64 and
  connected instrumentation passes.
- Packaging evidence should verify both Taru UniFFI and JNA dispatch libraries.
- Keep `-PtaruRustAndroidAbis=arm64-v8a` focused.

## Recommended Next Step

Run the existing connected UniFFI smoke on a real arm64 device when one is
available. This lane proves arm64 packaging, not arm64 runtime execution.
