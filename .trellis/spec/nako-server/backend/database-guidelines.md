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
