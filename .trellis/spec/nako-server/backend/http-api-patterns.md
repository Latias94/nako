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

## Scenario: Public Catalog Read Access Boundary

### 1. Scope / Trigger

- Trigger: changing Public Catalog browse, search, relation, detail, credits,
  images, or source-shaped read routes in `crates/nako-server/src/http/catalog.rs`.
- Code evidence: `src/http/catalog.rs`, `src/app/catalog.rs`,
  `src/http/tests/catalog.rs`.

### 2. Signatures

- Catalog HTTP handlers take `Extension(AuthenticatedPrincipal)` and pass it to
  `CatalogAppService`.
- Public item detail surfaces use app-service methods such as:
  - `CatalogAppService::get_item(&AuthenticatedPrincipal, MediaItemId)`
  - `CatalogAppService::list_item_credits(&AuthenticatedPrincipal, MediaItemId)`
  - `CatalogAppService::list_item_images(&AuthenticatedPrincipal, MediaItemId)`

### 3. Contracts

- HTTP handlers parse path/query inputs and return `nako_api::public_client`
  DTOs only.
- Public Catalog app services own browse access enforcement before response
  shaping for catalog read surfaces.
- Item detail source DTOs must be selected from accessible source records before
  DTO construction. Do not filter `ItemDetailResponse.sources` in HTTP after the
  DTO is built.
- `http::access::require_item_access` remains available for non-catalog routes
  whose semantics are still owned by their route slice, such as metadata.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Unknown item id | `NakoError::NotFound { entity: "media_item", ... }` |
| Ordinary principal cannot browse any source for item | `NakoError::Forbidden` with required Library Access level `browse` |
| Ordinary principal can browse only some sources on visible item | Detail returns the item and only accessible source DTOs |
| Administrator principal reads source-less or multi-source item | Preserve administrator access semantics |

### 5. Good / Base / Bad Cases

- Good: `/items/{item_id}` calls `app.catalog().get_item(&principal, item_id)`
  and the app service returns only browse-visible sources.
- Base: selected artwork byte routes use their own managed-artwork app-service
  access boundary because their HTTP response cache/ETag helpers remain
  route-local.
- Bad: route code calls `require_item_access`, then builds an unfiltered item
  detail response, then removes inaccessible source DTOs afterward.

### 6. Tests Required

- Focused catalog route tests for hidden item detail, credits, and images
  returning `403`.
- Focused catalog route test proving visible item detail omits hidden sources.
- Focused gate:
  `cargo nextest run -p nako-server http::tests::catalog --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;
let detail = app.catalog().get_item(item_id).await?;
Ok(Json(filter_item_detail_sources(&app, &principal, detail).await?))
```

#### Correct

```rust
Ok(Json(app.catalog().get_item(&principal, item_id).await?))
```

## Scenario: User Playlist Item Mutation Access Boundary

### 1. Scope / Trigger

- Trigger: changing playlist item add, remove, reorder, or visible item
  projection behavior in `crates/nako-server/src/http/user_playlist.rs` or
  `crates/nako-server/src/app/user_playlist.rs`.
- Code evidence: `src/http/user_playlist.rs`, `src/app/user_playlist.rs`,
  `src/http/tests/user_playlist.rs`, `src/app/tests/user_playlist.rs`.

### 2. Signatures

- Playlist item mutation HTTP handlers take `Extension(AuthenticatedPrincipal)`
  and pass the full principal into `UserPlaylistAppService`.
- App-service mutation request structs carry `AuthenticatedPrincipal` for:
  - `AddUserPlaylistItemRequest`
  - `RemoveUserPlaylistItemRequest`
  - `ReorderUserPlaylistItemsRequest`

### 3. Contracts

- HTTP handlers parse path/body data, pass the authenticated principal to the
  app service, and return public playlist DTOs.
- `UserPlaylistAppService` owns browse-access enforcement for item add,
  remove, and reorder before committing playlist item mutations.
- Use the existing accessible-media-item repository path for item visibility;
  do not reintroduce `require_item_access` loops in playlist HTTP handlers.
- Playlist ownership, expected-version handling, duplicate add behavior, and
  projection filtering remain app-service/repository responsibilities.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal adds a hidden item | `NakoError::Forbidden` with required Library Access level `browse` |
| Ordinary principal removes a hidden existing playlist item | `NakoError::Forbidden` with required Library Access level `browse` |
| Ordinary principal reorders a playlist containing a hidden item | `NakoError::Forbidden` with required Library Access level `browse` |
| Reorder body is malformed, duplicate, or misses existing items | Preserve `NakoError::InvalidInput` validation before mutation |
| Administrator mutates source-less or multi-source items | Preserve administrator access semantics |

### 5. Good / Base / Bad Cases

- Good: route parses `item_id`, then calls
  `app.user_playlist().add_item(AppAddUserPlaylistItemRequest { principal, ... })`.
- Base: playlist read projection filters hidden items and reports accessible
  item counts through repository projections.
- Bad: route code loops over reorder item IDs and calls
  `require_item_access(... Browse)` before calling the app service.

### 6. Tests Required

- App-service test proving hidden add, remove, and reorder are forbidden for an
  ordinary principal.
- HTTP route test proving hidden add, remove, and reorder return `403` while
  visible playlist item mutations still succeed.
- Focused gate:
  `cargo nextest run -p nako-server user_playlist --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;
app.user_playlist().add_item(request).await?;
```

#### Correct

```rust
app.user_playlist()
    .add_item(AppAddUserPlaylistItemRequest {
        principal,
        playlist_id,
        item_id,
        position,
        expected_version,
        added_at_ms: None,
    })
    .await?;
```

## Scenario: User Playback State Access Boundary

### 1. Scope / Trigger

- Trigger: changing Public Client user playback state read, progress update,
  watched update, or continue-watching behavior in `crates/nako-server`.
- Code evidence: `src/http/user_playback.rs`, `src/app/user_playback.rs`,
  `src/http/tests/user_playback.rs`, `src/app/tests/user_playback.rs`.

### 2. Signatures

- User playback HTTP handlers take `Extension(AuthenticatedPrincipal)` and
  pass the full principal into `UserPlaybackAppService`.
- App-service write request structs carry `AuthenticatedPrincipal`:
  - `UpdateUserPlaybackProgressRequest`
  - `SetUserWatchedStateRequest`
- `UserPlaybackAppService::get_state(&AuthenticatedPrincipal, MediaItemId)`
  owns read access for default/current playback state.

### 3. Contracts

- HTTP handlers parse path/body/timestamps and return public playback DTOs.
  They must not run `require_item_access` or `require_source_access` for user
  playback state routes.
- `UserPlaybackAppService` enforces Browse access for `get_state`.
- `UserPlaybackAppService` enforces Play access for progress and watched
  writes before committing state.
- If a write includes `source_id`, the app service checks source Play access
  and preserves source-to-item validation before writing state.
- Continue-watching list routes continue to use access-aware repository
  projections; do not turn them into route-local filtering loops.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has no Browse access for item | `GET /users/me/playback-state/items/{item_id}` returns `403` |
| Ordinary principal has Browse but not Play access | progress/watched write returns `403` |
| Write uses source from another item after access passes | `NakoError::InvalidInput` mentioning source/item mismatch |
| Continue-watching state exists for inaccessible item | list omits it and backfills visible rows before pagination |
| Administrator reads or writes source-less/multi-source item | Preserve administrator access semantics |

### 5. Good / Base / Bad Cases

- Good: route parses optional `source_id`, then calls
  `app.user_playback().update_progress(AppUpdateUserPlaybackProgressRequest { principal, ... })`.
- Base: continue-watching route calls
  `list_continue_watching_entries(&principal, page)`.
- Bad: route checks Play access with `require_item_access`, then the app
  service writes user playback state without knowing the caller principal.

### 6. Tests Required

- App-service test proving no-access reads and browse-only writes are forbidden.
- HTTP route test proving browse-only principals can read state but cannot
  update progress or watched state.
- Route test preserving source-from-another-item `400 invalid_input` behavior.
- Focused gate:
  `cargo nextest run -p nako-server user_playback --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Play).await?;
app.user_playback()
    .update_progress(AppUpdateUserPlaybackProgressRequest {
        principal_id: principal.principal_id,
        item_id,
        source_id,
        position_ms,
        duration_ms,
        reported_at_ms,
    })
    .await?;
```

#### Correct

```rust
app.user_playback()
    .update_progress(AppUpdateUserPlaybackProgressRequest {
        principal,
        item_id,
        source_id,
        position_ms,
        duration_ms,
        reported_at_ms,
    })
    .await?;
```

## Scenario: Renderer Play Command Access Boundary

### 1. Scope / Trigger

- Trigger: changing Public Client renderer play command creation, renderer
  playback-session startup, renderer command runtime records, or renderer
  source access checks in `crates/nako-server`.
- Code evidence: `src/http/renderer.rs`, `src/app/casting.rs`,
  `src/app/playback/renderer_flow.rs`, `src/http/tests/renderer.rs`,
  `src/app/tests/playback.rs`.

### 2. Signatures

- Renderer play HTTP handlers take `Extension(AuthenticatedPrincipal)` and pass
  the full principal into `CastingAppService`.
- `CastingAppService::play_on_renderer(PlayOnRendererRequest)` carries:
  - `principal: AuthenticatedPrincipal`
  - `renderer_session_id: RendererSessionId`
  - `source_id: MediaSourceId`
  - `position_ms: Option<u64>`
- `PlaybackAppService::start_renderer_playback_session(StartRendererPlaybackSessionRequest)`
  owns renderer playback planning after source access is enforced.

### 3. Contracts

- HTTP handlers parse `source_id`, path IDs, and JSON bodies only. They must not
  call `require_source_access` for renderer play command routes.
- Renderer play command creation must enforce source `Play` Library Access in
  the app-service path before playback sessions, renderer commands, transcode
  sessions, or HLS artifacts are created.
- Playback policy checks such as `remote_control` remain separate from Library
  Access. A principal without source `Play` access receives the standard
  Library Access forbidden message before playback-policy details are exposed.
- Renderer ownership, control capability checks, transport ticket construction,
  and Direct/Remux/HLS transport selection remain app-service responsibilities.
- Playback route-local source access helpers should not be reintroduced for
  renderer play command or renderer transport use paths.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Renderer play command returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but remote-control playback policy is disabled | Renderer play command returns `403` mentioning `remote_control` |
| Renderer play command is rejected before runtime startup | No playback session, renderer command, transcode session, or HLS artifact is created |
| Administrator starts renderer playback | Preserve administrator access and playback-policy semantics |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |

### 5. Good / Base / Bad Cases

- Good: `/renderers/{renderer_session_id}/commands/play` parses source ID and
  delegates to `app.casting().play_on_renderer(AppPlayOnRendererRequest { principal, ... })`.
- Base: renderer transport ticket issue and URL authoring stay in HTTP, while
  renderer transport use delegates current source `Play` rechecks to playback
  app-service session-use methods.
- Bad: route code calls `require_source_access(... Play)` before invoking
  `CastingAppService`, because non-HTTP renderer callers could bypass the
  access decision.

### 6. Tests Required

- App-service test proving a browse-only principal cannot start a renderer
  playback session and no runtime records are created.
- HTTP route test proving a browse-only principal receives `403` and no
  renderer command or playback/transcode runtime record is created.
- Existing renderer policy-denial route tests must continue proving Play access
  plus disabled `remote_control` policy still returns the playback-policy
  denial.
- Focused gate:
  `cargo nextest run -p nako-server renderer --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
app.casting().play_on_renderer(request).await?;
```

This makes HTTP the access authority and leaves future app-service callers able
to bypass renderer source access.

#### Correct

```rust
app.casting()
    .play_on_renderer(AppPlayOnRendererRequest {
        principal,
        renderer_session_id,
        source_id,
        position_ms,
    })
    .await?;
```

The app-service path owns source `Play` access before runtime records are
created, while HTTP remains a DTO and extractor boundary.

## Scenario: Playback Decision Access Boundary

### 1. Scope / Trigger

- Trigger: changing Public Client source playback decision lookup,
  `PlaybackAppService::get_source_playback_decision`, browser capability query
  mapping, or route-local playback decision source access checks in
  `crates/nako-server`.
- Code evidence: `src/http/playback.rs`, `src/app/playback/mod.rs`,
  `src/http/tests/playback.rs`, `src/app/tests/playback.rs`.

### 2. Signatures

- Playback decision HTTP handlers take `Extension(AuthenticatedPrincipal)` and
  pass the full principal into `PlaybackAppService`.
- `PlaybackAppService::get_source_playback_decision(&AuthenticatedPrincipal, MediaSourceId, ClientPlaybackCapabilities)`
  owns source loading, source `Play` Library Access, effective playback policy
  resolution, and playback planning.

### 3. Contracts

- HTTP playback decision lookup parses path/query inputs and delegates to the
  app service. It must not call route-local `require_source_access`.
- Source `Play` Library Access must be enforced in
  `PlaybackAppService::get_source_playback_decision` before playback planning,
  target capability reporting, or playback-policy details are returned.
- A principal without source `Play` access receives the standard Library Access
  forbidden message before playback-policy details such as `direct_play`,
  `remux`, or transcode permissions are exposed.
- Client capability query parsing, safe target DTO shaping, locator redaction,
  administrator access, unknown source behavior, and planner output stay
  unchanged.
- Direct Play, Remux, HLS, sidecar subtitle playback, and renderer transport
  use paths now delegate source `Play` access to `PlaybackAppService`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Playback decision lookup returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but playback policy denies a candidate mode | Playback decision response is returned with the existing policy-denied decision report |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |
| Administrator requests playback decision | Preserve administrator access and playback-policy semantics |
| Capability query requests remux/transcode preferences | Preserve existing query mapping and safe DTO output |

### 5. Good / Base / Bad Cases

- Good: `/sources/{source_id}/playback/decision` parses public query
  capabilities and calls
  `app.playback().get_source_playback_decision(&principal, source_id, client)`.
- Base: playback policy denial for a Play-authorized principal still returns a
  safe decision report instead of an HTTP error.
- Bad: route code calls `require_source_access(... Play)` before invoking the
  app service, because non-HTTP callers could bypass playback decision access.
- Bad: playback planning runs before source `Play` Library Access, because
  browse-only users would learn target compatibility or policy details for
  media they cannot play.

### 6. Tests Required

- App-service test proving a browse-only principal cannot get a playback
  decision and receives the standard Library Access `play` message before
  playback-policy details.
- HTTP route test proving a browse-only principal receives `403` with the same
  public message for playback decision lookup.
- Existing playback decision policy-denial route tests must continue proving a
  principal with source Play access receives a safe policy-denied decision
  report.
- Focused gates:
  `cargo nextest run -p nako-server playback_decision --no-fail-fast` and
  `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
Ok(Json(
    app.playback()
        .get_source_playback_decision(&principal, source_id, client)
        .await?,
))
```

This makes HTTP the access authority and leaves future app-service callers able
to bypass playback decision source access.

#### Correct

```rust
Ok(Json(
    app.playback()
        .get_source_playback_decision(&principal, source_id, client)
        .await?,
))
```

The app service owns source `Play` access before planning and policy details,
while HTTP remains the query parsing and DTO response boundary.

## Scenario: Direct Playback Access Boundary

### 1. Scope / Trigger

- Trigger: changing Direct Play source byte routes,
  `PlaybackAppService::direct_playback_stream`,
  `PlaybackAppService::direct_playback_preflight`, or browser-ticket-backed
  Direct Play session stream/preflight use in `crates/nako-server`.
- Code evidence: `src/http/playback.rs`, `src/app/playback/mod.rs`,
  `src/http/tests/playback.rs`, `src/app/tests/playback.rs`.

### 2. Signatures

- Direct Play HTTP GET/HEAD handlers resolve an
  `AuthenticatedPrincipal` or validated browser playback ticket principal and
  call app-service Direct Play methods.
- `PlaybackAppService::direct_playback_stream(DirectPlaybackStreamRequest)`
  owns source loading, source `Play` Library Access, Direct Play playback
  policy admission, and Direct Play response planning.
- `PlaybackAppService::direct_playback_preflight(DirectPlaybackPreflightRequest)`
  owns the same access and policy admission for HEAD/preflight response plans.
- `PlaybackAppService::direct_playback_session_stream(DirectPlaybackSessionStreamRequest)`
  and `PlaybackAppService::direct_playback_session_preflight(DirectPlaybackSessionStreamRequest)`
  recheck current source `Play` Library Access when a previously issued Direct
  browser ticket/session is used.

### 3. Contracts

- Direct Play `/sources/{source_id}/stream` GET/HEAD HTTP handlers parse
  request ranges, validate auth or tickets, assemble byte responses and
  headers, and delegate source `Play` access to the app service.
- Direct Play HTTP handlers must not call route-local
  `require_source_access(... RequiredLibraryAccess::Play)` for ordinary
  principal or Direct browser ticket paths.
- `PlaybackAppService::direct_playback_stream` and
  `PlaybackAppService::direct_playback_preflight` must enforce source `Play`
  Library Access before playback policy details, storage capability checks,
  byte plans, or playback sessions are exposed.
- Ticket-backed Direct session stream/preflight use must recheck current source
  `Play` Library Access at use time, so access revocation after ticket issue is
  effective.
- Ticket-backed Direct session stream/preflight use does not re-evaluate
  Direct Play playback policy at use time; mode policy is validated when the
  browser ticket is issued.
- Remux, HLS, sidecar subtitle playback, and renderer transport routes now
  delegate source `Play` access to `PlaybackAppService`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Direct Play GET/HEAD returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but Direct Play is disabled by playback policy | Direct Play app-service flow returns the mode-specific playback-policy denial |
| Direct browser ticket was issued, then source Play access is revoked before use | Ticket-backed Direct byte route returns `403` with required Library Access level `play` |
| Direct browser ticket is missing, malformed, expired, wrong-mode, or has the wrong subject | Preserve existing browser ticket unauthorized/forbidden behavior |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |
| Administrator requests Direct Play | Preserve administrator access and playback-policy semantics |

### 5. Good / Base / Bad Cases

- Good: Direct Play route code resolves principal/ticket context with the
  auth/ticket-only source playback resolver, then calls
  `app.playback().direct_playback_stream(...)` or
  `app.playback().direct_playback_preflight(...)`.
- Good: Direct browser ticket use calls the session stream/preflight
  app-service methods, and those methods recheck Library Access before looking
  up reusable playback sessions.
- Base: Remux and HLS source routes use the same auth/ticket resolver without
  route-local source `Play` access, then delegate access-sensitive work to
  `PlaybackAppService`.
- Bad: Direct Play route code calls `require_source_access(... Play)` before
  invoking the app service, because non-HTTP Direct Play callers could bypass
  source access.
- Bad: Direct Play policy checks run before source `Play` Library Access,
  because browse-only users would learn policy details such as `direct_play`.
- Bad: Direct browser ticket use trusts the issued ticket without a current
  source `Play` recheck, because revocation after ticket issue would not take
  effect.

### 6. Tests Required

- App-service test proving a browse-only principal cannot call both
  `direct_playback_stream` and `direct_playback_preflight`, receives the
  standard Library Access `play` message before `direct_play` policy details,
  and creates no playback session.
- HTTP route test proving a browse-only Direct Play stream request returns
  `403` with the same public Library Access message.
- HTTP route test proving a previously issued Direct browser ticket is rejected
  after source `Play` access is revoked.
- Existing Direct Play route gates must continue covering GET, HEAD/no-body,
  range handling, cache-control, playback-session headers, and ticket
  validation behavior.
- Focused gates:
  `cargo nextest run -p nako-server direct_stream --no-fail-fast`,
  `cargo nextest run -p nako-server direct_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`,
  `cargo nextest run -p nako-server browser_playback_ticket_rejects_browse_only_access_and_revocation_at_use --no-fail-fast`,
  and `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let resolved = resolve_source_playback_context(
    Extension(principal),
    &app,
    source_id,
    BrowserPlaybackTicketMode::Direct,
    ticket.as_deref(),
)
.await?;
require_source_access(&app, &resolved.principal, source_id, RequiredLibraryAccess::Play).await?;
```

This keeps Direct Play source `Play` access route-local after auth/ticket
resolution and leaves future app-service callers able to bypass the Direct byte
access boundary.

#### Correct

```rust
let resolved = resolve_source_playback_context(
    Extension(principal),
    &app,
    source_id,
    BrowserPlaybackTicketMode::Direct,
    ticket.as_deref(),
)
.await?;
```

The route still owns auth/ticket resolution and byte response mechanics, while
`PlaybackAppService` owns source `Play` access before Direct Play planning or
session use.

## Scenario: Remux Playback Access Boundary

### 1. Scope / Trigger

- Trigger: changing Remux source byte routes,
  `PlaybackAppService::remux_playback_stream`,
  `PlaybackAppService::remux_playback_preflight`, or browser-ticket-backed
  Remux session stream use in `crates/nako-server`.
- Code evidence: `src/http/playback.rs`,
  `src/app/playback/remux_flow.rs`, `src/http/tests/playback.rs`,
  `src/app/tests/playback.rs`.

### 2. Signatures

- Remux HTTP GET/HEAD handlers resolve an `AuthenticatedPrincipal` or
  validated browser playback ticket principal and call app-service Remux
  methods.
- `PlaybackAppService::remux_playback_stream(RemuxPlaybackStreamRequest)` owns
  source `Play` Library Access, Remux playback policy admission, transcode
  session startup/reuse, playback-session linkage, and byte response planning.
- `PlaybackAppService::remux_playback_preflight(RemuxPlaybackPreflightRequest)`
  owns the same access and policy admission for HEAD/preflight response plans.
- `PlaybackAppService::remux_playback_session_stream(RemuxPlaybackSessionStreamRequest)`
  rechecks current source `Play` Library Access when a previously issued Remux
  browser ticket/session is used.

### 3. Contracts

- Remux `/sources/{source_id}/stream/remux` GET/HEAD HTTP handlers parse query
  and range inputs, validate auth or tickets, assemble byte responses and
  headers, and delegate source `Play` access to the app service.
- Remux HTTP handlers must not call route-local
  `require_source_access(... RequiredLibraryAccess::Play)` for ordinary
  principal or Remux browser ticket paths.
- `PlaybackAppService::remux_playback_stream` and
  `PlaybackAppService::remux_playback_preflight` must enforce source `Play`
  Library Access before Remux playback-policy details, transcode session
  startup, storage staging, byte plans, or playback sessions are exposed.
- Ticket-backed Remux session stream use must recheck current source `Play`
  Library Access at use time, so access revocation after ticket issue is
  effective.
- If a ticket-backed Remux session has not yet linked a transcode session,
  Remux playback policy still applies before artifact startup, but Library
  Access denial must happen first.
- HLS, sidecar subtitle playback, and renderer transport routes now delegate
  source `Play` access to `PlaybackAppService`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Remux GET/HEAD returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but Remux is disabled by playback policy | Remux app-service flow returns the mode-specific playback-policy denial |
| Remux browser ticket was issued, then source Play access is revoked before use | Ticket-backed Remux byte route returns `403` with required Library Access level `play` and does not start FFmpeg |
| Remux browser ticket is missing, malformed, expired, wrong-mode, or has the wrong subject | Preserve existing browser ticket unauthorized/forbidden behavior |
| Remux process capacity is busy after access and policy pass | Preserve immediate playback resource pressure conflict |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |
| Administrator requests Remux | Preserve administrator access and playback-policy semantics |

### 5. Good / Base / Bad Cases

- Good: Remux route code resolves principal/ticket context with the
  auth/ticket-only source playback resolver, then calls
  `app.playback().remux_playback_stream(...)` or
  `app.playback().remux_playback_preflight(...)`.
- Good: Remux browser ticket use calls the session stream app-service method,
  and that method rechecks Library Access before playback-session lookup or
  artifact startup.
- Base: HLS playlist and segment routes use the same auth/ticket-only source
  playback resolver and delegate source `Play` access to
  `PlaybackAppService`.
- Bad: Remux route code calls `require_source_access(... Play)` before
  invoking the app service, because non-HTTP Remux callers could bypass source
  access.
- Bad: Remux policy checks run before source `Play` Library Access, because
  browse-only users would learn policy details such as `remux`.
- Bad: Remux browser ticket use trusts the issued ticket without a current
  source `Play` recheck, because revocation after ticket issue would not take
  effect.

### 6. Tests Required

- App-service test proving a browse-only principal cannot call both
  `remux_playback_stream` and `remux_playback_preflight`, receives the standard
  Library Access `play` message before `remux` policy details, and creates no
  playback or transcode session.
- HTTP route test proving a browse-only Remux stream request returns `403` with
  the same public Library Access message.
- HTTP route test proving a previously issued Remux browser ticket is rejected
  after source `Play` access is revoked, before FFmpeg starts.
- Existing Remux route gates must continue covering GET, HEAD/no-body, range
  handling, active/completed session reuse, cache-control, playback-session
  headers, resource admission, and ticket validation behavior.
- Focused gates:
  `cargo nextest run -p nako-server remux_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`,
  `cargo nextest run -p nako-server playback_routes_require_play_library_access --no-fail-fast`,
  `cargo nextest run -p nako-server remux_browser_playback_ticket_rejects_revocation_at_use --no-fail-fast`,
  `cargo nextest run -p nako-server remux_stream --no-fail-fast`,
  and `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let resolved = resolve_source_playback_context(
    &app,
    principal,
    source_id,
    BrowserPlaybackTicketMode::Remux,
    ticket.as_deref(),
)
.await?;
require_source_access(&app, &resolved.principal, source_id, RequiredLibraryAccess::Play).await?;
```

This keeps Remux source `Play` access route-local after auth/ticket resolution
and leaves future app-service callers able to bypass the Remux byte access
boundary.

#### Correct

```rust
let resolved = resolve_source_playback_context(
    &app,
    principal,
    source_id,
    BrowserPlaybackTicketMode::Remux,
    ticket.as_deref(),
)
.await?;
```

The route still owns auth/ticket resolution and byte response mechanics, while
`PlaybackAppService` owns source `Play` access before Remux planning, artifact
startup, or session use.

## Scenario: HLS Playback Access Boundary

### 1. Scope / Trigger

- Trigger: changing HLS playlist routes, HLS segment routes,
  browser-ticket-backed HLS use, `PlaybackAppService::hls_playlist_playback`,
  `PlaybackAppService::hls_playlist_for_playback_session`, or
  `PlaybackAppService::hls_segment_playback` in `crates/nako-server`.
- Code evidence: `src/http/playback.rs`, `src/app/playback/mod.rs`,
  `src/app/playback/hls_flow.rs`, `src/app/playback/hls_artifact.rs`,
  `src/http/tests/playback.rs`, and `src/app/tests/playback.rs`.

### 2. Signatures

- HLS playlist HTTP handlers resolve an `AuthenticatedPrincipal` or validated
  browser playback ticket principal and call app-service HLS playlist methods.
- `PlaybackAppService::hls_playlist_playback(HlsPlaylistPlaybackRequest)` owns
  source `Play` Library Access, HLS playback policy admission, HLS transcode
  session startup/reuse, playback-session creation/linkage, and playback
  playlist planning.
- `PlaybackAppService::hls_playlist_for_playback_session(HlsPlaylistSessionRequest)`
  rechecks current source `Play` Library Access when a browser-ticket-backed or
  existing HLS playback session is used.
- `PlaybackAppService::hls_segment_playback(HlsSegmentPlaybackRequest)` owns
  source `Play` Library Access before manifest-backed segment planning.
- `resolve_source_playback_context(...)` is an auth/ticket-only source playback
  resolver. It must not carry a route-local source-access flag.

### 3. Contracts

- HLS `/sources/{source_id}/stream/hls/playlist.m3u8` HTTP handlers parse query
  inputs, validate auth or tickets, decorate playlist URLs, assemble playlist
  responses and headers, and delegate source `Play` access to the app service.
- HLS `/playback/sessions/{session_id}/hls/segments/{segment_name}` HTTP
  handlers resolve the playback-session target and auth/ticket context, then
  delegate source `Play` access and segment planning to the app service.
- HLS playlist and segment HTTP handlers must not call route-local
  `require_source_access(... RequiredLibraryAccess::Play)` through
  `resolve_source_playback_context` for ordinary principal or HLS browser
  ticket paths.
- `PlaybackAppService::hls_playlist_playback` must enforce source `Play`
  Library Access before HLS playback-policy details, resource admission,
  FFmpeg input staging, transcode session startup, playback session creation,
  or playlist response planning.
- `PlaybackAppService::hls_playlist_for_playback_session` must recheck current
  source `Play` Library Access before existing-session reuse or lazy HLS
  playlist startup.
- `PlaybackAppService::hls_segment_playback` must enforce source `Play`
  Library Access before manifest-backed segment planning or byte response
  serving.
- `hls_source_with_policy`, `hls_playlist_with_policy`, and HLS artifact
  manifest planning keep their existing ownership until dedicated tasks change
  those boundaries. Sidecar subtitle playback and renderer transport use paths
  delegate source `Play` access to `PlaybackAppService`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | HLS playlist and segment use returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but HLS transcode is disabled by playback policy | HLS app-service flow returns the mode-specific playback-policy denial |
| HLS browser ticket was issued, then source Play access is revoked before playlist use | Ticket-backed HLS playlist route returns `403` with required Library Access level `play` |
| HLS browser ticket was issued, playlist was generated, then source Play access is revoked before segment use | Ticket-backed HLS segment route returns `403` with required Library Access level `play` |
| HLS browser ticket is missing, malformed, expired, wrong-mode, or has the wrong playback session subject | Preserve existing browser ticket unauthorized/forbidden behavior |
| HLS process capacity is busy after access and policy pass | Preserve bounded HLS resource-admission behavior |
| Unknown source or playback session ID | Preserve existing `NakoError::NotFound` behavior |
| Administrator requests HLS playback | Preserve administrator access and playback-policy semantics |

### 5. Good / Base / Bad Cases

- Good: HLS playlist route resolves auth/ticket context, then calls
  `app.playback().hls_playlist_playback(...)` or
  `app.playback().hls_playlist_for_playback_session(...)`.
- Good: HLS segment route resolves a target with
  `hls_segment_playback_target`, validates ticket/session scope, then calls
  `app.playback().hls_segment_playback(...)`.
- Base: renderer transport HLS playlist/segment use may still pass through the
  renderer transport resolver, but the HLS app-service method also owns the
  source `Play` access boundary for the returned principal.
- Bad: HLS route code calls `require_source_access(... Play)` before invoking
  the app service, because non-HTTP HLS callers could bypass source access.
- Bad: HLS policy checks, resource admission, FFmpeg staging, or segment
  manifest planning run before source `Play` Library Access, because
  browse-only users would learn policy or artifact details.
- Bad: `resolve_source_playback_context` grows another boolean for route-local
  source access, because that recreates divergent Direct/Remux/HLS behavior.

### 6. Tests Required

- App-service test proving a browse-only principal cannot call
  `hls_playlist_playback`, receives the standard Library Access `play` message
  before `video_transcode` or HLS policy details, and creates no playback or
  HLS transcode session.
- HTTP route test proving a browse-only HLS playlist request returns `403` with
  the same public Library Access message.
- HTTP route tests proving a previously issued HLS browser ticket is rejected
  after source `Play` access is revoked for both playlist use and segment use.
- Existing HLS route gates must continue covering playlist startup, running
  session readiness, segment manifest protection, playlist URL rewriting,
  cache-control, playback-session headers, resource admission, and ticket
  validation behavior.
- Focused gates:
  `cargo nextest run -p nako-server hls_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`,
  `cargo nextest run -p nako-server playback_routes_require_play_library_access --no-fail-fast`,
  `cargo nextest run -p nako-server hls_browser_playback_ticket_rejects_revocation_at_playlist_use --no-fail-fast`,
  `cargo nextest run -p nako-server hls_browser_playback_ticket_rejects_revocation_at_segment_use --no-fail-fast`,
  `cargo nextest run -p nako-server hls_playlist --no-fail-fast`,
  `cargo nextest run -p nako-server hls_segment --no-fail-fast`, and
  `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let resolved = resolve_source_playback_context(
    &app,
    principal,
    source_id,
    BrowserPlaybackTicketMode::Hls,
    ticket.as_deref(),
    true,
)
.await?;
```

This keeps HLS source `Play` access route-local and leaves future app-service
callers able to bypass the HLS playlist or segment access boundary.

#### Correct

```rust
let resolved = resolve_source_playback_context(
    &app,
    principal,
    source_id,
    BrowserPlaybackTicketMode::Hls,
    ticket.as_deref(),
)
.await?;
```

The route owns auth/ticket resolution and response mechanics, while
`PlaybackAppService` owns source `Play` access before HLS playlist startup,
existing-session reuse, or segment planning.

## Scenario: Subtitle Playback Access Boundary

### 1. Scope / Trigger

- Trigger: changing sidecar subtitle playback route
  `/sources/{source_id}/subtitles/{stream_index}`,
  `PlaybackAppService::subtitle_playback`, subtitle browser-ticket-backed use,
  or subtitle principal/ticket resolution in `crates/nako-server`.
- Code evidence: `src/http/playback.rs`, `src/app/playback/mod.rs`,
  `src/http/tests/playback.rs`, `src/app/tests/playback.rs`.

### 2. Signatures

- Subtitle HTTP GET handlers resolve an `AuthenticatedPrincipal` or validated
  subtitle browser playback ticket principal and call
  `PlaybackAppService::subtitle_playback(SubtitlePlaybackRequest)`.
- `SubtitlePlaybackRequest` carries:
  - `principal: AuthenticatedPrincipal`
  - `source_id: MediaSourceId`
  - `stream_index: u32`
- `resolve_subtitle_playback_principal(...)` is an auth/ticket-only resolver.
  It validates subtitle ticket identity and stream scope, or returns the
  ordinary authenticated principal.

### 3. Contracts

- Subtitle HTTP handlers parse source/stream path inputs, validate auth or a
  subtitle browser playback ticket, assemble text subtitle responses and
  headers, and delegate source `Play` access to the app service.
- Subtitle HTTP handlers must not call route-local
  `require_source_access(... RequiredLibraryAccess::Play)` for ordinary
  principal or subtitle browser ticket paths.
- `PlaybackAppService::subtitle_playback` must enforce source `Play` Library
  Access before subtitle playback-policy details, probe lookup, sidecar
  stream selection, sidecar file-name resolution, storage backend lookup,
  sidecar stat/read, or response shaping.
- Subtitle browser ticket use must recheck current source `Play` Library
  Access at use time through `PlaybackAppService::subtitle_playback`, so
  access revocation after ticket issue is effective.
- Subtitle browser ticket issue behavior, subtitle stream scoping, content
  type mapping, sidecar path redaction, byte limits, and text body behavior
  keep their existing route or app-service ownership.
- Renderer transport resolvers are auth/ticket-only and delegate source `Play`
  access to playback app-service session-use methods.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Subtitle route returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but media playback or remote playback is disabled by policy | Subtitle app-service flow returns the existing playback-policy denial after source Play access passes |
| Subtitle browser ticket was issued, then source Play access is revoked before use | Ticket-backed subtitle route returns `403` with required Library Access level `play` |
| Subtitle browser ticket is missing, malformed, expired, wrong-mode, wrong-source, or wrong-stream | Preserve existing unauthorized/forbidden ticket behavior |
| Subtitle stream is missing after source Play access passes | Preserve `NakoError::NotFound` for `subtitle_stream` |
| Subtitle sidecar file is missing after source Play access passes | Preserve redacted subtitle sidecar not-found behavior |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |
| Administrator requests subtitle playback | Preserve administrator access and playback-policy semantics |

### 5. Good / Base / Bad Cases

- Good: subtitle route resolves only auth or subtitle ticket identity, then
  calls `app.playback().subtitle_playback(SubtitlePlaybackRequest { ... })`.
- Good: subtitle browser ticket use delegates to the same app-service method
  as ordinary subtitle playback, so current Library Access is rechecked before
  sidecar details.
- Base: subtitle ticket issue continues to validate source access and subtitle
  stream scope in `validate_browser_playback_ticket_request` before signing a
  ticket.
- Bad: subtitle route code calls `require_source_access(... Play)` before
  invoking the app service, because non-HTTP subtitle callers could bypass
  source access.
- Bad: subtitle policy checks, probe lookup, sidecar file-name resolution, or
  storage access run before source `Play` Library Access, because browse-only
  users would learn playback-policy or sidecar details.
- Bad: subtitle browser ticket use trusts the issued ticket without a current
  source `Play` recheck, because revocation after ticket issue would not take
  effect.

### 6. Tests Required

- App-service test proving a browse-only principal cannot call
  `subtitle_playback`, receives the standard Library Access `play` message
  before `media_playback` or subtitle sidecar details, and does not need a
  subtitle probe/sidecar to be rejected.
- HTTP route test proving a browse-only subtitle request returns `403` with
  the same public Library Access message and no subtitle policy or sidecar
  detail.
- HTTP route test proving a previously issued subtitle browser ticket is
  rejected after source `Play` access is revoked.
- Existing subtitle route and browser-ticket tests must continue covering
  sidecar subtitle text, content type, sidecar locator redaction, bearer bypass
  with valid tickets, and wrong-stream ticket rejection.
- Focused gates:
  `cargo nextest run -p nako-server subtitle_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`,
  `cargo nextest run -p nako-server subtitle_route_requires_play_library_access --no-fail-fast`,
  `cargo nextest run -p nako-server subtitle_browser_playback_ticket_rejects_revocation_at_use --no-fail-fast`,
  `cargo nextest run -p nako-server subtitle --no-fail-fast`, and
  `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let principal = resolve_subtitle_playback_principal(
    &app,
    principal,
    source_id,
    stream_index,
    ticket.as_deref(),
)
.await?;
require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
```

This keeps subtitle source `Play` access route-local and leaves future
app-service callers able to bypass the subtitle sidecar access boundary.

#### Correct

```rust
let principal = resolve_subtitle_playback_principal(
    &app,
    principal,
    source_id,
    stream_index,
    ticket.as_deref(),
)
.await?;
app.playback()
    .subtitle_playback(SubtitlePlaybackRequest {
        principal,
        source_id,
        stream_index,
    })
    .await?;
```

The route owns auth/ticket resolution and subtitle response mechanics, while
`PlaybackAppService` owns source `Play` access before subtitle policy, probe,
sidecar, or storage work.

## Scenario: Renderer Transport Access Boundary

### 1. Scope / Trigger

- Trigger: changing renderer transport ticket use on Direct, Remux, HLS
  playlist, or HLS segment playback routes; changing
  `resolve_renderer_transport_principal` or
  `resolve_renderer_transport_principal_for_session` in
  `crates/nako-server/src/http/playback.rs`.
- Code evidence: `src/http/playback.rs`, `src/http/renderer.rs`,
  `src/app/playback/mod.rs`, `src/app/playback/remux_flow.rs`,
  `src/app/playback/hls_flow.rs`, `src/http/tests/renderer.rs`.

### 2. Signatures

- Renderer transport HTTP query fields are:
  - `renderer_session_id: Option<String>`
  - `playback_session_id: Option<String>`
  - `renderer_ticket: Option<String>`
- `resolve_renderer_transport_principal(...)` returns
  `Option<ResolvedRendererTransport>`.
- `ResolvedRendererTransport` carries:
  - `principal: AuthenticatedPrincipal`
  - `renderer_session_id: RendererSessionId`
  - `playback_session_id: PlaybackSessionId`

### 3. Contracts

- Renderer transport resolvers parse playback-session and renderer-session
  IDs, validate renderer ticket scope, validate renderer owner identity, and
  validate renderer network scope.
- Renderer transport resolvers must not call route-local source `Play` access
  helpers. They are auth/ticket-only resolvers.
- Direct renderer transport use must call
  `PlaybackAppService::direct_playback_session_stream` or
  `PlaybackAppService::direct_playback_session_preflight`.
- Remux renderer transport use must call
  `PlaybackAppService::remux_playback_session_stream`.
- HLS renderer playlist use must call
  `PlaybackAppService::hls_playlist_for_playback_session`.
- HLS renderer segment use must call
  `PlaybackAppService::hls_segment_playback` after resolving the session
  target.
- The playback app-service session-use method must enforce current source
  `Play` Library Access before playback session reuse, lazy artifact startup,
  segment planning, byte serving, or playlist response shaping.
- Renderer transport ticket issue, renderer command payloads, transport URL
  authoring, public DTOs, and invalid-ticket response behavior remain owned by
  their existing HTTP/app-service boundaries.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Renderer transport ticket was issued, then source Play access is revoked before Direct use | Ticket-backed Direct transport returns `403` with required Library Access level `play` |
| Renderer transport ticket was issued, then source Play access is revoked before Remux use | Ticket-backed Remux transport returns `403` with required Library Access level `play` |
| Renderer transport ticket was issued, then source Play access is revoked before HLS playlist use | Ticket-backed HLS playlist transport returns `403` with required Library Access level `play` |
| Renderer transport ticket was issued, playlist generated, then source Play access is revoked before segment use | Ticket-backed HLS segment transport returns `403` with required Library Access level `play` |
| Renderer ticket is missing, empty, malformed, wrong session, wrong renderer, wrong mode, wrong source, wrong network scope, or non-owner principal | Preserve existing unauthorized/forbidden renderer transport behavior |
| Renderer play command is requested by a Browse-only principal | Preserve app-service renderer play command source `Play` denial before runtime records |
| Renderer play command is requested by a Play principal without `remote_control` | Preserve playback-policy denial mentioning `remote_control` |

### 5. Good / Base / Bad Cases

- Good: renderer transport resolver validates only ticket, owner, scope, and
  identity, then route code calls the appropriate playback app-service
  session-use method.
- Good: revoked renderer transport use returns the standard Library Access
  `play` forbidden message from the app-service method.
- Base: renderer transport URL authoring remains in `http/renderer.rs`
  because it is a public response assembly concern.
- Bad: resolver calls a source access helper before returning
  `ResolvedRendererTransport`, because that makes HTTP the access authority
  and can drift from app-service use paths.
- Bad: renderer transport route serves bytes, starts lazy Remux/HLS artifacts,
  or plans HLS segments before the app-service source `Play` recheck.

### 6. Tests Required

- HTTP renderer test proving Direct renderer transport use returns `403` with
  the standard Library Access `play` message after source access is revoked.
- HTTP renderer test proving Remux renderer transport use returns `403` with
  the same message after source access is revoked.
- HTTP renderer test proving HLS renderer playlist transport use returns
  `403` with the same message after source access is revoked.
- Existing renderer transport tests must continue covering Remux success, HLS
  playlist/segment success, Direct external adapter transport, invalid
  renderer ticket rejection, URL redaction, and command payload redaction.
- Focused gates:
  `cargo nextest run -p nako-server renderer_transport_direct_rejects_revoked_source_play_access_at_use --no-fail-fast`,
  `cargo nextest run -p nako-server renderer_transport_remux_rejects_revoked_source_play_access_at_use --no-fail-fast`,
  `cargo nextest run -p nako-server renderer_transport_hls_rejects_revoked_source_play_access_at_playlist_use --no-fail-fast`,
  `cargo nextest run -p nako-server renderer_play_command_with_cast_ticket --no-fail-fast`,
  `cargo nextest run -p nako-server synthetic_external_adapter_play_command_receives_cast_safe_transport_envelope --no-fail-fast`, and
  `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let validated = app.renderer_transport_tickets().validate(request)?;
require_source_access(&app, &validated.principal, source_id, RequiredLibraryAccess::Play).await?;
Ok(Some(ResolvedRendererTransport { principal: validated.principal, ... }))
```

This makes renderer transport source access route-local and duplicates the
session-use access checks owned by `PlaybackAppService`.

#### Correct

```rust
let validated = app.renderer_transport_tickets().validate(request)?;
Ok(Some(ResolvedRendererTransport {
    principal: validated.principal,
    renderer_session_id,
    playback_session_id,
}))
```

The route validates renderer transport identity, then the selected playback
app-service session-use method enforces current source `Play` access before
serving transport bytes or playlists.

## Scenario: Browser Playback Ticket Access Boundary

### 1. Scope / Trigger

- Trigger: changing Public Client browser playback ticket issuing,
  `PlaybackAppService::validate_browser_playback_ticket_request`, browser
  ticket mode validation, or route-local playback ticket source access checks
  in `crates/nako-server`.
- Code evidence: `src/http/playback.rs`, `src/app/playback/mod.rs`,
  `src/http/tests/playback.rs`, `src/app/tests/playback.rs`.

### 2. Signatures

- Browser ticket HTTP handlers take `Extension(AuthenticatedPrincipal)` and
  pass the full principal into `PlaybackAppService`.
- `PlaybackAppService::validate_browser_playback_ticket_request(BrowserPlaybackTicketValidationRequest)`
  carries:
  - `principal: AuthenticatedPrincipal`
  - `source_id: MediaSourceId`
  - `mode: BrowserPlaybackTicketMode`
  - `subtitle_stream_index: Option<u32>`

### 3. Contracts

- HTTP browser ticket issuing parses the public ticket request, normalizes
  client capabilities, delegates source/mode validation to the app service, and
  signs a browser playback ticket only after validation succeeds.
- Browser ticket issuing must enforce source `Play` Library Access in
  `PlaybackAppService::validate_browser_playback_ticket_request`, not through
  route-local `require_source_access`.
- Source `Play` Library Access denial must happen before playback-policy
  details such as `direct_play`, `remux`, `video_transcode`,
  `audio_transcode`, or `media_playback` are exposed.
- Mode-specific playback-policy checks, remote playback checks, subtitle stream
  validation, playback session creation, ticket signing, and URL construction
  keep their existing app-service or route ownership.
- Direct Play, Remux, HLS, sidecar subtitle playback, and renderer transport
  use paths already recheck current source `Play` access through
  `PlaybackAppService`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Ordinary principal has Browse but not Play access for the source library | Browser ticket issue returns `403` with required Library Access level `play` |
| Ordinary principal has Play access but requested mode is disabled by playback policy | Browser ticket issue returns `403` with the mode-specific playback-policy denial |
| Subtitle ticket omits `subtitle_stream_index` after source Play access passes | Preserve `NakoError::InvalidInput` for missing subtitle stream index |
| Subtitle ticket references a missing subtitle stream after source Play access passes | Preserve subtitle stream not-found behavior |
| Unknown source ID | Preserve `NakoError::NotFound` for `media_source` |
| Administrator issues a browser ticket | Preserve administrator access and playback-policy semantics |

### 5. Good / Base / Bad Cases

- Good: `/sources/{source_id}/playback/browser-ticket` parses the public body
  and calls
  `app.playback().validate_browser_playback_ticket_request(BrowserPlaybackTicketValidationRequest { principal, ... })`
  before ticket signing.
- Base: already-issued browser tickets are still rechecked on media byte routes
  when used, so access revocation after issue remains effective.
- Bad: route code calls `require_source_access(... Play)` before validation,
  because non-HTTP callers of the app service could still bypass browser ticket
  source access.
- Bad: playback-policy checks run before source `Play` Library Access, because
  browse-only users would learn policy details for media they cannot play.

### 6. Tests Required

- App-service test proving a browse-only principal cannot validate a browser
  playback ticket request and receives the standard Library Access `play`
  message before playback-policy details.
- HTTP route test proving a browse-only principal cannot issue a browser
  playback ticket and receives `403` with the same public message.
- Existing mode-specific playback-policy denial tests must continue proving a
  principal with source Play access receives the playback-policy denial.
- Focused gates:
  `cargo nextest run -p nako-server browser_ticket --no-fail-fast` and
  `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
app.playback()
    .validate_browser_playback_ticket_request(request)
    .await?;
```

This keeps HTTP as the source access authority and leaves future app-service
callers able to bypass browser ticket source access.

#### Correct

```rust
app.playback()
    .validate_browser_playback_ticket_request(BrowserPlaybackTicketValidationRequest {
        principal,
        source_id,
        mode,
        subtitle_stream_index,
    })
    .await?;
```

The app-service validation owns source `Play` access first, then mode-specific
playback-policy checks, while HTTP remains the public request and ticket
response boundary.

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
- Good: do the same at the public/Admin library scan enqueue boundary so queued
  `disk.scan` jobs carry only the normalized safe `request_id`.
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
- App/route test: public and Admin library scan POST routes persist only the
  normalized safe `request_id` inside queued `disk.scan` job input.

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
- `selected_image_preflight_response(...) -> Option<Response>` owns the
  metadata-derived conditional 304 short-circuit after auth and library access
  checks.
- `ManagedArtworkAppService::selected_image_access(&AuthenticatedPrincipal, SelectedArtworkId)`
  owns public selected artwork Browse access enforcement and returns the
  app-service access context used by preflight and byte reads.
- `ManagedArtworkAppService::selected_image_preflight(&SelectedArtworkImageAccess, ImageVariantRequest)`
  owns metadata-derived safe ETag lookup for an already-authorized selected
  artwork image request.
- `ManagedArtworkAppService::read_selected_image(&SelectedArtworkImageAccess, ImageVariantRequest)`
  owns selected artwork byte loading for an already-authorized selected artwork
  image request.
- `apply_selected_artwork_cache_headers(&mut HeaderMap)` is the selected
  artwork-only helper for the private client-cache baseline.
- `selected_image_etag_matches(if_none_match, etag) -> bool` is the route-local
  selected artwork validator guard for conditional responses.

### 3. Contracts

- Selected artwork image GET responses must include
  `Cache-Control: private, max-age=86400`.
- Selected artwork image HEAD responses must include the same cache policy and
  must not include a response body.
- Selected artwork image GET/HEAD responses with a matching `If-None-Match`
  value must return `304 Not Modified`. Matching supports exact quoted tags,
  weak `W/"etag"` tags, comma-separated validator lists, and wildcard `*`.
- A selected artwork 304 response must include the current safe `ETag` and
  `Cache-Control: private, max-age=86400`, and must not include a response
  body.
- Keep this policy selected-artwork-only. Do not apply it to HLS, Direct Play,
  Remux, Admin JSON routes, or unrelated public JSON catalog routes.
- Preserve existing `Content-Type`, `Content-Length`, safe `ETag`, auth,
  library access, selected artwork lookup, and variant query behavior.
- Auth and library access checks must run before any selected artwork 304
  response.
- Selected artwork Browse access belongs to `ManagedArtworkAppService`, not
  `http::access`; HTTP handlers pass the authenticated principal into
  `selected_image_access` and then pass the returned access context into
  preflight/read calls.
- Access checks must still happen before variant query validation so
  unauthorized selected artwork callers receive `403` instead of image variant
  validation details.
- Metadata-derived ETag preflight may short-circuit a matching `If-None-Match`
  match before bytes are read, but only after auth and library access checks
  and only when it proves the same safe ETag that a normal response would
  emit.
- App services must not parse HTTP `If-None-Match`, own `HeaderMap`, or build
  HTTP selected artwork responses. HTTP owns conditional request parsing and
  response header assembly.
- Do not add `Last-Modified`, immutable headers, generated DTOs, schema
  changes, or shared-cache/CDN behavior without a dedicated cache-contract
  task.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Original selected artwork GET response is authored | Includes `Cache-Control: private, max-age=86400` plus existing content headers and safe ETag. |
| Original selected artwork HEAD response is authored | Includes the same cache policy, content headers, and safe ETag with an empty body. |
| Resized selected artwork variant GET/HEAD response is authored | Includes the same cache policy while preserving variant-specific content length and ETag. |
| `If-None-Match` exactly or weakly matches the current original or variant ETag | Returns `304 Not Modified` with current ETag, selected artwork cache policy, and empty body. |
| `If-None-Match` is a comma-separated list containing the current ETag | Returns `304 Not Modified` with current ETag, selected artwork cache policy, and empty body. |
| `If-None-Match` is `*` and the selected artwork exists | Returns `304 Not Modified` with current ETag, selected artwork cache policy, and empty body. |
| `If-None-Match` is missing, malformed, or does not match | Existing `200` GET/HEAD response behavior is unchanged. |
| Selected artwork is missing or unauthorized | Existing not-found/forbidden behavior is unchanged. |
| Unauthorized selected artwork GET/HEAD includes matching or wildcard `If-None-Match` | Returns `403` before any `304` response. |
| Unauthorized selected artwork GET/HEAD includes an invalid variant query | Returns `403` before variant validation details. |
| Variant query is invalid | Existing bad-request behavior is unchanged. |

### 5. Good / Base / Bad Cases

- Good: call `apply_selected_artwork_cache_headers` only from
  `selected_image_response`, which is shared by the selected artwork GET and
  HEAD handlers.
- Good: compare `If-None-Match` against the same quoted ETag `HeaderValue` that
  will be returned on a normal selected artwork response, so matching cannot
  drift from header authoring.
- Good: parse only selected artwork request validators locally, recognizing
  weak tags, comma-separated lists, and wildcard `*` without exposing raw
  source/artifact ETags.
- Good: `get_image`/`head_image` call
  `app.artwork().selected_image_access(&principal, image_id)` before variant
  parsing, then pass the returned access context to preflight/read helpers.
- Base: safe selected artwork ETags continue to identify original versus
  bounded variants; the cache helper does not change ETag generation.
- Base: matching 304 short-circuiting can happen before bytes are read when
  the route can prove the same safe ETag from selected artwork and artifact
  metadata.
- Bad: reusing the HLS `no-store` helper for selected artwork, which defeats
  client artwork caching.
- Bad: applying `private, max-age=86400` through a generic byte-route helper
  that changes HLS, Direct Play, Remux, or Admin response behavior.
- Bad: returning 304 before auth/library access checks or matching against raw
  user-provided ETag strings instead of the route-authored safe ETag header.
- Bad: calling `require_selected_artwork_access` from `http::catalog` or
  passing `HeaderMap`/`If-None-Match` into the artwork app service.

### 6. Tests Required

- HTTP route test: original selected artwork GET response includes
  `Cache-Control: private, max-age=86400`.
- HTTP route test: original selected artwork HEAD response includes the same
  cache policy and an empty body.
- HTTP route test: resized selected artwork GET/HEAD responses include the same
  cache policy while preserving variant-specific ETags.
- HTTP route test: exact, weak, list, and wildcard matching `If-None-Match`
  values return `304 Not Modified` with the current ETag/cache headers and no
  body.
- HTTP route test: non-matching `If-None-Match` preserves normal `200` image
  response behavior.
- HTTP route test: unauthorized selected artwork GET and HEAD return `403`
  even with matching or wildcard `If-None-Match` headers.
- HTTP route test: unauthorized selected artwork requests return `403` before
  invalid variant query details.
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
