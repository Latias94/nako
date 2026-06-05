# Quality Guidelines

API contract work must keep generated artifacts and route inventories honest.

## Required Patterns

- Update DTO source first, generator second, generated artifacts last.
- Regenerate generated Admin Web contract files from `nako-api`; do not edit
  generated TypeScript directly.
- Keep Admin `/admin/v1/*` routes out of Public Client/OpenAPI/SDK outputs
  unless the task explicitly changes the public contract.
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
- Do not add a DTO field before deciding redaction and audience.

## Review Checklist

- Does this route belong to Admin API or Public Client API?
- Is the generated output updated by the generator?
- Are route inventory tests updated?
- Are sensitive fields redacted or omitted?

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
