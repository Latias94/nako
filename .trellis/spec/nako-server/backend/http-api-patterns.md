# HTTP API Routes and Auth

Use this spec for `crates/nako-server` HTTP work. These rules describe the
current Axum implementation and the accepted ADR boundary; they are not a
refactor target.

## Scenario: Server HTTP Route Slice

### 1. Scope / Trigger

- Trigger: adding or changing an HTTP endpoint, auth bypass, admin-only route,
  library access check, request query, or route test in `crates/nako-server`.
- Code evidence: `src/http.rs`, `src/http/account.rs`, `src/http/admin.rs`,
  `src/http/user_playlist.rs`, `src/http/access.rs`, `src/http/tests/mod.rs`.
- Architecture authority: ADR 0019, ADR 0024, ADR 0027, ADR 0036, ADR 0037,
  ADR 0053.

### 2. Signatures

- Route groups expose `pub(super) fn routes() -> Router<NakoApp>`.
- Public login/invitation routes use `pub(super) fn public_routes() -> Router<NakoApp>`.
- Handlers take Axum extractors directly:
  - `State(app): State<NakoApp>`
  - `Extension(principal): Extension<AuthenticatedPrincipal>`
  - `Path(id): Path<StrongId>` or tuple paths for multiple IDs
  - `Query(query): Query<QueryType>`
  - `Json(request): Json<RequestDto>`
- Handlers return `ApiResult<impl IntoResponse>` or `ApiResult<Json<ResponseDto>>`.

### 3. Contracts

- Root router assembly is centralized in `build_router_with_auth`.
- `system::public_routes()` is public. `account::public_routes()` is
  unauthenticated but still gets `network::enforce_network_boundary`.
- Protected routes are merged into one router, then layered with
  `enforce_network_boundary`, `auth::require_auth`, `Extension(app.clone())`,
  and `Extension(auth)`.
- Admin routes live under `/admin/v1/*` and finish with
  `.route_layer(middleware::from_fn(require_admin_principal))`.
- All responses pass through `add_api_version_header`, which inserts
  `x-nako-api-version` from `nako_api::public_client`.
- Request/response wire types come from `nako_api`; do not expose internal
  database records directly through new handlers.
- Query parsing lives in `http/query.rs`; parse string filters into domain enums
  and IDs before calling app services.
- ADR 0019 keeps HTTP handlers thin: translate request/response and delegate to
  focused app services rather than growing `NakoApp` as a feature god object.

### 4. Validation & Error Matrix

| Condition | Current behavior |
|-----------|------------------|
| Missing `InboundAuthState` extension | `401` with `ErrorResponse` code `unauthorized` and `WWW-Authenticate: Bearer` |
| Auth disabled in config | Inserts `AuthenticatedPrincipal::bootstrap_admin()` and continues |
| Bearer token matches configured token | Inserts bootstrap admin principal and continues |
| Bearer token resolves to user session | Inserts `UserPrincipalId`, `AuthenticatedPrincipal`, and `UserSessionId` |
| Bearer token missing, empty, or invalid | `401` with `WWW-Authenticate: Bearer` |
| Non-admin principal reaches `/admin/v1/*` | `403`, code `forbidden`, message `administrator role is required` |
| Library/item/source access is insufficient | `NakoError::Forbidden` with required Library Access level in the message |
| Playback ticket bypass | Only `GET` or `HEAD` on the listed media byte routes with a `ticket` query key |

### 5. Good / Base / Bad Cases

- Good: add route constants or DTOs in `nako-api`, add the Axum route in the
  right module, use `State`, `Extension`, `Path`, `Query`, and `Json`, call an
  app service, return a `nako_api` response DTO, and add route tests.
- Base: read-only admin diagnostics route under `/admin/v1/...` with
  `require_admin_principal` inherited from `admin::routes()`.
- Bad: mounting a new admin route outside `admin::routes()`, bypassing
  `require_auth`, accepting raw `String` IDs when a strong ID type exists, or
  returning internal records directly.

### 6. Tests Required

- Add focused HTTP tests under `crates/nako-server/src/http/tests/` for route
  shape, auth/admin rejection, query parsing, public redaction, and status/body.
- Use `tower::ServiceExt` and local helper routers, following
  `public_client_router_with_principal` in `http/tests/mod.rs`.
- When adding admin UI-facing endpoints, also test the `nako-api` admin contract
  and the `apps/admin-web` data-source/client mapping.
- Focused gate examples:
  - `cargo nextest run -p nako-server http::tests::<module> --no-fail-fast`
  - `cargo check -p nako-server --tests`

### 7. Wrong vs Correct

#### Wrong

```rust
Router::new().route("/admin/v1/example", get(handler))
```

This creates an admin route without the existing admin route layer.

#### Correct

```rust
pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/admin/v1/example", get(handler))
        .route_layer(middleware::from_fn(require_admin_principal))
}
```

For a new route in an existing admin module, add it to the existing
`admin::routes()` chain so it inherits the admin principal check.

## Scenario: HTTP Request Trace Context

### 1. Scope / Trigger

- Trigger: changing root router middleware, request identity, CORS request
  headers, or handler-visible trace context in `crates/nako-server`.
- Code evidence: `src/http.rs`, `src/http/trace_context.rs`,
  `src/http/network.rs`, `src/http/tests/system.rs`.
- Architecture authority: ADR 0053 and
  `docs/architecture/CONTROL_PLANE.md`.

### 2. Signatures

- `trace_context::attach_http_trace_context(Request, Next) -> Response` is the
  root HTTP trace-context middleware.
- `trace_context::HttpTraceContext` is inserted into request extensions for
  future handlers that need request identity.
- `trace_context::X_REQUEST_ID_HEADER` is the canonical `x-request-id`
  response/request header.

### 3. Contracts

- Root router assembly must keep trace context outside middleware that can
  short-circuit, such as network boundary and auth rejection, so all responses
  get `x-request-id`.
- A valid inbound `x-request-id` is bounded, uses only ASCII alphanumeric,
  dash, underscore, or dot, and is normalized to lowercase.
- Missing or invalid inbound IDs are replaced with generated opaque IDs.
- CORS preflight allow headers include `x-request-id` so browser clients can
  provide a safe request ID.
- Request IDs are response headers and handler extensions only in the first
  slice. Do not add them to public/Admin DTOs, generated contracts, durable
  job rows, database schema, or response bodies without a dedicated task.
- When a handler needs request correlation in app/runtime diagnostics, convert
  `HttpTraceContext` at the HTTP boundary into an app-layer trace context that
  carries only the normalized safe `request_id`. App services must not parse
  HTTP headers or know the `x-request-id` header name.
- Internal diagnostic payloads may include `request_id` only when the value came
  from the typed trace context. They must not include raw paths, URLs, playback
  tickets, bearer tokens, Source Locators, FFmpeg argv, provider payloads, or
  arbitrary user text.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| No inbound request ID | Generate a safe opaque `req_...` ID and return it in `x-request-id`. |
| Valid inbound request ID | Normalize to lowercase, insert typed context, and echo it in `x-request-id`. |
| Inbound request ID contains whitespace, slash, comma, semicolon, URL/path characters, or is too long | Reject the inbound value, generate a safe opaque replacement, and do not echo the unsafe value. |
| Network/CORS middleware returns preflight or forbidden response | Response still includes generated or accepted `x-request-id`. |
| Auth middleware rejects a protected request | `401` keeps `WWW-Authenticate`, `x-nako-api-version`, and `x-request-id`. |

### 5. Good / Base / Bad Cases

- Good: add request-scoped diagnostics by extracting
  `Extension<HttpTraceContext>` and logging only `request_id`.
- Good: pass a sanitized app-layer trace context into HLS/playback runtime
  orchestration and include only `request_id` in internal outbox event payloads.
- Base: a route ignores trace context; root middleware still returns
  `x-request-id` for client/operator correlation.
- Base: a non-HTTP or test-only app-service call passes no trace context and
  preserves existing event payloads.
- Bad: use raw URL, local path, bearer token, playback ticket, provider payload,
  or arbitrary user text as a request ID.
- Bad: mount trace context only on protected routes, which misses `/health`,
  CORS preflight, addon runtime routes, or auth/network rejections.
- Bad: make app services depend on `HeaderMap`, raw header strings, or
  `x-request-id` parsing.

### 6. Tests Required

- Unit test: safe inbound request IDs normalize and unsafe values are rejected.
- Middleware test: typed context is available to an Axum handler and response
  header matches.
- Root router test: `/health` returns generated `x-request-id`.
- Root router test: valid inbound IDs are echoed and unsafe inbound IDs are
  replaced without leaking the unsafe string.
- Root router test: auth rejection and network/preflight short-circuit
  responses still include `x-request-id`.
- App/route test: when HLS playlist startup receives a safe inbound
  `x-request-id`, the resulting `PlaybackSessionFinished` outbox payload
  includes the normalized `request_id` and no ticket/path-sensitive material.

### 7. Wrong vs Correct

#### Wrong

```rust
let request_id = request.uri().to_string();
```

This can expose raw paths, query strings, playback tickets, or other sensitive
operator data.

#### Correct

```rust
let context = request
    .extensions()
    .get::<HttpTraceContext>()
    .expect("trace context middleware should run before handlers");
tracing::info!(request_id = %context.request_id(), "request accepted");
```

Handlers use the typed context and log only the sanitized request ID.

#### Wrong

```rust
async fn handler(headers: HeaderMap) {
    app.playback().start(headers.get("x-request-id").unwrap().to_str().unwrap()).await;
}
```

This pushes raw HTTP headers and unvalidated user input into app logic.

#### Correct

```rust
async fn handler(Extension(context): Extension<HttpTraceContext>) {
    let trace = PlaybackTraceContext::from_request_id(context.request_id().to_owned());
    app.playback()
        .hls_playlist_playback(HlsPlaylistPlaybackRequest {
            principal,
            source_id,
            client,
            preferences,
            playback_generation,
            trace_context: Some(trace),
            transport_query,
        })
        .await;
}
```

HTTP owns extraction and validation; app code receives only the safe request ID.

## Scenario: HLS Artifact Cache-Control

### 1. Scope / Trigger

- Trigger: changing HLS playlist or HLS segment HTTP responses in
  `crates/nako-server`.
- Code evidence: `src/http/playback.rs`,
  `src/http/tests/playback.rs`.
- Architecture authority: ADR 0053 and
  `docs/architecture/CONTROL_PLANE.md`.

### 2. Signatures

- `hls_playlist_response(body, session_id) -> Response` owns playlist response
  headers.
- `hls_segment(...) -> ApiResult<Response>` owns segment route response headers
  after the app service returns a manifest-approved segment plan.
- `apply_hls_artifact_cache_headers(&mut Response)` is the HLS-only helper for
  session artifact cache policy.

### 3. Contracts

- HLS playlist responses must include `Cache-Control: no-store`.
- HLS segment responses must include `Cache-Control: no-store`.
- Keep HLS response construction separate from Direct Play and Remux response
  construction. Do not change `apply_direct_play_headers` from an HLS-only task;
  Direct Play and Remux cache policy belongs to the dedicated playback byte
  route contract below.
- Preserve existing content type, content length, byte range, playback session
  id, auth, ticket, and status behavior.
- Do not add ETags, Last-Modified, immutable segment caching, public/Admin DTOs,
  generated contracts, or schema changes without a dedicated cache-contract
  task.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| HLS playlist response is authored | Includes `Cache-Control: no-store` plus existing playlist headers. |
| HLS segment response is served | Includes `Cache-Control: no-store` plus existing byte response headers. |
| Direct Play or Remux response is served | Cache behavior is controlled by `apply_direct_play_headers`, not by the HLS helper. |
| Segment is missing, unauthorized, unfinished, or invalid | Existing error/status behavior is unchanged. |

### 5. Good / Base / Bad Cases

- Good: call `apply_hls_artifact_cache_headers` only from HLS playlist and
  segment response construction.
- Base: no-store is conservative until token-aware cache keys, immutable
  artifact identity, and conditional GET behavior are specified.
- Bad: editing `apply_direct_play_headers` while trying to fix an HLS-only
  response bug.
- Bad: adding `ETag` or immutable `max-age` for session artifacts without
  access-control and invalidation tests.

### 6. Tests Required

- HTTP route test: HLS playlist response includes `Cache-Control: no-store`.
- HTTP route test: HLS segment response includes `Cache-Control: no-store`.
- Focused gate: `cargo nextest run -p nako-server hls_playlist --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
fn hls_segment(...) -> Response {
    let mut response = stream_direct_play_response(...).await?;
    response
}
```

This routes HLS session artifacts through Direct Play/Remux byte response
assembly instead of the manifest-backed HLS response path.

#### Correct

```rust
fn apply_hls_artifact_cache_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
}
```

HLS session artifacts get an explicit conservative cache policy without
changing other playback response types.

## Scenario: Playback Byte Cache-Control

### 1. Scope / Trigger

- Trigger: changing Direct Play or Remux media byte responses in
  `crates/nako-server`.
- Code evidence: `src/http/playback.rs`,
  `src/http/tests/playback.rs`.
- Architecture authority: ADR 0017, ADR 0036, ADR 0053, and
  `docs/architecture/CONTROL_PLANE.md`.

### 2. Signatures

- `stream_direct_play_response(body, uri, plan) -> ApiResult<Response>` owns
  Direct Play streaming response assembly.
- `stream_local_file_response(path, uri, plan) -> ApiResult<Response>` owns
  local Direct Play byte response assembly.
- `empty_direct_play_response(plan) -> Response` owns Direct Play and Remux
  HEAD/preflight or range-not-satisfiable empty response assembly.
- `apply_direct_play_headers(&mut Response, &DirectPlayResponsePlan)` owns
  Direct Play and Remux byte response headers.

### 3. Contracts

- Direct Play and Remux media byte responses must include
  `Cache-Control: no-store`.
- This applies to GET, HEAD/preflight, partial content, and
  range-not-satisfiable responses that use `apply_direct_play_headers`.
- Preserve existing status, `Accept-Ranges`, `Content-Type`, `Content-Length`,
  optional `Content-Range`, playback session header, auth, ticket validation,
  and body/no-body behavior.
- Keep this policy separate from HLS and selected artwork helpers.
- Do not add ETags, conditional GET, public/Admin DTOs, generated contracts,
  schema changes, immutable headers, or shared-cache/CDN behavior without a
  dedicated cache-contract task.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Direct Play GET/range response is served | Includes `Cache-Control: no-store` plus existing byte/range headers. |
| Direct Play HEAD/preflight response is served | Includes `Cache-Control: no-store` with existing headers and empty body. |
| Remux GET/range response is served | Includes `Cache-Control: no-store` plus existing byte/range/session headers. |
| Remux HEAD/preflight response is served | Includes `Cache-Control: no-store` with existing headers and empty body. |
| HLS playlist/segment response is served | Uses the HLS-specific no-store helper. |
| Selected artwork image response is served | Uses the selected artwork private cache/ETag contract, not playback byte policy. |

### 5. Good / Base / Bad Cases

- Good: add the header in `apply_direct_play_headers`, because that helper is
  already shared by Direct Play and Remux byte response paths.
- Base: Direct Play and Remux remain uncacheable transport responses; they do
  not get ETags or conditional GET in this slice.
- Bad: adding playback byte `no-store` by editing individual route handlers,
  which misses HEAD, range-not-satisfiable, or remux reuse paths.
- Bad: reusing selected artwork private cache headers for media byte routes.

### 6. Tests Required

- HTTP route test: Direct Play GET/range response includes
  `Cache-Control: no-store`.
- HTTP route test: Direct Play HEAD response includes `Cache-Control: no-store`
  and no body.
- HTTP route test: Remux GET/range response includes `Cache-Control: no-store`.
- HTTP route test: Remux HEAD response includes `Cache-Control: no-store` and
  no body.
- Focused gates:
  `cargo nextest run -p nako-server direct_stream_head_returns_headers_without_body --no-fail-fast`,
  `cargo nextest run -p nako-server direct_stream_route_records_playback_session_without_transcode_artifact --no-fail-fast`,
  `cargo nextest run -p nako-server remux_stream_route_runs_and_reuses_completed_output --no-fail-fast`, and
  `cargo nextest run -p nako-server head_remux_stream_route_exposes_session_without_body --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn remux_stream_source(...) -> Response {
    let mut response = stream_local_file_response(...).await?;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
```

This misses Direct Play, HEAD/preflight, and empty response paths.

#### Correct

```rust
fn apply_direct_play_headers(response: &mut Response, plan: &DirectPlayResponsePlan) {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
}
```

The shared byte response helper covers Direct Play and Remux consistently.

## Scenario: Selected Artwork Image Cache-Control

### 1. Scope / Trigger

- Trigger: changing authenticated Public Client selected artwork image byte
  responses in `crates/nako-server`.
- Code evidence: `src/http/catalog.rs`,
  `src/http/tests/addons.rs`.
- Architecture authority: ADR 0053,
  `docs/architecture/CONTROL_PLANE.md`, and
  `docs/architecture/LIBRARY_PIPELINE.md`.

### 2. Signatures

- `get_image(...) -> ApiResult<impl IntoResponse>` owns public selected artwork
  GET route translation.
- `head_image(...) -> ApiResult<impl IntoResponse>` owns public selected
  artwork HEAD route translation.
- `selected_image_response(image, include_body, if_none_match) -> Response`
  owns shared selected artwork byte response header assembly and conditional
  response matching.
- `apply_selected_artwork_cache_headers(&mut HeaderMap)` is the selected
  artwork-only helper for the private client-cache baseline.
- `selected_image_etag_matches(if_none_match, etag) -> bool` is the route-local
  exact-match guard for selected artwork conditional responses.

### 3. Contracts

- Selected artwork image GET responses must include
  `Cache-Control: private, max-age=86400`.
- Selected artwork image HEAD responses must include the same cache policy and
  must not include a response body.
- Selected artwork image GET/HEAD responses with an exact matching
  `If-None-Match` value must return `304 Not Modified`.
- A selected artwork 304 response must include the current safe `ETag` and
  `Cache-Control: private, max-age=86400`, and must not include a response
  body.
- Keep this policy selected-artwork-only. Do not apply it to HLS, Direct Play,
  Remux, Admin JSON routes, or unrelated public JSON catalog routes.
- Preserve existing `Content-Type`, `Content-Length`, safe `ETag`, auth,
  library access, selected artwork lookup, and variant query behavior.
- Auth and library access checks must run before any selected artwork 304
  response.
- Do not add metadata-only ETag preflight, weak-validator parsing, wildcard
  validators, `Last-Modified`, immutable headers, generated DTOs, schema
  changes, or shared-cache/CDN behavior without a dedicated cache-contract task.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Original selected artwork GET response is authored | Includes `Cache-Control: private, max-age=86400` plus existing content headers and safe ETag. |
| Original selected artwork HEAD response is authored | Includes the same cache policy, content headers, and safe ETag with an empty body. |
| Resized selected artwork variant GET/HEAD response is authored | Includes the same cache policy while preserving variant-specific content length and ETag. |
| `If-None-Match` exactly matches the current original or variant ETag | Returns `304 Not Modified` with current ETag, selected artwork cache policy, and empty body. |
| `If-None-Match` is missing, malformed, or does not match | Existing `200` GET/HEAD response behavior is unchanged. |
| Selected artwork is missing or unauthorized | Existing not-found/forbidden behavior is unchanged. |
| Variant query is invalid | Existing bad-request behavior is unchanged. |

### 5. Good / Base / Bad Cases

- Good: call `apply_selected_artwork_cache_headers` only from
  `selected_image_response`, which is shared by the selected artwork GET and
  HEAD handlers.
- Good: compare `If-None-Match` against the same quoted ETag `HeaderValue` that
  will be returned on a normal selected artwork response, so matching cannot
  drift from header authoring.
- Base: safe selected artwork ETags continue to identify original versus
  bounded variants; the cache helper does not change ETag generation.
- Base: 304 matching happens after the current image response has been derived.
  A metadata-only ETag preflight is a performance follow-on, not part of the
  first conditional-response contract.
- Bad: reusing the HLS `no-store` helper for selected artwork, which defeats
  client artwork caching.
- Bad: applying `private, max-age=86400` through a generic byte-route helper
  that changes HLS, Direct Play, Remux, or Admin response behavior.
- Bad: returning 304 before auth/library access checks or matching against raw
  user-provided ETag strings instead of the route-authored safe ETag header.

### 6. Tests Required

- HTTP route test: original selected artwork GET response includes
  `Cache-Control: private, max-age=86400`.
- HTTP route test: original selected artwork HEAD response includes the same
  cache policy and an empty body.
- HTTP route test: resized selected artwork GET/HEAD responses include the same
  cache policy while preserving variant-specific ETags.
- HTTP route test: matching `If-None-Match` returns `304 Not Modified` with the
  current ETag/cache headers and no body.
- HTTP route test: non-matching `If-None-Match` preserves normal `200` image
  response behavior.
- Focused gates:
  `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
  and
  `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
fn selected_image_response(...) -> Response {
    let mut response = ...;
    apply_hls_artifact_cache_headers(&mut response);
    response
}
```

This treats long-lived authenticated artwork like session-scoped HLS playback
artifacts and disables useful private client caching.

#### Wrong

```rust
async fn get_image(headers: HeaderMap, ...) -> Response {
    if headers.contains_key(header::IF_NONE_MATCH) {
        return StatusCode::NOT_MODIFIED.into_response();
    }
    ...
}
```

This can bypass auth/access checks and returns 304 without proving the client
has the current selected artwork ETag.

#### Correct

```rust
fn selected_image_response(image: ManagedArtworkImageBytes, if_none_match: Option<&HeaderValue>) -> Response {
    let headers = response.headers_mut();
    apply_selected_artwork_cache_headers(headers);
    response
}
```

Selected artwork gets a route-specific private cache baseline without changing
playback artifacts or unrelated routes.

#### Correct

```rust
if etag
    .as_ref()
    .is_some_and(|etag| selected_image_etag_matches(if_none_match, etag))
{
    let mut response = Body::empty().into_response();
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    let headers = response.headers_mut();
    apply_selected_artwork_cache_headers(headers);
    headers.insert(header::ETAG, etag.clone());
    return response;
}
```

The route matches only against the current safe ETag after normal selected
artwork lookup and access checks.
