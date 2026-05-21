# Generated SDK Runtime Ownership — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

`SDKRT-010`, `SDKRT-020`, `SDKRT-030`, and `SDKRT-035` are complete. The prior
SDK lanes are closed:

- `android-generated-public-client-sdk` moved Android DTO and route mirrors to
  an OpenAPI-backed Kotlin/JVM SDK.
- `generated-sdk-forward-compat-tolerance` made generated public string values
  tolerant of unknown future wire values.

The remaining question has been frozen at the M0 level: Android still owns
execution, public error parsing, API-version header checks, redaction,
transport failure mapping, and product diagnostics today, but the durable target
is now an early shared Rust client core with app-supplied Android transport.

The owner has clarified that shared Rust client core / UniFFI complexity can be
pulled forward now if doing so prevents future architecture debt. Treat early
Rust core as a first-class candidate, not as an automatic follow-on.

## Active Task

`SDKRT-040` is ready.

Goal: wire the Android connection flow to the proven Rust core / UniFFI binding
boundary while keeping Android-owned transport, token storage, profile
persistence, product diagnostics, UI, and Media3 responsibilities unchanged.

## Decisions Already Inherited

- ADR 0025: Public Client API OpenAPI v1 is the SDK contract authority.
- ADR 0026: native shells with shared Rust client core are the long-term
  flagship direction.
- ADR 0031: generated client SDK work was sequenced before mobile Rust/UniFFI;
  ADR 0032 now supersedes its post-generated-SDK mobile Rust/UniFFI sequencing.
- ADR 0032: pull shared Rust client core forward behind app-supplied Android
  transport.
- Android UI, navigation, Media3, media sessions, token storage, product copy,
  and platform security policy stay app-owned.
- Generated DTO/request code must remain synchronized from `taru-api` if it is
  generator-owned.

## Frozen Decisions From SDKRT-010

- Select Option E: pull shared Rust client core forward now.
- Do not add a Kotlin SDK runtime layer unless Rust core is later rejected.
- Do not make the first tracer Rust-owned Android networking.
- Start with an FFI-safe no-socket Rust client core and app-supplied Android
  transport.
- Keep `taru-client` as the existing reqwest/async Rust adapter; define whether
  a new `taru-client-core` and `taru-client-uniffi` split is needed in
  `SDKRT-020`.
- Rust core should own protocol-level request construction, API-version
  observation, public error parsing, JSON decode classification, redaction
  primitives, and eventually playback decision/request interpretation.
- Android should own token vaults, profile persistence, cleartext/TLS policy,
  product failure categories, user copy, Compose/navigation, and Media3.

## Frozen Decisions From SDKRT-020

- Create ADR 0032:
  `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`.
- Mark ADR 0031 as superseded for post-generated-SDK mobile Rust/UniFFI
  sequencing.
- Introduce/define `taru-client-core` as the new permissive, no-socket,
  FFI-safe core.
- Keep `taru-client` as the reqwest/async adapter that should later reuse the
  core.
- Put UniFFI scaffolding in a thin `taru-client-uniffi` binding crate later,
  not in the core.
- First tracer: connection health plus authenticated library auth probe with
  Android-supplied transport.
- First tracer may skip library-list DTO decode to avoid strict Rust enum
  tolerance blockers.

## Open Decisions

- What Gradle/NDK/UniFFI build topology is acceptable for ordinary Android
  builds?
- Whether generated Kotlin binding sources should be committed now or produced
  by a documented Gradle/bindgen task in the Android build.

## Blockers

None for `SDKRT-040`; however, ordinary Android builds must make the new Rust /
UniFFI prerequisite explicit if native binding consumption is introduced.

## Recommended Next Step

Continue with `SDKRT-040`: inspect the generated UniFFI Kotlin shape, choose the
least surprising Android build integration, then migrate only the connection
probe flow. Do not broaden into playback, profile persistence, token vaults, UI,
or Rust-owned networking.

## Verification

Planning docs should pass:

```powershell
python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null
git diff --check
```
