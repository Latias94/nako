# 0031: Sequence Generated Client SDK Before Mobile Rust FFI

## Status

Superseded by [0032: Pull Shared Rust Client Core Forward Behind
App-Supplied Transport](0032-shared-rust-client-core-app-supplied-transport.md)
for post-generated-SDK mobile Rust/UniFFI sequencing.

This ADR remains the historical record for why Android generated SDK adoption
was sequenced before mobile Rust/UniFFI.

## Context

ADR 0026 accepts native client shells with a shared Rust client core as Taru's
long-term flagship client direction. Android should use Kotlin, Compose, and
Media3 ExoPlayer for the platform shell. iOS should use Swift, SwiftUI, and
AVFoundation/AVKit. A shared Rust core may later reuse `taru-client` and
`taru-client-protocol` through a narrow FFI boundary such as UniFFI.

The Android foundation workstream closed with a different near-term shape:
the app still uses direct Kotlin clients, but the generic Public Client API
policy now sits behind a deep Kotlin module:

- API-version checks;
- public error-envelope parsing;
- JSON decode failure mapping;
- transport failure mapping;
- token-safe request previewing;
- bearer redaction;
- path/query helper policy.

The remaining Android duplication risk is mostly handwritten public DTO and
route drift. ADR 0025 already makes the Public Client API OpenAPI v1 artifact
the future contract authority for generated client SDKs. Moving Android to
Rust/UniFFI before that contract stabilizes would mix three concerns at once:

1. OpenAPI contract generation;
2. Android route/client replacement;
3. mobile FFI packaging, ABI, async, and diagnostics policy.

That sequencing would raise Android build complexity before there is a second
production native shell, offline/download cache, or cross-platform client-core
logic that justifies the cost.

## Decision

Taru will sequence generated client SDK work before introducing a mobile
Rust/UniFFI client core into Android.

For the current Android foundation:

- Android must not depend on Rust/UniFFI compilation for ordinary app builds.
- Android continues to use Kotlin, Compose, and Media3 for UI, navigation,
  state presentation, and playback.
- The current Kotlin `PublicClientApiExecutor` remains the local seam for
  Public Client API execution until an OpenAPI-backed Kotlin SDK replaces the
  handwritten route clients.
- The next client-contract step should be an OpenAPI-backed generated Kotlin
  SDK after ADR 0025's OpenAPI v1 artifact is stable enough to be the contract
  authority.

The generated Kotlin SDK should own:

- public DTO mirrors;
- route path/query construction;
- request and response serialization;
- public error-envelope handling;
- API-version header behavior;
- safe generated client tests that reject admin/internal route leakage.

The generated Kotlin SDK should not own:

- Compose UI state;
- Android navigation;
- Media3 player instances;
- Android media sessions;
- platform permissions;
- Android-specific diagnostics presentation;
- product copy or accessibility semantics.

A shared Rust/UniFFI client core remains the long-term direction from ADR
0026, but it should start only after a separate target-state workstream defines
the narrow portable interface and validation matrix.

Good triggers for starting that Rust/UniFFI workstream are:

- a production iOS shell starts and would otherwise duplicate Android protocol
  and playback-decision logic;
- downloads/offline/cache coordination becomes a cross-platform feature;
- Android TV or another native shell needs the same non-UI client state;
- generated SDK adoption still leaves substantial duplicated portable logic
  across Kotlin and Swift;
- playback decision interpretation, request descriptor construction, or
  diagnostics redaction become complex enough to require one portable
  implementation.

When introduced, the Rust client core may own:

- Public Client API calls built on `taru-client` and `taru-client-protocol`;
- bearer-token handling and token-safe request descriptors;
- API-version checks;
- public error-envelope handling;
- DTO hydration and client-facing enum mapping;
- browse/search query state;
- playback decision interpretation;
- streaming request construction;
- portable cache/download coordination metadata;
- token-safe diagnostics redaction.

The Rust client core must not own:

- Android Compose UI;
- Android navigation or saved route state;
- Media3 player instances;
- iOS AVFoundation player instances;
- media-session integration;
- subtitle/audio-track UI;
- PiP, cast, remote controls, or TV focus behavior;
- platform permissions;
- platform-specific playback diagnostics presentation.

## Consequences

- Android app builds stay simpler while the product and Public Client API are
  still moving quickly.
- The first concrete drift-reduction step targets handwritten Kotlin DTOs and
  route clients through OpenAPI-backed generation instead of mobile FFI.
- ADR 0026 remains valid as the long-term direction; this ADR constrains the
  order, not the destination.
- Future Rust/UniFFI work has a clearer acceptance bar: it must provide a
  narrow portable interface with at least two real adapters or a
  cross-platform feature such as offline/download/cache.
- Android playback depth remains native because Media3 stays in Kotlin.
- iOS playback depth remains native because AVFoundation/AVKit stay in Swift.
- The shared Rust core is prevented from becoming a hidden portable
  application framework.
- Build, packaging, ABI, crash-reporting, and async bridge complexity are
  deferred until there is enough cross-platform leverage to justify them.

## Alternatives Considered

- Introduce Rust/UniFFI into Android immediately. Rejected because Android is
  currently the only production native shell in this repo, and the primary
  remaining duplication is DTO/route drift that OpenAPI-backed generation can
  address with less build complexity.
- Keep only handwritten Kotlin clients indefinitely. Rejected because public
  DTO and route drift will increase as the Public Client API grows, and ADR
  0025 already accepts generated OpenAPI contracts as the compatibility
  mechanism.
- Replace Android route clients directly with the Rust SDK now. Rejected
  because it couples Android app iteration to Rust mobile packaging before a
  mobile FFI target-state document exists.
- Put Media3 playback behind a Rust-owned abstraction. Rejected by ADR 0026:
  clients should use platform-native media stacks, while Rust may interpret
  playback decisions and construct safe requests.
- Use Kotlin Multiplatform as the shared core instead of Rust. Rejected as the
  default sequencing because Taru already has Rust protocol/client crates and
  iOS playback still needs Swift/AVFoundation ownership.

## Related Workstreams

- `docs/workstreams/android-fearless-client-refactor/`
- Future `docs/workstreams/android-generated-public-client-sdk/`
- Future `docs/workstreams/shared-rust-client-core-uniffi/`
- Future `docs/workstreams/android-downloads-offline/`
- Future `docs/workstreams/android-tv-shell/`
- [0025: Generate Public Client OpenAPI From Protocol-Owned Wire Types](0025-openapi-public-client-sdk-contract.md)
- [0026: Use Native Client Shells With a Shared Rust Client Core](0026-native-client-shells-with-shared-rust-client-core.md)
