# Android Rust Core Runtime Hardening — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Lane Open

Status: Complete on 2026-05-21.

Exit when the four follow-ons are turned into a single serialized task ledger
with explicit guardrails and gates.

## M1 — Android Rust/UniFFI Build Ergonomics

Status: Complete on 2026-05-21.

Exit when:

- host binding generation and Android ABI packaging are separated;
- JVM unit tests need only generated bindings plus host native library;
- APK assembly builds packageable Android ABI libraries;
- local ABI selection is documented.

Result: Host library, generated Kotlin bindings, debug/release Android ABI
libraries, and variant JNI packaging are separate Gradle tasks. JVM unit tests
build only the host library and generated bindings. Debug assembly can select
an ABI set with `-PtaruRustAndroidAbis=...`.

## M2 — Rust Client Adapter Reuse

Exit when:

- `taru-client-core` exposes reusable request/response policy;
- `taru-client` consumes that policy for route request specs, bearer headers,
  status/API-version checks, and public error envelopes;
- existing `taru-client` route tests pass.

## M3 — Rust Public Wire Tolerance

Exit when:

- public Rust string values that can grow decode unknown strings into preserved
  values;
- known-value ergonomics remain clear;
- protocol, API generator, and client tests pass.

## M4 — Android Playback Core Tracer

Exit when:

- playback decision request construction uses Rust core through UniFFI;
- playback target interpretation uses Rust core through UniFFI;
- Android still owns Media3, transport execution, session preflight execution,
  diagnostics, profile/token storage, and product copy;
- focused playback tests and Android assemble pass.

## M5 — Closeout

Exit when:

- `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, and
  `WORKSTREAM.json` agree;
- fresh final evidence is recorded;
- closeout notes identify residual risks and next lanes.
