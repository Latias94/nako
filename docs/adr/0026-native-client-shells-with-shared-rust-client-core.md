# 0026: Use Native Client Shells With a Shared Rust Client Core

Status: accepted

## Context

Taru's server-side Public Client API, protocol DTOs, OpenAPI contract,
TypeScript SDK, Rust SDK, and Rust client CLI already make client applications
protocol-first rather than framework-first. `CONTEXT.md` also treats Flutter,
web, native apps, and future clients as **Client Applications** that consume
the **Public Client API**, not as assumptions that should shape server
internals.

The next client architecture decision is not only about UI productivity. A
media-server client must eventually own high-quality playback behavior:

- startup latency, seeking, buffering, and preloading;
- hardware decode behavior and platform codec support;
- subtitles, audio tracks, chapters, and playback session control;
- picture-in-picture, media sessions, route selection, remote controls,
  background behavior, and TV-style focus/navigation;
- direct, remux, and HLS playback surfaces exposed by Taru;
- offline/download behavior and local cache coordination;
- platform-specific error recovery and diagnostics.

Cross-platform UI frameworks can produce useful client applications, but the
flagship playback experience depends on the platform media stack. iOS playback
depth comes from AVFoundation/AVKit. Android playback depth comes from Media3
ExoPlayer. Taru should avoid putting a Flutter, KMP, or web assumption into
the Public Client API or playback runtime contract.

`repo-ref/litter` is useful reference architecture for this direction: it uses
native iOS and Android shells with shared Rust logic through UniFFI, keeping
platform code focused on UI and platform APIs while shared state and protocol
logic live in Rust. Taru should copy the architectural lesson, not the source.

## Decision

Taru's flagship media playback clients should use native platform shells with
a shared Rust client core.

- iOS clients should use Swift/SwiftUI with AVFoundation/AVKit for playback.
- Android clients should use Kotlin/Compose with Media3 ExoPlayer for
  playback.
- Future tvOS, Android TV, desktop, or living-room clients should follow the
  same principle: use the platform-native media stack for primary playback.
- The shared Rust client core should reuse `taru-client` and
  `taru-client-protocol` where practical, exposed to mobile platforms through
  a narrow FFI boundary such as UniFFI.
- Shared Rust code may own Public Client API calls, bearer-token handling,
  API-version checks, public error-envelope handling, DTO hydration, browse and
  search query state, playback decision interpretation, streaming request
  construction, and portable cache/download coordination metadata.
- Native platform code owns actual player instances, media-session
  integration, subtitle/audio-track presentation, PiP, cast/route UI, remote
  controls, background behavior, platform permissions, and platform-specific
  playback diagnostics.
- Flutter, KMP, and web clients remain valid **Client Applications**, but they
  are not the flagship playback architecture and must not drive server API
  design.
- Web should be treated first as an admin, setup, remote-control, and light
  browsing surface unless a separate ADR accepts a web-first playback target.

The server-side **Playback Runtime** remains Taru-owned. Native clients consume
public playback decisions, direct/remux/HLS URLs, session inspection, and
session cancellation through the Public Client API. Clients must not bypass
**Playback Source Selection**, **Library Access**, or server-owned playback
resource policy by inferring local storage paths or internal server state.

## Consequences

- Taru optimizes the long-term playback user experience over lowest common
  denominator cross-platform UI reuse.
- Client implementation cost is higher because iOS and Android UI/player
  shells are separate applications.
- Public DTOs, SDKs, route inventory, auth behavior, and playback request
  builders become more valuable because they prevent duplicated protocol code
  across native clients.
- The Rust shared core must stay small enough to be a client boundary, not a
  hidden portable application framework.
- Native player behavior can be tuned per platform without forcing server API
  churn or inventing artificial cross-platform player abstractions.
- Flutter or KMP prototypes can still exist, but they should consume the same
  Public Client API and should not become the contract authority.
- Future client workstreams should separate shared client-core work from
  platform shell work so SDK, FFI, iOS, Android, web, and TV validation can
  evolve independently.

## Alternatives Considered

- Flutter-first client. Rejected as the flagship direction because it improves
  shared UI velocity but eventually pushes deep playback, media-session,
  PiP/background, TV, subtitle/audio-track, and platform diagnostics work back
  into native plugin code.
- KMP/Compose Multiplatform-first client. Rejected as the default direction
  because it fits Android and shared Kotlin logic well, but iOS playback still
  needs AVFoundation ownership and the project already has a Rust public client
  SDK/protocol boundary.
- Web-first client. Rejected for flagship playback because browser playback,
  background behavior, route control, TV ergonomics, and platform integration
  are not enough for Taru's intended deep media-client experience. Web remains
  useful for administration, setup, remote control, and lightweight access.
- Fully native clients with no shared client core. Rejected because it would
  duplicate auth, API-version checks, error envelope parsing, playback request
  construction, and DTO hydration across platforms.
- Shared Rust core owning the player abstraction. Rejected because playback
  runtime should be platform-native on clients and Taru-owned on the server.
  Rust can construct requests and interpret decisions, but the native shell
  should own the actual playback engine.

## Related Workstreams

- `docs/workstreams/public-client-api/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/openapi-client-contract/`
- `docs/workstreams/client-sdk-contract/`
- `docs/workstreams/rust-client-sdk/`
- `docs/workstreams/client-cli/`
- Future `clients` workstream for concrete iOS, Android, web, TV, FFI, and
  shared client-core planning.
