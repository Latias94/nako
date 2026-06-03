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
- Base: a route ignores trace context; root middleware still returns
  `x-request-id` for client/operator correlation.
- Bad: use raw URL, local path, bearer token, playback ticket, provider payload,
  or arbitrary user text as a request ID.
- Bad: mount trace context only on protected routes, which misses `/health`,
  CORS preflight, addon runtime routes, or auth/network rejections.

### 6. Tests Required

- Unit test: safe inbound request IDs normalize and unsafe values are rejected.
- Middleware test: typed context is available to an Axum handler and response
  header matches.
- Root router test: `/health` returns generated `x-request-id`.
- Root router test: valid inbound IDs are echoed and unsafe inbound IDs are
  replaced without leaking the unsafe string.
- Root router test: auth rejection and network/preflight short-circuit
  responses still include `x-request-id`.

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
