# 0032: Pull Shared Rust Client Core Forward Behind App-Supplied Transport

## Status

Accepted.

Supersedes the post-generated-SDK mobile Rust/UniFFI sequencing rule in ADR
0031. ADR 0031 remains useful historical context for why generated SDK adoption
was done before mobile Rust work.

## Context

ADR 0026 accepts native platform shells with a shared Rust client core as
Taru's long-term flagship client direction. ADR 0031 then sequenced generated
client SDK work before mobile Rust/UniFFI because Android still had handwritten
DTO and route drift, and moving to Rust first would have mixed OpenAPI
generation, route replacement, mobile packaging, ABI, async, and diagnostics
policy at the same time.

That first sequencing goal has now served its purpose:

- Android consumes the generated Kotlin/JVM Public Client SDK for public DTOs,
  constants, and route descriptors across the migrated route families.
- Generated Kotlin public string values now preserve unknown future wire values.
- Android still owns a reusable `PublicClientApiExecutor` seam for HTTP
  execution, bearer handling, public error-envelope parsing, API-version
  checks, decode failures, transport failure mapping, and redaction.
- `taru-client` already owns Rust request construction, reqwest execution,
  public error parsing, API-version checks, and streaming request builders, but
  it is not FFI-safe as-is because it exposes reqwest, URL/header/status types,
  async traits, and reqwest errors.
- `taru-client-protocol` still has strict serde enums for some additive public
  string fields, which would regress the forward-compatible behavior Android
  just gained if mobile decode moved to Rust without a tolerance plan.

The owner preference is to expose and solve Rust core / UniFFI complexity now
if doing so prevents a Kotlin runtime from becoming a second portable client
implementation that must later be replaced.

## Decision

Taru will pull shared Rust client-core work forward now, before adding a new
Kotlin SDK runtime layer.

The first target is **not** full Rust-owned Android networking. The first
target is a no-socket, FFI-safe Rust client core that builds Public Client API
request specifications and interprets response specifications, while Android
continues to execute HTTP through its platform transport and security policy.

The crate topology should be:

- `taru-client-protocol`: permissive Public Client API wire DTOs, public error
  envelope, API constants, route inventory, and public wire vocabularies.
- `taru-client-core`: new permissive, FFI-safe core for request construction,
  response interpretation, API-version checks, public error-envelope parsing,
  redaction primitives, and eventually playback decision/request
  interpretation. It must not depend on reqwest, Tokio, Android, Compose,
  Media3, or UniFFI-specific generated code.
- `taru-client`: Rust async/reqwest adapter that can reuse `taru-client-core`
  while keeping CLI/server tooling ergonomic.
- `taru-client-uniffi`: thin binding crate over `taru-client-core`. It owns
  UniFFI scaffolding and generated binding surfaces, not policy.
- `sdk/kotlin`: generated JVM SDK artifact for Public Client API DTOs,
  constants, and request descriptors. It remains useful for JVM consumers and
  transition tests, but it is not the durable runtime-policy owner.

The first Android tracer should use app-supplied transport:

1. Android normalizes the base URL, applies cleartext/TLS policy, resolves the
   active profile, and retrieves the token from its vault.
2. Rust core builds a FFI-safe `GET /health` request specification.
3. Android executes that request with the existing Android transport.
4. Rust core interprets status, `x-taru-api-version`, public error envelope,
   and `HealthResponse`.
5. Rust core builds an authenticated `GET /libraries?limit=1&offset=0` request
   specification for the auth probe.
6. Android executes that request.
7. Rust core interprets the HTTP status, API-version header, and public error
   envelope without requiring library-list DTO decode for the tracer.
8. Android maps Rust core outcomes to existing product-owned
   `ConnectionFailureCategory`, diagnostics, and user copy.

The FFI boundary must use explicit FFI-safe records and enums. It must not
expose `reqwest::Url`, `HeaderMap`, `StatusCode`, `reqwest::Error`, generic
typed decode APIs, borrowed Rust data, async traits, or platform-specific
transport exceptions across UniFFI.

Recommended first FFI-safe shapes:

- `CoreHttpHeader { name: String, value: String }`
- `CoreHttpRequest { request_id: String, method: String, url: String,
  headers: Vec<CoreHttpHeader>, body_utf8: Option<String>,
  safe_preview: CoreSafeRequestPreview }`
- `CoreHttpResponse { request_id: String, status_code: i32,
  headers: Vec<CoreHttpHeader>, body_utf8: String }`
- `CoreSafeRequestPreview { method: String, url: String,
  headers: Vec<CoreHttpHeader> }`
- `CorePublicError { code: String, message: String }`
- `CoreRuntimeFailure { kind: CoreRuntimeFailureKind, status_code: Option<i32>,
  observed_api_version: Option<String>, public_error: Option<CorePublicError>,
  request: Option<CoreSafeRequestPreview> }`
- `CoreConnectionProbeOutcome` as an explicit route-specific outcome:
  `NextRequest(CoreHttpRequest)`, `Success(CoreConnectionProbeSuccess)`, or
  `Failure(CoreRuntimeFailure)`.

The Rust core may own:

- request path/query construction for core-owned flows;
- bearer header injection when Android supplies a token;
- API-version header observation;
- `/health.version` compatibility checks;
- public error-envelope parsing;
- invalid JSON / invalid public response classification;
- redaction of bearer tokens, explicit secrets, local paths, file URLs, and
  unsafe diagnostic text;
- playback decision interpretation and streaming request construction once the
  connection tracer proves the boundary;
- portable cache/download coordination metadata in later lanes.

The Rust core must not own:

- Android profile persistence or profile selection UI;
- token vault storage;
- Android cleartext policy, TLS trust behavior, proxy policy, or platform HTTP
  exceptions in the first tracer;
- Compose state, navigation, accessibility copy, product diagnostics copy, or
  product failure categories;
- Media3 player instances, Android media sessions, PiP, cast, route UI,
  subtitle/audio-track UI, or TV focus behavior;
- server/admin/internal APIs.

Before Rust core decodes browse/playback/catalog DTOs for Android, Rust public
wire value handling must preserve unknown additive public strings. It may do
that by replacing strict protocol enums with tolerant value types, by adding
route-specific tolerant DTOs for FFI, or by otherwise guaranteeing Android does
not regress from the generated Kotlin SDK `wireValue` behavior. The first
connection tracer may avoid this by not decoding strict-enum route bodies.

## Consequences

- The durable runtime owner becomes Rust, matching ADR 0026, instead of adding a
  Kotlin runtime layer that would later need replacement.
- Android build complexity is pulled forward deliberately, but only after a
  no-socket core and binding topology are documented.
- Android networking, cleartext/TLS policy, profile/token storage, UI, and
  Media3 remain platform-owned.
- `taru-client` should gradually become an adapter over the core rather than a
  second Rust implementation of the same request/response policy.
- Rust-side forward compatibility becomes a first-class requirement before
  mobile browse/playback decode moves to Rust.
- The first tracer can be validated without moving playback, browse UI,
  catalog presentation, or user-playback state.

## Alternatives Considered

- Keep Android-owned `PublicClientApiExecutor` indefinitely. Rejected because it
  preserves short-term simplicity but leaves protocol runtime behavior in the
  Android app and invites duplicate portable implementations.
- Add a hand-written Kotlin SDK runtime. Rejected as the default because it
  would create a second portable runtime for auth, errors, version checks,
  redaction, and playback interpretation while the accepted long-term owner is
  Rust.
- Bind Android directly to the existing `taru-client`. Rejected because
  `taru-client` currently exposes reqwest, async transport, URL/header/status
  types, and errors that are not an appropriate UniFFI boundary.
- Start with Rust-owned Android networking. Rejected for the first tracer
  because it would move TLS, cleartext, proxy, certificate, and platform HTTP
  behavior before the core request/response contract is proven.
- Wait for iOS, downloads, or Android TV before starting Rust core. Rejected
  because the generated SDK adoption blocker is gone and the owner explicitly
  prefers solving core/FFI complexity before a Kotlin runtime hardens.

## Related Workstreams

- `docs/workstreams/generated-sdk-runtime-ownership/`
- `docs/workstreams/android-generated-public-client-sdk/`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/`
- [0025: Generate Public Client OpenAPI From Protocol-Owned Wire Types](0025-openapi-public-client-sdk-contract.md)
- [0026: Use Native Client Shells With a Shared Rust Client Core](0026-native-client-shells-with-shared-rust-client-core.md)
- [0031: Sequence Generated Client SDK Before Mobile Rust FFI](0031-android-client-sdk-before-mobile-rust-ffi.md)
