# Android UniFFI Native Smoke — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. `UNS-010`, `UNS-020`, `UNS-030`, and `UNS-090` are
complete.

## Active Task

None.

## Guardrails

- Instrumentation smoke must exercise packaged Android `.so`, not host library
  override.
- Keep networking, server fixtures, profile/token persistence, UI, and Media3
  out of scope.
- Keep focused ABI selection for local validation.

## Recommended Next Step

Keep this smoke as a focused regression gate. If future Android UniFFI issues
appear, start a new lane for broader device/emulator coverage rather than
expanding this smoke into playback E2E.
