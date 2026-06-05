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
