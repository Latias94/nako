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
