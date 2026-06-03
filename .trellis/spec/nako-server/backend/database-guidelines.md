# Database Guidelines

`nako-server` uses repository traits and database facades; it does not own SQL,
migrations, row mappers, or adapter-specific persistence behavior.

## Rules

- App services call repository traits or `NakoDatabase` facade methods. They do
  not issue raw SQL.
- HTTP handlers should not coordinate database work directly. Handlers translate
  request/response and delegate to app services.
- Schema, migration, and row-mapping changes belong in `nako-core` and
  `nako-db`.
- Cross-layer changes that add a persisted field usually need updates in:
  `nako-core`, `nako-db`, `nako-api`, `nako-server`, and tests.
- List surfaces exposed through server routes must remain bounded and paginated
  per ADR 0053.

## Scenario: Staging Admission Attribution Consumption

### 1. Scope / Trigger

- Trigger: server storage-pressure admission or diagnostics consume persisted
  staging manifest attribution.
- Scope: `app/storage.rs`, scan admission, startup/job guards, and Admin
  staging diagnostics.

### 2. Signatures

- `StorageBackendRegistry::library_scan_admission_error(&Library) ->
  Result<Option<NakoError>>`
- `StorageBackendRegistry::summarize_staging_budget_policy() ->
  Result<Vec<StagingBudgetPolicySlice>>`
- `StagingManifestRecord { attribution: StagingAttribution, ... }`

### 3. Contracts

- Server code must treat persisted staging attribution as the only authority for
  per-library staging ownership.
- Scan admission must check backend-level critical pressure before library slice
  pressure so ambiguous or unknown records can still block the matching backend.
- `attributed(library_id)` records may increase both the backend slice and the
  matching library slice.
- `ambiguous` and `unknown` records may increase only the backend slice.
- Diagnostics may expose attribution kind and optional library id, but not raw
  source locators, local paths, fingerprints, etags, credentials, or raw
  backend failures.

### 4. Validation & Error Matrix

- Backend slice is critical but library slice is not -> admission must still
  reject with a redaction-safe staging-pressure error.
- Ambiguous same-root record increases a per-library slice -> contract
  violation.
- Server recomputes ownership from `source_uri` roots or prefixes -> contract
  violation.
- Admin staging body contains raw locator/path/error details -> redaction
  contract violation.

### 5. Good/Base/Bad Cases

- Good: a WebDAV backend with one attributed record and one ambiguous record is
  blocked because backend pressure is critical, even if the library slice alone
  is below threshold.
- Base: a library with only attributed records is blocked by whichever becomes
  critical first: backend or library slice.
- Bad: admission allows a scan because the library slice is under threshold
  while the backend aggregate is already critical.

### 6. Tests Required

- Focused server tests for attributed local/remote pressure, ambiguous same-root
  pressure, and mixed attributed+ambiguous backend pressure.
- HTTP/Admin tests prove staging policy slices stay bounded and redaction-safe.
- Startup/job guard tests prove critical backend pressure blocks remote scan
  execution before pipeline work starts.

### 7. Wrong vs Correct

#### Wrong

```rust
if library_policy.is_critical() {
    return Err(staging_pressure_error(library_policy));
}
```

#### Correct

```rust
if backend_policy.is_critical() {
    return Err(staging_pressure_error(backend_policy));
}
if library_policy.is_critical() {
    return Err(staging_pressure_error(library_policy));
}
```

## Good/Base/Bad Cases

- Good: `app/metadata.rs` calls a metadata service/repository and maps the
  summary to an API DTO.
- Base: Admin diagnostics route reads a bounded page through an app service.
- Bad: an Axum handler runs a SQL query or builds pagination by loading every
  row.

## Wrong vs Correct

### Wrong

```rust
async fn handler(State(app): State<NakoApp>) -> ApiResult<Json<Response>> {
    let rows = sqlx::query("SELECT * FROM jobs").fetch_all(app.pool()).await?;
    Ok(Json(map_rows(rows)))
}
```

### Correct

```rust
async fn handler(State(app): State<NakoApp>) -> ApiResult<Json<Response>> {
    let page = app.jobs().list_jobs(filter, page).await?;
    Ok(Json(map_jobs(page)))
}
```

## Evidence

- `crates/nako-server/src/app/*.rs`
- `crates/nako-server/src/http/*.rs`
- `crates/nako-db/src/contract_tests.rs`
