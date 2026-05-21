# Generated SDK Runtime Ownership

Status: Active
Last updated: 2026-05-21

## Why This Lane Exists

The generated Kotlin/JVM SDK now owns Public Client API DTO mirrors, constants,
route path/query construction, and tolerant public string-value wrappers.
Android still owns the runtime behaviors around those generated request
descriptors. This lane exists to decide whether that split is still the cleanest
boundary, or whether protocol-level runtime semantics should move into a
portable SDK/runtime seam before more clients duplicate them.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- Existing docs:
  - `docs/workstreams/android-generated-public-client-sdk/CLOSEOUT.md`
  - `docs/workstreams/generated-sdk-forward-compat-tolerance/CLOSEOUT.md`
- Current code evidence:
  - `crates/taru-api/src/sdk.rs`
  - `sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
  - `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientApiExecutor.kt`
  - `apps/android/app/src/main/java/dev/taru/android/connection/TaruHttpTransport.kt`
  - `apps/android/app/src/main/java/dev/taru/android/connection/SensitiveText.kt`
  - `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`
  - `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`

## Problem

The current generated SDK boundary is clean for DTO and route drift, but the
runtime boundary is ambiguous:

- `sdk/kotlin` owns `TaruRequestDescriptor`, API constants, route constructors,
  generated DTOs, and tolerant public string wrappers.
- Android owns `PublicClientApiExecutor`, `PublicApiAuth`,
  `PublicApiResult`, `PublicApiFailure`, `TaruHttpTransport`,
  `SafeRequestPreview`, public error parsing, API-version header checking,
  JSON decode failure mapping, transport failure mapping, and redaction.
- Android also owns product policy: cleartext restrictions, TLS copy, profile
  persistence, token vaults, connection/playback failure categories, UI copy,
  and Media3 playback.

Some Android-owned runtime behavior is protocol-level and likely reusable.
Some is product or platform policy and must not move. Without a frozen matrix,
future work can either under-share protocol behavior and duplicate bugs, or
over-share app behavior and turn the SDK into a hidden Android product layer.

The product direction now explicitly prefers surfacing shared Rust client-core
and UniFFI complexity early if that prevents future architecture debt. That
means ADR 0031's sequencing rule must be re-evaluated rather than assumed as a
hard blocker.

## Current Boundary Snapshot

| Responsibility | Current owner | Candidate durable owner |
| --- | --- | --- |
| Public DTOs and constants | Generated Kotlin SDK | Generated Kotlin SDK |
| Route path/query construction | Generated Kotlin SDK | Generated Kotlin SDK |
| Request descriptor shape | Generated Kotlin SDK | Generated Kotlin SDK |
| HTTP transport implementation | Android app | Platform app or supplied transport adapter |
| Bearer token storage | Android app | Platform app |
| Bearer header injection | Android executor | SDK runtime or app adapter; needs decision |
| Public error-envelope DTO decoding | Android executor | SDK runtime, using generated `ErrorResponse` |
| API-version header check | Android executor | SDK runtime unless product-specific policy differs |
| JSON decode failure classification | Android executor | SDK runtime with app mapping layer |
| Token-safe request preview | Android executor | Shared redaction primitive plus app-owned diagnostics |
| Cleartext/TLS policy | Android app | Platform app |
| Connection/playback failure categories | Android app | Android app |
| UI copy and accessibility semantics | Android app | Android app |
| Media3 playback runtime | Android app | Android app |
| FFI-safe request/response DTOs | Not present | Shared Rust client core / UniFFI if pulled forward |
| Rust protocol enum tolerance | Partially strict in `taru-client-protocol` | Must be solved before Rust core owns mobile decode |

## Target State

This lane should close with one of two honest outcomes:

1. **No runtime move yet.** The workstream records that Android-owned execution
   remains the right boundary for the current single-shell stage, lists the
   trigger for revisiting it, and closes without code movement.
2. **Narrow runtime move.** A small SDK/runtime seam owns protocol-level
   behavior only, while Android supplies transport/platform policy and maps
   runtime failures into product diagnostics. The first tracer proves one
   low-risk flow without changing UI, Media3, or token persistence ownership.
3. **Early Rust core target state.** A shared Rust client core becomes the
   preferred runtime owner now. The lane records the ADR impact, crate/FFI
   topology, app-supplied versus Rust-owned transport decision, and the first
   minimal Android tracer before implementation starts.

Either outcome must leave an explicit ownership matrix and split SDK
publishing, KMP, Rust/UniFFI, and wider multi-SDK runtime work into separate
lanes.

## In Scope

- Inventory of Android `PublicClientApiExecutor` responsibilities.
- Ownership matrix for SDK, SDK runtime, Android app, and future shared Rust
  core boundaries.
- Decision on whether generated SDK output should include runtime code, whether
  runtime should be hand-written beside generated DTOs, or whether it should
  stay app-owned.
- Decision on whether ADR 0031 is sufficient, amended, or superseded because
  shared Rust client core is being pulled forward.
- Target-state evaluation for shared Rust client core / UniFFI, including
  crate split, FFI-safe data shapes, runtime ownership, and Android build
  topology.
- If accepted after planning, a smallest tracer around one connection or simple
  JSON route path.

## Out Of Scope

- Publishing `sdk/kotlin` as an external Maven artifact.
- Kotlin Multiplatform restructuring.
- Full Android migration to Rust/UniFFI before an accepted target state.
- iOS, Swift, Dart, or TypeScript runtime implementation.
- Rust-owned mobile networking/TLS/proxy behavior unless separately accepted;
  an app-supplied Android transport remains a valid first tracer.
- Server route semantics, OpenAPI v2, Admin API, or authentication model
  changes.
- Android Compose state, navigation, Media3, media sessions, playback controls,
  or UI diagnostics copy changes.
- Token vault, profile persistence, cleartext security policy, certificate
  trust, or Android permission changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Android is still the only production native shell in this repo. | Medium | ADR 0031 and Android workstream closeouts | Rust/UniFFI or KMP may need earlier design. |
| Generated SDK DTO/request adoption is complete for migrated Public Client API route families. | High | `android-generated-public-client-sdk/CLOSEOUT.md` | Runtime ownership would be premature if route drift remains. |
| Generated Kotlin public string values now tolerate unknown additive wire values. | High | `generated-sdk-forward-compat-tolerance/CLOSEOUT.md` | Runtime decode policy would need to solve enum tolerance first. |
| `PublicClientApiExecutor` mixes protocol-level and app/product policy. | High | `PublicClientApiExecutor.kt`, `TaruConnectionClient.kt`, `TaruPlaybackClient.kt` | A blind move would either duplicate code or leak Android policy into SDK. |
| ADR 0031's original sequencing may no longer match the owner's risk preference. | Medium | User direction on 2026-05-21 | This lane may need a new ADR before implementation. |
| Existing `taru-client` is useful but not mobile-FFI-ready as-is. | High | `crates/taru-client/src/lib.rs` uses `reqwest`, `Url`, `HeaderMap`, async transport, and strict protocol DTO enums. | A clean Rust core likely needs a smaller FFI-safe boundary or crate split. |

## Architecture Direction

The provisional bias changes after the product-owner direction: do not invest
in a Kotlin runtime until `SDKRT-010` proves it is better than pulling the Rust
core forward.

The cleanest early Rust path is not "make Android depend on full `reqwest`
networking tomorrow." The preferred first tracer should be an FFI-safe Rust
client core that can own protocol-level request construction and response
interpretation while Android still supplies platform transport and security
policy. That flushes out the hard parts early:

- crate split around `taru-client` versus a new `taru-client-core`/
  `taru-mobile-core`;
- FFI-safe request/response and error DTOs;
- tolerant Rust-side public wire values;
- API-version and public error-envelope behavior;
- redaction-safe diagnostics;
- Gradle/NDK/UniFFI build topology.

A Kotlin runtime remains a valid fallback if Rust core proves too expensive for
the current Android-only stage.

A good runtime/core shape would:

- accept generated `TaruRequestDescriptor` values;
- expose a small transport abstraction or adapter contract;
- use generated DTO serializers and generated `ErrorResponse`;
- classify protocol-level runtime failures without product copy;
- preserve API-version header observation and safe redaction;
- let Android map runtime failures into connection/playback diagnostics.

The runtime must not own Android profile selection, token vaults, cleartext
policy, TLS trust copy, user-facing messages, Compose state, navigation, Media3,
or playback session presentation.

`SDKRT-010` is allowed to reject early Rust core if the inventory shows that
the first tracer would mostly be FFI packaging with little architecture value.

## Candidate Options

### Option A — Keep Android-Owned Executor

Lowest implementation risk. Best if Taru remains Android-only for now and the
runtime is mostly product policy. The cost is future duplication for other
clients and continued Android-only ownership of protocol error/version rules.

### Option B — Add Platform-Neutral Runtime To `sdk/kotlin`

Move protocol-level execution semantics into the Kotlin SDK module. Android
supplies transport and maps runtime failures. This reduces duplicate client
logic, but it makes `sdk/kotlin` more than generated DTOs and may create public
API compatibility pressure before publishing policy exists.

### Option C — Split Generated DTOs And Hand-Written Runtime Package

Keep generated DTO/request output mechanically synchronized while adding a
clearly non-generated runtime package or module. This is the cleanest boundary
if runtime code moves now, because it separates generated contract artifacts
from reusable hand-written execution semantics.

### Option D — Defer To Rust/UniFFI

Keeps ADR 0031 unchanged. This is still safe, but it conflicts with the current
owner preference to expose integration complexity early when it prevents future
debt.

### Option E — Pull Shared Rust Client Core Forward

Create or refactor toward an FFI-safe Rust client core now. Start with
app-supplied Android transport, not Rust-owned Android networking, unless the
ADR explicitly chooses otherwise. Rust owns protocol-level request construction,
response interpretation, public error/version policy, playback decision
interpretation, and redaction primitives. Android owns token storage,
cleartext/TLS policy, UI copy, Compose, navigation, Media3, and product
diagnostic categories.

This option best matches the updated owner preference, but it requires an ADR
amendment/supersession because ADR 0031 previously sequenced Rust/UniFFI after
generated SDK adoption and stronger cross-platform triggers.

## Closeout Condition

This lane can close when:

- the ownership matrix is frozen;
- ADR 0031 impact is resolved;
- either no runtime move is explicitly accepted, or the first narrow tracer is
  implemented and tested;
- Android app policy remains outside SDK/runtime code;
- evidence gates pass;
- follow-ons such as publishing, KMP, Rust/UniFFI, and multi-SDK runtime
  tolerance are split or explicitly deferred.
