# Generated SDK Runtime Ownership — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

`SDKRT-010` is complete. The prior SDK lanes are closed:

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

`SDKRT-020` is ready.

Goal: create or supersede the ADR for pulling shared Rust client core forward,
then define the smallest FFI-safe core API, crate topology, Android build
topology, and first connection-flow tracer.

## Decisions Already Inherited

- ADR 0025: Public Client API OpenAPI v1 is the SDK contract authority.
- ADR 0026: native shells with shared Rust client core are the long-term
  flagship direction.
- ADR 0031: generated client SDK work was sequenced before mobile Rust/UniFFI;
  `SDKRT-010` decided this sequencing should now be amended or superseded.
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

## Open Decisions

- Should `SDKRT-020` create ADR 0032 that supersedes ADR 0031's mobile-FFI
  sequencing, or amend ADR 0031 directly?
- Should the FFI-safe core crate be `taru-client-core`, and should UniFFI live
  in a separate `taru-client-uniffi` crate?
- What exact FFI-safe request/response DTOs should cross the boundary?
- How will Rust-side public wire values preserve unknown additive strings so
  Android does not regress from the Kotlin generated SDK tolerance lane?
- What Gradle/NDK/UniFFI build topology is acceptable for ordinary Android
  builds?
- Does the first tracer cover only `GET /health`, or the full connection
  health plus authenticated `GET /libraries?limit=1&offset=0` probe?

## Blockers

None. Do not implement runtime movement until `SDKRT-020` records the ADR and
FFI-safe core API shape.

## Recommended Next Step

Set a Codex goal for `SDKRT-020` only. That task should create/update ADR
authority and define the core/FFI/API target state. Parallel workers are not
useful until the ADR and crate topology are frozen.

## Verification

Planning docs should pass:

```powershell
python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null
git diff --check
```
