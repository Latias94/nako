# Quality Guidelines

API contract work must keep generated artifacts and route inventories honest.

## Required Patterns

- Update DTO source first, generator second, generated artifacts last.
- Regenerate generated Admin Web contract files from `nako-api`; do not edit
  generated TypeScript directly.
- Keep Admin `/admin/v1/*` routes out of Public Client/OpenAPI/SDK outputs
  unless the task explicitly changes the public contract.
- Keep Public Client playback capability fields in sync across protocol,
  OpenAPI, SDK/client builders, server query mapping, and HTTP docs.
- Add tests for route inventory, DTO generation, redaction, and Public/Admin
  separation.
- Use snake_case serde wire fields to match existing contracts.

## Gate Selection

- Focused admin contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- Public/OpenAPI/SDK contract:
  `cargo nextest run -p nako-api --no-fail-fast`
- Admin Web contract refresh:
  `npm run generate:admin-api --prefix apps/admin-web`
- Cross-crate API/server:
  `cargo check -p nako-api -p nako-server --tests`

## Forbidden Patterns

- Do not expose internal database/domain records directly as wire responses.
- Do not add route strings in Admin Web instead of the generated contract.
- Do not make Public Client contracts depend on Admin-only concepts.
- Do not put FFmpeg, hardware, resource pressure, or operator policy facts into
  Public Client playback capability DTOs.
- Do not add a DTO field before deciding redaction and audience.

## Review Checklist

- Does this route belong to Admin API or Public Client API?
- Is the generated output updated by the generator?
- Are route inventory tests updated?
- Are sensitive fields redacted or omitted?
- For Public Client playback capabilities, are protocol DTOs, OpenAPI,
  generated/client builders, server query mapping, and HTTP docs still aligned?

## Scenario: Admin Route Inventory Parity Gate

### 1. Scope / Trigger

- Trigger: adding, renaming, or deleting an implemented `/admin/v1/*` server
  route in `crates/nako-server/src/http/admin.rs` or
  `crates/nako-server/src/http/addons.rs`, or changing generated Admin route
  constants in `crates/nako-api/src/admin_contract.rs`.
- Purpose: prevent silent drift between implemented Admin HTTP routes, generated
  Admin Web route constants, and intentionally server-only Admin routes.

### 2. Signatures

- Generated routes:
  `admin_contract_routes() -> Vec<AdminContractRoute>`.
- Explicit exclusions:
  `admin_contract_route_exclusions() -> Vec<AdminContractRouteExclusion>`.
- Path normalization:
  `normalize_admin_route_path(path: &str) -> String`, converting Axum
  `:param` syntax into generated `{param}` syntax before comparison.
- Server route inventory test:
  `http::tests::admin_route_inventory::implemented_admin_routes_are_generated_or_explicitly_excluded`.

### 3. Contracts

- Every implemented `/admin/v1/*` literal route in the Admin HTTP and Addon
  Admin route modules must be either generated in `NAKO_ADMIN_ROUTES` or listed
  in `admin_contract_route_exclusions()` with a non-empty reason.
- Generated Admin route constants must map back to implemented server routes.
- Exclusions are for implemented Admin routes only; stale exclusions must fail
  the gate.
- Generated and excluded Admin routes must remain outside Public Client route
  inventories and generated Public SDK output.
- Do not satisfy the gate by adding Public Client routes or by hand-editing
  generated Admin Web TypeScript artifacts.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| New implemented `/admin/v1/*` route is Admin Web-facing | Add it to `ADMIN_ROUTE_SUFFIXES`, regenerate generated Admin Web contracts if output changes, and keep the route implemented server-side |
| New implemented `/admin/v1/*` route is server/operator-only for now | Add an explicit `AdminContractRouteExclusion` suffix and reason |
| Generated route has no implemented server route | `implemented_admin_routes_are_generated_or_explicitly_excluded` fails |
| Implemented route is neither generated nor excluded | `implemented_admin_routes_are_generated_or_explicitly_excluded` fails |
| Exclusion points to a removed server route | `implemented_admin_routes_are_generated_or_explicitly_excluded` fails |
| Axum uses `:id` while generated route uses `{id}` | `normalize_admin_route_path` treats them as the same path |
| Public Client inventory contains an Admin path | `admin_contract_routes_stay_out_of_public_client_inventory` fails |

### 5. Good/Base/Bad Cases

- Good: a new Admin Web route is added to the server route module and to
  `ADMIN_ROUTE_SUFFIXES`, then generated contracts are refreshed through the
  generator.
- Good: a server-only maintenance route is added to the server route module and
  to `admin_contract_route_exclusions()` with a reason explaining why it is not
  generated in this slice.
- Base: existing generated Admin routes and explicit exclusions continue to
  cover all implemented Admin HTTP and Addon Admin routes.
- Bad: adding a server route and relying on Admin Web string literals or mock
  fallback data instead of generated `NAKO_ADMIN_ROUTES`.
- Bad: adding an Admin route to the Public Client route inventory to make a
  parity test pass.

### 6. Tests Required

- Focused API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Focused server inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Cross-crate compile when route inventory helpers or server route tests change:
  `cargo check -p nako-api --tests` and
  `cargo check -p nako-server --tests`.
- Formatting and whitespace:
  `cargo fmt --all -- --check` and `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```rust
Router::new().route("/admin/v1/example", get(handler))
```

Adding the route without updating the generated Admin route inventory or an
explicit exclusion leaves Admin Web and server contracts free to drift.

#### Correct

```rust
const ADMIN_ROUTE_SUFFIXES: [(&str, &str); N] = [
    ("example", "example"),
];
```

Generate Admin Web contracts from `nako-api` when the generated output changes.
If the route is intentionally server-only for the slice, add it to
`admin_contract_route_exclusions()` with a specific reason instead.

## Scenario: Addon Event Delivery Admin DTO Redaction

### 1. Scope / Trigger

- Trigger: changing Admin Addon Event Delivery attempts, scheduler work,
  deliver, or replay response DTOs, generated Admin contract output, or server
  mapping for `/admin/v1/events/{event_id}/addon-*` routes.
- Purpose: keep raw Addon event payloads, stored delivery errors, dispatch
  errors, sidecar URLs, tokens, paths, and fingerprints out of Admin wire
  contracts.

### 2. Signatures

- Attempts response:
  `AddonEventDeliveryAttemptsResponse { event_id, attempts:
  Vec<AddonEventDeliveryAttemptSummary> }`.
- Attempt summary:
  `AddonEventDeliveryAttemptSummary { id, addon_id, event_id, declaration_id,
  attempt_number, status, http_status, has_error, requested_at, completed_at,
  next_retry_at, lease_expires_at, forced_replay, replay_reason_code }`.
- Dispatch response:
  `AddonEventDispatchResponse { event, attempted_subscriptions, delivered,
  failed, skipped_subscriptions, attempts, error_count }`.
- Replay response:
  `AddonEventReplayResponse { reason_code, dispatch }`.

### 3. Contracts

- `nako-core::AddonEventDeliveryAttemptRecord.error` is persistence/internal
  retry diagnostic material. Do not expose it directly in Admin DTOs.
- Admin delivery attempt DTOs expose `has_error: bool`, not `error:
  Option<String>`.
- Admin dispatch DTOs expose `error_count: u32`, not `errors: Vec<String>`.
- `AddonEventDispatchEventSummary.subject` remains part of the wire event
  summary for server/API callers, but Admin Web route projections must not
  render it unless a future task defines a redaction-safe subject summary.
- Generated TypeScript contracts under `apps/admin-web` and `web` must be
  regenerated from `nako-api`; do not hand-edit generated files.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Stored attempt has raw `error` text | Response summary sets `has_error: true` and omits the text |
| Attempt has no stored error | Response summary sets `has_error: false` |
| Dispatch fails before attempt completion | Increment `error_count`; log server-side; do not return raw string |
| Dispatch worker join fails | Increment `error_count`; log server-side; do not return raw join error |
| Generated contract contains `error: string`, `errors: string[]`, path, token, URL, or fingerprint fields for this boundary | Treat as contract failure |

### 5. Good/Base/Bad Cases

- Good: map `AddonEventDeliveryAttemptRecord` into
  `AddonEventDeliveryAttemptSummary` at the app/API boundary.
- Base: scheduler work exposes safe reason codes and routing plan status/target
  only.
- Bad: returning `Vec<AddonEventDeliveryAttemptRecord>` or raw dispatch
  `errors` from Admin API responses.

### 6. Tests Required

- API contract tests prove generated TypeScript contains `has_error` and
  `error_count`, and does not contain raw `error/errors` fields for Addon Event
  Delivery DTOs.
- Server route tests seed failed delivery attempts with sensitive error
  material and assert responses omit the raw strings while retaining
  `has_error`/`error_count`.
- Admin Web data-source and route tests assert generated raw material is not
  rendered.
- Run:
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server addon_event --no-fail-fast`
  - `npm run check --prefix apps/admin-web`

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct AddonEventDispatchResponse {
    pub attempts: Vec<AddonEventDeliveryAttemptRecord>,
    pub errors: Vec<String>,
}
```

#### Correct

```rust
pub struct AddonEventDispatchResponse {
    pub attempts: Vec<AddonEventDeliveryAttemptSummary>,
    pub error_count: u32,
}
```

Admin API contracts should expose operator-safe facts and counts. Raw delivery
diagnostics stay in storage/logs unless a dedicated safe diagnostic DTO is
designed.

## Scenario: Public Client Playback Capability Contract Parity

### 1. Scope / Trigger

- Trigger: adding, renaming, deleting, or changing a Public Client playback
  capability field, HLS playback capability query parameter, browser playback
  ticket capability field, renderer media capability field, or generated client
  builder that carries these fields.
- Scope:
  `nako-client-protocol` playback DTOs, `nako-api` OpenAPI/SDK generation,
  `nako-client`, `nako-client-core`, server playback/renderer HTTP mapping,
  generated SDK query surfaces, and `docs/api/HTTP_API.md`.

### 2. Signatures

- Browser ticket body:
  `BrowserPlaybackTicketRequest { capabilities:
  Option<BrowserPlaybackCapabilitiesDto> }`.
- Flat v1 Public Client playback capability query/body fields:
  `direct_play`, `container`, `video_codec`, `audio_codec`,
  `max_video_bitrate`, `max_width`, `max_height`, `max_audio_channels`,
  `supports_hdr`, `supports_subtitles`, `hls_variant_policy`, and
  `hls_segment_container`.
- Remux stream query and browser ticket remux planning may additionally carry
  `output_container`.
- Renderer body:
  `RendererRegistrationRequest.media_capabilities` and
  `RendererHeartbeatRequest.media_capabilities`.
- Public client capability response/session shape:
  `ClientPlaybackCapabilitiesDto`, where `container`, `video_codec`, and
  `audio_codec` are represented as response/body fields `containers`,
  `video_codecs`, and `audio_codecs`.
- Server query mapping:
  `PlaybackCapabilitiesQuery -> ClientPlaybackCapabilities`.
- Server browser body mapping:
  `BrowserPlaybackCapabilitiesDto -> ClientPlaybackCapabilities`.
- Server renderer body mapping:
  `ClientPlaybackCapabilitiesDto -> ClientPlaybackCapabilities`.

### 3. Contracts

- `nako-client-protocol` owns the Public Client wire DTOs. `nako-api` maps
  domain decisions into those DTOs and emits OpenAPI/SDK artifacts from them.
- Current flat capability fields must remain aligned across protocol DTOs,
  OpenAPI schemas/query parameters, Rust client query builders,
  `nako-client-core`, generated SDK query surfaces, server query/body mapping,
  and HTTP API docs.
- Query wire values are singular snake_case request-preference names. Renderer
  and session capability DTO collection fields are pluralized because they
  describe accepted capability sets.
- Public Client capability DTOs describe client/player facts and request
  preferences only. They must not expose Admin-only diagnostics, FFmpeg command
  facts, hardware probe facts, GPU/device paths, resource pressure, or operator
  fallback policy.
- New output/device profile fields must be additive unless the task explicitly
  performs a versioned breaking contract change.
- If a new capability field changes playback planning output, the corresponding
  `nako-playback` profile identity and planner tests must be updated.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| New Public playback capability field is added | Update protocol DTOs, OpenAPI schema/query params, Rust client/client-core builders, generated SDK surfaces, server mapping, docs, and tests |
| Field only belongs to Admin diagnostics | Keep it out of Public Client DTOs and expose it through Admin DTOs with redaction tests |
| Field changes planner output | Include it in playback profile identity and add planner tests |
| Field is optional additive profile data | Preserve existing flat-field behavior when it is absent |
| Client/core/UniFFI/Kotlin query builder omits a supported flat field | Builder round-trip tests must fail until the query renderer is updated |
| Renderer or browser ticket body omits a supported flat field | Server private mapping tests must fail until body mapping is updated |
| Generated contract drift | Focused `nako-api` contract/OpenAPI/SDK tests must fail until regenerated or updated |
| HTTP docs omit a supported query/body field | Treat as contract drift and update `docs/api/HTTP_API.md` |

### 5. Good/Base/Bad Cases

- Good: adding a `device_family` field updates protocol DTOs, OpenAPI schemas,
  client query/body builders, server mapping, playback profile identity tests,
  SDK output, and HTTP docs in the same task.
- Good: adding Admin support evidence for selected acceleration keeps the field
  in Admin DTOs and does not add it to `ClientPlaybackCapabilitiesDto`.
- Base: current flat fields such as `max_video_bitrate`, `supports_hdr`,
  `hls_variant_policy`, and `hls_segment_container` are consistently available
  in every supported Public Client query/body builder.
- Bad: server accepts a new query parameter that no generated client or docs can
  send.
- Bad: Public Client DTOs expose FFmpeg encoder names, GPU device paths, or
  operator hardware fallback policy.

### 6. Tests Required

- API/OpenAPI/SDK contract tests:
  `cargo nextest run -p nako-api --no-fail-fast`.
- Protocol field-set gate:
  `cargo nextest run -p nako-client-protocol public_playback_capability_dtos_keep_current_flat_field_contract --no-fail-fast`.
- Rust client/client-core tests when query/body builders change:
  `cargo nextest run -p nako-client -p nako-client-core --no-fail-fast`.
- UniFFI mirror tests when `CorePlaybackCapabilities` changes:
  `cargo nextest run -p nako-client-uniffi --no-fail-fast`.
- Server route tests for query/body mapping when HTTP handlers change:
  focused `nako-server` playback or renderer route tests.
- Playback planner tests when new capability facts can affect decision output
  or profile identity.
- Generated SDK/doc checks appropriate to the changed generated artifacts.
  Kotlin generated output must be refreshed with
  `cargo run -q -p nako-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`.
- Formatting and whitespace:
  `cargo fmt --all -- --check` and `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct ClientPlaybackCapabilitiesDto {
    pub hardware_acceleration: Option<String>,
    pub ffmpeg_encoder: Option<String>,
}
```

This turns Public Client capability into a host/runtime diagnostic surface and
mixes client facts with operator policy.

#### Correct

```rust
pub struct ClientPlaybackCapabilitiesDto {
    pub device_family: Option<ClientPlaybackDeviceFamily>,
    pub profile_version: Option<u32>,
}
```

Client profile fields describe what the player can do. Server hardware,
fallback, and stage readiness remain in Admin diagnostics and transcode runtime
records.

## Scenario: Admin Diagnostic Summary DTO

### 1. Scope / Trigger

- Trigger: adding or changing an Admin `/admin/v1/*` diagnostic response,
  especially a nested summary object derived from storage, VFS, jobs, playback,
  provider, addon, or database runtime state.
- Apply this when a field is operator-facing but derived from records that may
  contain paths, Source Locators, Source Fingerprints, etags, tokens, provider
  payloads, raw errors, or other host-sensitive values.

### 2. Signatures

- Rust DTO source lives under `crates/nako-api/src/admin/*.rs`.
- The TypeScript contract source is
  `crates/nako-api/src/admin_contract.rs`.
- Generated Admin contracts are:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Generate from the Rust source with:
  `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output <path>`.

### 3. Contracts

- Summary DTO fields must be typed counts, booleans, timestamps, percentages,
  redacted enums, or explicitly safe messages.
- Use named DTOs for reusable nested summaries instead of anonymous frontend-only
  shapes when the Rust DTO has a named struct.
- Do not expose raw identifiers whose value is a sensitive locator or backend
  reference. For storage diagnostics, expose `source_scheme`, `has_etag`,
  `has_fingerprint`, counts, and safe `StorageFailureClass` values instead of
  Source Locator, etag, Source Fingerprint, local path, or raw backend error
  strings.
- Admin-only diagnostic DTO changes must not appear in Public Client/OpenAPI/SDK
  outputs unless the task explicitly changes the public contract.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| DTO shape changes | Update Rust DTO, `admin_contract.rs`, both generated TypeScript contracts, and mock/fixture objects. |
| Diagnostic source records contain paths or Source Locators | Response omits raw values and exposes only safe counts, schemes, booleans, or safe classes. |
| Diagnostic records include etags or Source Fingerprints | Response exposes only presence booleans or aggregate counts. |
| Diagnostic records include raw backend/provider errors | Response omits raw text or maps through a safe message/class. |
| Generated contract drift | `cargo nextest run -p nako-api admin_contract --no-fail-fast` must fail until regenerated. |

### 5. Good/Base/Bad Cases

- Good: an Admin storage staging summary exposes
  `pressure.status`, `used_ratio_milli`, and record counts while route tests seed
  sensitive manifest rows and assert the response body does not contain paths,
  tokens, etags, Source Fingerprints, or raw backend errors.
- Base: a read-only diagnostic response adds a boolean such as
  `has_validation_error` and updates generated contracts plus mock fixtures.
- Bad: returning a repository record or adding `source_uri`, `local_path`,
  `etag`, `fingerprint`, or raw `validation_error` to an Admin diagnostic DTO.

### 6. Tests Required

- API contract: `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route: add a focused route test that deserializes the response DTO and
  asserts both the new fields and redaction against sensitive fixture values.
- Frontend contract consumers: run the relevant TypeScript check/test for every
  generated contract copy or fixture touched.
- Cross-crate compile: `cargo check -p nako-api -p nako-server --tests`; broaden
  with storage/library/db packages when the diagnostic is derived from those
  crates.

### 7. Wrong vs Correct

#### Wrong

```rust
pub struct AdminStorageDiagnostic {
    pub source_uri: String,
    pub local_path: String,
    pub fingerprint: Option<String>,
    pub last_error: Option<String>,
}
```

#### Correct

```rust
pub struct AdminStorageDiagnostic {
    pub source_scheme: String,
    pub has_fingerprint: bool,
    pub failed_records: u32,
    pub last_failure_class: Option<StorageFailureClass>,
}
```

The correct DTO gives operators useful pressure and failure signals without
leaking Source Locators, Source Fingerprints, local paths, or raw backend
payloads.

## Scenario: Managed Artwork Maintenance Read-Only Admin Contract

### 1. Scope / Trigger

- Trigger: changing Admin Managed Artwork lifecycle, storage drift, remediation
  plan DTOs, generated Admin route constants, or route exclusions for
  `/admin/v1/artwork/*` maintenance routes.
- Scope:
  `GET /admin/v1/artwork/artifacts/lifecycle`,
  `GET /admin/v1/artwork/artifacts/storage-drift`,
  `GET /admin/v1/artwork/artifacts/remediation-plan`,
  `crates/nako-api/src/admin/managed_artwork.rs`,
  `crates/nako-api/src/admin_contract.rs`, and generated Admin Web contracts.

### 2. Signatures

- Generated route keys:
  `managedArtworkArtifactLifecycle`,
  `managedArtworkArtifactStorageDrift`, and
  `managedArtworkArtifactRemediationPlan`.
- Lifecycle DTOs:
  `AdminManagedArtworkArtifactLifecycleResponse`,
  `AdminManagedArtworkArtifactLifecycleSummary`, and
  `AdminManagedArtworkArtifactLifecycleItem`.
- Storage drift DTOs:
  `AdminManagedArtworkArtifactStorageDriftResponse`,
  `AdminManagedArtworkArtifactStorageDriftSummary`,
  `AdminManagedArtworkArtifactStorageDriftArtifact`, and
  `AdminManagedArtworkArtifactStorageDriftFile`.
- Remediation DTOs:
  `AdminManagedArtworkArtifactRemediationPlanResponse`,
  `AdminManagedArtworkArtifactRemediationSummary`,
  `AdminManagedArtworkArtifactRemediationMissingArtifact`, and
  `AdminManagedArtworkArtifactRemediationStrayFile`.

### 3. Contracts

- These three routes are read-only diagnostics and belong in the generated
  Admin Web contract.
- The worker execution route `artwork/ingests/process-next` remains an explicit
  exclusion until a separate runtime-control policy task defines a stable
  Admin Web-facing command boundary. Candidate accept, ingest requeue, artifact
  publish, artifact cleanup, stray-file remediation, and addon install-guide
  preview have separate generated contracts and should not be re-added to the
  exclusion list.
- DTOs may expose counts, booleans, safe enum codes, stable artifact/ingest/item
  IDs, dimensions, byte counts, media type, and timestamps.
- DTOs and generated TypeScript must not expose raw file names, local paths,
  artifact roots, `storage_uri`, `managed-artwork://` handles, `source_uri`,
  `cache_uri`, provider URLs/query strings, tokens, credentials, content hashes,
  etags, or raw backend payloads.
- Generated contract artifacts under `apps/admin-web` and `web` must be
  regenerated from `nako-api`; do not hand-edit generated TypeScript output.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Read-only Managed Artwork diagnostic route becomes Admin Web-facing | Add it to `ADMIN_ROUTE_SUFFIXES`, add DTO TypeScript body, and regenerate contracts |
| Mutating Managed Artwork route remains server/operator-only | Keep an explicit exclusion with a specific reason |
| Confirmed Managed Artwork mutation becomes Admin Web-facing | Add a dedicated generated mutation contract with route key, query/request semantics, response DTOs, and redaction tests |
| Candidate accept becomes Admin Web-facing | Add `managedArtworkCandidateAccept`, emit `AcceptManagedArtworkCandidateResponse` and `JobResponse`, and keep the request body empty |
| Ingest requeue becomes Admin Web-facing | Add `managedArtworkIngestRequeue`, emit `RequeueManagedArtworkIngestResponse` and `ManagedArtworkIngestJobSummary`, and keep the request body empty |
| Server route inventory changes | `implemented_admin_routes_are_generated_or_explicitly_excluded` must pass with no stale exclusions |
| Diagnostic source records include paths, URIs, hashes, tokens, roots, etags, or provider payloads | Response/generated contract expose only safe summaries and presence booleans |
| Public Client contract output contains these Admin diagnostics | Treat as contract drift and remove from Public Client outputs |

### 5. Good/Base/Bad Cases

- Good: generated route inventory includes all three read-only diagnostics, the
  candidate accept selection command, and the ingest requeue retry command,
  while `process-next` remains excluded.
- Good: dedicated confirmed mutation tasks generate publish or stray-file
  remediation routes with typed Admin Web client tests and no raw storage
  material in response DTOs.
- Good: Admin Web maps generated DTOs into route-local safe rows before
  rendering.
- Base: Managed Artwork gallery remains item-scoped; maintenance diagnostics
  are global operator reads.
- Bad: returning a managed artifact storage record directly or exposing
  `storage_uri`, a filesystem path, raw file name, or content hash in generated
  TypeScript.

### 6. Tests Required

- API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Cross-crate compile:
  `cargo check -p nako-api --tests` and `cargo check -p nako-server --tests`.
- Frontend consumer checks:
  `npm run check --prefix apps/admin-web` and focused Admin Web route tests.

## Scenario: Managed Artwork Candidate Accept Generated Admin Contract

### 1. Scope / Trigger

- Trigger: generating or changing the Admin contract for
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- Scope: `crates/nako-api/src/admin/managed_artwork.rs`,
  `crates/nako-api/src/admin/operations.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `crates/nako-server/src/http/admin.rs`, and generated Admin TypeScript
  contracts under `apps/admin-web` and `web`.

### 2. Signatures

- Generated route key: `managedArtworkCandidateAccept`.
- Route:
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- Path parameter: `candidate_id`, encoded by Admin Web path helper.
- Request body: empty JSON object `{}`.
- Response:
  `AcceptManagedArtworkCandidateResponse { candidate_id, candidate_status, ingest, job }`.
- Ingest summary:
  `ManagedArtworkIngestSummary { id, candidate_id, job_id, library_id, item_id, kind, status, has_artifact, has_failure, failure_code, created_at, updated_at }`.
- Job summary:
  `JobResponse { id, kind, status, resource_class, priority, library_id, source_id, has_input, has_summary, has_error, attempt, max_attempts, retry_of_job_id, next_attempt_at, queued_at, started_at, completed_at, diagnostics? }`.

### 3. Contracts

- The route is an Admin-only operator selection command. It must stay out of
  Public Client route inventories, OpenAPI public outputs, and generated Public
  SDKs.
- The client supplies only `candidate_id` as a path parameter and `{}` as the
  POST body. It must not submit provider URLs, paths, storage handles, hashes,
  tokens, artifact IDs, or backend payloads.
- Accepting a candidate queues Managed Artwork ingest and returns safe candidate,
  ingest, and job summaries. It does not immediately publish selected public
  artwork.
- Candidate accept is not a destructive cleanup/delete command, so this
  low-level generated contract does not require a `confirm=true` query. Any
  future page workflow still needs a dedicated live-only UI task.
- Current Managed Artwork route exclusions after generated mutation contracts
  should contain only `artwork/ingests/process-next`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Route becomes generated | Remove only `artwork/candidates/{candidate_id}/accept` from `admin_contract_route_exclusions()` and add the generated route key |
| Candidate ID contains reserved URL characters | Admin Web path helper URL-encodes `candidate_id` |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Candidate accept succeeds | Response exposes safe candidate status, ingest summary, and job summary fields |
| Response/generated contract contains paths, URIs, provider URLs, tokens, file names, roots, etags, hashes, or backend payloads | Treat as a contract violation |
| Public Client output contains the route or DTOs | Treat as contract drift and remove them from Public outputs |

### 5. Good/Base/Bad Cases

- Good: Admin Web client calls
  `managedArtworkCandidateAccept` with encoded `candidate_id` and an empty
  body, then deserializes `AcceptManagedArtworkCandidateResponse`.
- Base: server queues a Managed Artwork ingest job and public selected artwork
  remains unchanged until the ingest/publication workflow completes.
- Bad: accepting `{ image_url }`, `{ storage_uri }`, or `{ artifact_id }` in
  the request body, or adding this Admin route to Public Client route inventory.

### 6. Tests Required

- API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Server candidate accept behavior:
  `cargo nextest run -p nako-server admin_accept_artwork_candidate --no-fail-fast`.
- Admin Web client:
  `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` and
  `npm run check --prefix apps/admin-web`.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkCandidateAccept, {
  image_url,
});
```

#### Correct

```typescript
return this.postJson(
  routeWithParam(
    NAKO_ADMIN_ROUTES.managedArtworkCandidateAccept,
    "candidate_id",
    candidateId,
  ),
  {},
);
```

The server owns candidate lookup and ingest queueing. Admin Web submits only the
opaque candidate ID selected by the operator.

## Scenario: Managed Artwork Ingest Requeue Generated Admin Contract

### 1. Scope / Trigger

- Trigger: generating or changing the Admin contract for
  `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`.
- Scope: `crates/nako-api/src/admin/managed_artwork.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `crates/nako-server/src/http/admin.rs`, and generated Admin TypeScript
  contracts under `apps/admin-web` and `web`.

### 2. Signatures

- Generated route key: `managedArtworkIngestRequeue`.
- Route:
  `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`.
- Path parameter: `ingest_id`, encoded by Admin Web path helper.
- Request body: empty JSON object `{}`.
- Response:
  `RequeueManagedArtworkIngestResponse { ingest, job, requeued, had_failure }`.
- Ingest summary:
  `ManagedArtworkIngestSummary { id, candidate_id, job_id, library_id, item_id, kind, status, has_artifact, has_failure, failure_code, created_at, updated_at }`.
- Job summary:
  `ManagedArtworkIngestJobSummary { id, kind, status, resource_class, library_id, source_id, has_input, has_summary, has_error, queued_at, started_at, completed_at }`.

### 3. Contracts

- The route is an Admin-only operator retry command. It must stay out of Public
  Client route inventories, OpenAPI public outputs, and generated Public SDKs.
- The client supplies only `ingest_id` as a path parameter and `{}` as the POST
  body. It must not submit provider URLs, storage handles, job input JSON,
  summary JSON, raw errors, paths, hashes, tokens, or artifact handles.
- Requeue may reset a failed Managed Artwork ingest and its durable job to a
  queued state. Idempotent replay for an already queued ingest may return
  `requeued: false`.
- Requeue is a retry command, not a worker executor. The
  `artwork/ingests/process-next` worker route must remain explicitly excluded
  until a separate runtime-control task defines an Admin Web-facing contract.
- A future page workflow still needs a dedicated live-only task before wiring
  controls to this low-level generated client method.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Route becomes generated | Remove only `artwork/ingests/{ingest_id}/requeue` from `admin_contract_route_exclusions()` and add the generated route key |
| Ingest ID contains reserved URL characters | Admin Web path helper URL-encodes `ingest_id` |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Failed ingest is requeued | Response exposes safe ingest and job summaries with `requeued: true` and `had_failure: true` |
| Already queued ingest is replayed | Response may expose `requeued: false` without leaking raw failure or job payload material |
| Stored/succeeded ingest is requeued | Server returns its existing safe conflict response without provider URL, token, or raw error material |
| Response/generated contract contains paths, URIs, provider URLs, tokens, file names, roots, etags, hashes, job input/summary JSON, or backend payloads | Treat as a contract violation |
| Public Client output contains the route or DTOs | Treat as contract drift and remove them from Public outputs |

### 5. Good/Base/Bad Cases

- Good: Admin Web client calls `managedArtworkIngestRequeue` with encoded
  `ingest_id` and an empty body, then deserializes
  `RequeueManagedArtworkIngestResponse`.
- Base: server resets a failed ingest/durable job to queued and leaves actual
  provider fetch/artifact storage to the worker/runtime path.
- Bad: accepting `{ input_json }`, `{ summary_json }`, `{ storage_uri }`, or
  `{ artifact_id }` in the request body, or generating the `process-next`
  worker route as part of the retry contract.

### 6. Tests Required

- API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Server requeue behavior:
  `cargo nextest run -p nako-server admin_managed_artwork_ingest_requeue --no-fail-fast`.
- Admin Web client:
  `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` and
  `npm run check --prefix apps/admin-web`.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkIngestRequeue, {
  input_json,
});
```

#### Correct

```typescript
return this.postJson(
  routeWithParam(
    NAKO_ADMIN_ROUTES.managedArtworkIngestRequeue,
    "ingest_id",
    ingestId,
  ),
  {},
);
```

The server owns retry validation and durable job reset. Admin Web submits only
the opaque ingest ID selected by the operator.

## Scenario: Managed Artwork Stray File Cleanup Confirmed Admin Contract

### 1. Scope / Trigger

- Trigger: generating or changing the Admin contract for
  `POST /admin/v1/artwork/artifacts/remediate-stray-files`.
- Scope: `crates/nako-api/src/admin/managed_artwork.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `crates/nako-server/src/http/admin.rs`,
  `crates/nako-server/src/http/query.rs`, and generated Admin TypeScript
  contracts under `apps/admin-web` and `web`.

### 2. Signatures

- Generated route key:
  `managedArtworkArtifactRemediateStrayFiles`.
- Query:
  `AdminManagedArtworkArtifactStrayFileCleanupQuery { confirm?: boolean, file_scan_limit?: number }`.
- Response:
  `AdminManagedArtworkArtifactStrayFileCleanupResponse { summary, cleaned_files, blocked_files, dry_run }`.
- Summary:
  `AdminManagedArtworkArtifactStrayFileCleanupSummary { file_scan_limit, scanned_files, cleanable_stray_files, blocked_stray_files, deleted_files, missing_files, failed_files, file_scan_truncated }`.
- Cleaned item:
  `AdminManagedArtworkArtifactStrayFileCleanupItem { recognized_artifact_id, extension, byte_len, status }`.
- Status:
  `AdminManagedArtworkArtifactStrayFileCleanupStatus = deleted | missing | failed`.

### 3. Contracts

- The route is an Admin-only confirmed mutation. It must stay out of Public
  Client route inventories, OpenAPI public outputs, and generated Public SDKs.
- Server query parsing requires `confirm=true` before cleanup executes.
- Admin Web client calls must send query parameters and an empty JSON object
  body; callers must not submit raw file names, storage paths, artifact roots,
  URIs, hashes, or backend details.
- Cleanup response DTOs may expose only counts, stable artifact IDs, extension,
  byte length, safe status/reason/action enum codes, and `dry_run`.
- Response DTOs and generated TypeScript must not expose raw file names, local
  paths, artifact roots, `storage_uri`, `managed-artwork://` handles,
  `source_uri`, `cache_uri`, provider URLs/query strings, tokens, credentials,
  raw content hashes, etags, or backend payloads.
- Generated contract artifacts under `apps/admin-web` and `web` must be
  regenerated from `nako-api`; do not hand-edit generated TypeScript output.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `confirm=true` is absent | Server returns invalid input and does not delete files |
| `confirm=true` is present with a bounded `file_scan_limit` | Server may delete only parseable untracked artifact files after rechecking active DB state |
| Route becomes generated | Remove only this suffix from `admin_contract_route_exclusions()` and add the generated route key |
| Generated response contains paths, URIs, hashes, tokens, file names, roots, etags, or provider payloads | Treat as a contract violation |
| Public Client output contains the route or DTOs | Treat as contract drift and remove them from Public outputs |

### 5. Good/Base/Bad Cases

- Good: Admin Web client calls
  `managedArtworkArtifactRemediateStrayFiles?confirm=true&file_scan_limit=50`
  with `POST {}` and deserializes only safe cleanup summary fields.
- Base: read-only maintenance diagnostics still expose remediation plans without
  executing cleanup.
- Bad: accepting a raw file path in the request body, returning a raw
  `storage_uri`, or adding this Admin route to Public Client route inventory.

### 6. Tests Required

- API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Server remediation behavior:
  `cargo nextest run -p nako-server admin_managed_artwork_remediation --no-fail-fast`.
- Admin Web client:
  `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` and
  `npm run check --prefix apps/admin-web`.

### 7. Wrong vs Correct

#### Wrong

```typescript
client.postJson(NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediateStrayFiles, {
  path: "F:/media/.nako/artwork/private.png",
});
```

#### Correct

```typescript
client.remediateManagedArtworkArtifactStrayFiles({
  confirm: true,
  file_scan_limit: 50,
});
```

The server owns target discovery and deletion authority; the client supplies
only confirmation and a bounded scan limit.

## Scenario: Managed Artwork Artifact Cleanup Confirmed Admin Contract

### 1. Scope / Trigger

- Trigger: generating or changing the Admin contract for
  `POST /admin/v1/artwork/artifacts/cleanup`.
- Scope: `crates/nako-api/src/admin/managed_artwork.rs`,
  `crates/nako-api/src/admin_contract.rs`,
  `crates/nako-server/src/http/admin.rs`,
  `crates/nako-server/src/http/query.rs`, and generated Admin TypeScript
  contracts under `apps/admin-web` and `web`.

### 2. Signatures

- Generated route key: `managedArtworkArtifactCleanup`.
- Query:
  `AdminManagedArtworkArtifactCleanupQuery extends AdminPageQuery { confirm?: boolean }`.
- Response:
  `AdminManagedArtworkArtifactCleanupResponse { examined_artifacts, cleanup_candidate_artifacts, cleaned_artifacts, file_deleted_artifacts, file_missing_artifacts, file_delete_failed_artifacts, dry_run }`.
- Cleaned item:
  `AdminManagedArtworkArtifactCleanupItem { id, ingest_id, library_id, item_id, kind, byte_len, media_type }`.

### 3. Contracts

- The route is an Admin-only confirmed mutation. It must stay out of Public
  Client route inventories, OpenAPI public outputs, and generated Public SDKs.
- Server query parsing requires `confirm=true` before cleanup executes.
- Cleanup accepts only confirmation and pagination query parameters. Do not
  reuse read-only lifecycle `cleanup_candidates_only` as an executable cleanup
  selector.
- Admin Web client calls must send query parameters and an empty JSON object
  body; clients must not submit raw artifact IDs, file names, storage paths,
  artifact roots, URIs, hashes, or backend details in the request body.
- Cleanup targets stay repository-owned cleanup candidates. HTTP handlers do not
  decide which artifacts are eligible.
- Response DTOs and generated TypeScript must not expose raw file names, local
  paths, artifact roots, `storage_uri`, `managed-artwork://` handles,
  `source_uri`, `cache_uri`, provider URLs/query strings, tokens, credentials,
  raw content hashes, etags, or backend payloads.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `confirm=true` is absent | Server returns invalid input and does not delete artifact rows or files |
| `confirm=true` is present | Server may delete only repository-owned unselected cleanup candidates |
| Route becomes generated | Remove only this suffix from `admin_contract_route_exclusions()` and add the generated route key |
| Generated response contains paths, URIs, hashes, tokens, file names, roots, etags, or provider payloads | Treat as a contract violation |
| Public Client output contains the route or DTOs | Treat as contract drift and remove them from Public outputs |

### 5. Good/Base/Bad Cases

- Good: Admin Web client calls
  `managedArtworkArtifactCleanup?confirm=true&limit=5&offset=10` with
  `POST {}` and deserializes only safe cleanup summary fields.
- Base: lifecycle diagnostics can preview cleanup candidates without executing
  cleanup.
- Bad: accepting `{ artifact_id }`, `{ storage_uri }`, or `{ path }` in the
  request body, or adding this Admin route to Public Client route inventory.

### 6. Tests Required

- API contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Server route inventory:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`.
- Server cleanup behavior:
  `cargo nextest run -p nako-server admin_managed_artwork_cleanup --no-fail-fast`.
- Admin Web client:
  `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` and
  `npm run check --prefix apps/admin-web`.

### 7. Wrong vs Correct

#### Wrong

```typescript
client.postJson(NAKO_ADMIN_ROUTES.managedArtworkArtifactCleanup, {
  artifact_id: "artifact-orphan",
});
```

#### Correct

```typescript
client.cleanupManagedArtworkArtifacts({
  confirm: true,
  limit: 5,
  offset: 10,
});
```

The repository owns cleanup candidate selection; the client supplies only
confirmation and pagination.

## Scenario: Admin Overview Source Fingerprint Hash Diagnostic

### 1. Scope / Trigger

- Trigger: changing the Admin overview source fingerprint hash diagnostic block,
  source fingerprint coverage counters, or source-hash job queue counters.
- Scope: `AdminOverviewResponse.source_fingerprint_hash`,
  `AdminOverviewSourceFingerprintHashSummary`,
  `MediaRepository::summarize_media_source_fingerprints`,
  `SourceFingerprintHashAppService::admin_overview_summary`, generated Admin
  TypeScript contracts, and Admin Web overview rendering.

### 2. Signatures

- API DTO:
  `AdminOverviewResponse { source_fingerprint_hash:
  AdminOverviewSourceFingerprintHashSummary }`.
- Repository aggregate:
  `summarize_media_source_fingerprints() ->
  Result<MediaSourceFingerprintSummary>`.
- Server app service:
  `admin_overview_summary() ->
  Result<AdminOverviewSourceFingerprintHashSummary>`.

### 3. Contracts

- Source coverage counters are exact aggregate counts from the repository, not
  HTTP-layer pagination or Admin Web calculations.
- Job counters are derived from `summarize_job_queue_pressure()` filtered to
  `JobKind::SourceFingerprintHash` and
  `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS`.
- The DTO may expose counts and safe queued/retry timestamps only. It must not
  expose Source Locator, local path, raw Source Fingerprint, raw content hash,
  job input JSON, job summary JSON, or raw job error body.
- Admin Web generated contracts under `apps/admin-web` and `web` must be
  refreshed from `nako-api`; do not hand-edit generated artifacts.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| No media sources or source hash jobs exist | Return zero counters and `null` timestamps |
| Sources have raw fingerprints or locator-like strings | Count them without returning the values |
| Source hash jobs exist in multiple statuses | Aggregate queued/running/succeeded/failed/cancelled counts by status |
| Queued source hash retries are delayed | Include delayed retry count and safe next retry timestamp |
| Other job kinds share the same queue | Exclude them from source fingerprint hash counters |
| Generated contract drift | `cargo nextest run -p nako-api admin_contract --no-fail-fast` fails until regenerated |

### 5. Good/Base/Bad Cases

- Good: add a repository SQL aggregate that returns total/fingerprinted/content
  hash source counts and map it into the Admin overview summary.
- Good: use existing job queue pressure summaries for job status counters.
- Base: Admin Web renders only counts and timestamps from the generated
  contract.
- Bad: scanning every media source inside the HTTP overview handler.
- Bad: exposing `fingerprint`, `source_uri`, `locator`, `path`, `hash`, or raw
  job JSON fields in the DTO or Admin Web.

### 6. Tests Required

- API serialization test asserts the new fields and rejects sensitive terms.
- DB contract test proves source fingerprint aggregate counts on persisted
  `media_sources` rows.
- Server app/route test proves source-hash queue counters and overview response
  remain redaction-safe.
- Admin contract test proves generated TypeScript artifacts match the
  generator.
- Admin Web tests prove the overview route renders only aggregate fields and
  omits unsafe extra fields.

### 7. Wrong vs Correct

#### Wrong

```rust
let sources = store.list_media_sources(library_id, PageRequest::new(PageRequest::MAX_LIMIT, 0)).await?;
```

This makes Admin overview coverage depend on a bounded page and will undercount
larger libraries.

#### Correct

```rust
let coverage = store.summarize_media_source_fingerprints().await?;
```

The repository owns exact aggregate counting, while the Admin overview service
maps only safe counters into the DTO.

## Scenario: Admin Jobs Source Fingerprint Hash Drilldown

### 1. Scope / Trigger

- Trigger: changing source fingerprint hash job diagnostics, Admin Jobs filters,
  Admin Web Jobs filter controls, or any route that lets operators inspect
  source-hash durable jobs.
- Scope: `GET /admin/v1/jobs`, `AdminJobsQuery`, `AdminJobListItem`,
  `JobListFilter`, Admin Web Jobs route search state, and generated Admin
  contracts.

### 2. Signatures

- Query filters:
  `kind=source_fingerprint_hash`,
  `resource_class=disk.scan.source_fingerprint_hash`, optional `status`,
  `library_id`, `source_id`, `limit`, and `offset`.
- Response:
  `AdminJobListResponse { jobs: AdminJobListItem[], page: PageInfo }`.
- Job item fields remain safe metadata:
  `id`, `kind`, `status`, `resource_class`, `library_id`, `source_id`,
  input/summary/error presence booleans, and timestamps.

### 3. Contracts

- Source fingerprint hash job drilldown reuses the existing Admin Jobs list
  before adding any source-hash-specific endpoint.
- Admin Web shortcuts may prefill the exact `kind` and `resource_class` values,
  but URL search params remain authoritative.
- `source_id` is a Media Source identifier filter, not a Source Locator or
  storage URI.
- Admin Jobs responses expose only presence booleans for input, summary, and
  error payloads. They must not expose job `input_json`, `summary_json`, raw
  error bodies, Source Locators, local paths, Source Fingerprints, content
  hashes, storage URIs, tokens, or backend payloads.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Source hash jobs exist with other job kinds in the queue | `kind` and `resource_class` filters return only source hash jobs |
| `source_id` is present | Parse as `MediaSourceId` and narrow results through `JobListFilter.source_id` |
| `source_id` is malformed | Return the existing invalid-input error without echoing unsafe payload details |
| Job input/summary/error contains raw paths, locators, or hashes | Response body omits those payloads and returns only `has_input`, `has_summary`, and `has_error` |
| Admin Web quick filter is clicked | Route search receives the exact kind/resource-class values and resets `offset` to `0` |

### 5. Good/Base/Bad Cases

- Good: `/admin/v1/jobs?kind=source_fingerprint_hash&resource_class=disk.scan.source_fingerprint_hash&source_id=...`
  returns the safe generic job row for that Media Source.
- Good: Admin Web renders a quick filter that writes existing Jobs route search
  params instead of filtering rows in memory.
- Base: generic Jobs route still works for library scans, metadata jobs, addon
  tasks, and source hash jobs through the same DTO.
- Bad: adding a source-hash-specific job detail route merely to show the same
  generic job fields.
- Bad: rendering raw source-hash job payloads, locators, fingerprints, or
  content hashes in Admin Web.

### 6. Tests Required

- Server route test seeds source hash jobs with sensitive input/error payloads,
  filters by `kind`, `resource_class`, and `source_id`, and asserts only safe
  job metadata is returned.
- Admin Web route test proves URL-owned search maps `source_id`, the quick
  filter writes exact source hash filters, and localized controls render.
- Admin Web redaction/rendering test must continue to reject raw job payload,
  locator, path, token, and hash terms.

### 7. Wrong vs Correct

#### Wrong

```typescript
const jobs = result.value.jobs.filter((job) => job.kind === "source_fingerprint_hash");
```

This hides rows client-side while leaving the authoritative URL/query contract
unusable for operator links and refreshes.

#### Correct

```typescript
onSearchChange({
  kind: "source_fingerprint_hash",
  resource_class: "disk.scan.source_fingerprint_hash",
  offset: 0,
});
```

The Admin Jobs route owns filtering, and the UI only writes safe query params.
