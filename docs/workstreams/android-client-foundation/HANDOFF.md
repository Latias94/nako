# Android Client Foundation Handoff

Status: Proposed
Last updated: 2026-05-17

## Current State

The design baseline is documented. No Android project files or shared mobile
Rust crates have been created yet.

Resolved decisions:

- Android is the first implementation target.
- Android-first is implementation order, not product strategy.
- The first product slice is playback-first with a minimal media-library browse
  loop.
- Android uses native playback through Media3 ExoPlayer.
- Shared Rust client core may own protocol, auth, DTO, playback-decision, and
  request-construction logic, but not player instances.
- iOS remains a peer future native client target under ADR 0026.

## Next Task

Start with `ACF-010`: create the Android scaffold under `apps/android`.

Before implementing, decide whether the first scaffold should be:

- one `:app` module only, optimized for fast iteration; or
- `:app` plus early `core/*` and `feature/*` modules, optimized for long-term
  separation.

Recommended first answer: start with one `:app` module plus clear packages.
Split Gradle modules after connection, browse, and playback boundaries become
real enough to justify the build overhead.

## Risks To Preserve

- Do not make Android-specific assumptions part of the Public Client API.
- Do not depend on AGPL server/internal crates from Android or shared client
  code.
- Do not create a Rust-owned player abstraction.
- Do not expand into server administration, metadata editing, addons, webhook,
  automation, or storage diagnostics in the first client slice.

## Validation Reminder

Use `EVIDENCE_AND_GATES.md` for candidate commands. Android commands become
authoritative only after the Gradle scaffold exists.
