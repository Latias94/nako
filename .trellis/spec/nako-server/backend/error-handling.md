# Error Handling

Use the current `src/http/error.rs` boundary for HTTP-facing failures.

## Current Conventions

- Route handlers return `ApiResult<T>`, an alias for `Result<T, ApiError>`.
- `ApiError` wraps `nako_core::NakoError` and implements `IntoResponse`.
- App/domain code should return `NakoError`; HTTP handlers should use `?` and
  let `From<NakoError> for ApiError` convert it.
- Public error bodies use `nako_api::public_client::ErrorResponse` and
  `ClientErrorCode`, matching ADR 0023.
- `Database` and provider/storage implementation details are redacted into
  public messages such as `database operation failed`,
  `external provider operation failed: <provider>`, or `storage operation failed`.
- `InvalidInput`, `NotFound`, `Conflict`, `Unauthorized`, `Forbidden`, and
  `Unsupported` use their `NakoError` display text as the public message.

## HTTP Status Mapping

| `NakoError` kind | HTTP status |
|------------------|-------------|
| `InvalidInput`, `Unsupported` | `400 Bad Request` |
| `Unauthorized` | `401 Unauthorized` plus `WWW-Authenticate: Bearer` |
| `Forbidden` | `403 Forbidden` |
| `NotFound` | `404 Not Found` |
| `Conflict` | `409 Conflict` |
| storage budget exhausted | `507 Insufficient Storage` |
| storage timeout | `504 Gateway Timeout` |
| storage rate limited | `503 Service Unavailable` |
| provider/storage operation | `502 Bad Gateway` |
| database operation | `500 Internal Server Error` |

## Logging Contract

- Server errors log with `error!(error = %self.0, status = %status, "request failed")`.
- Non-server request rejections log with
  `warn!(error = %self.0, status = %status, "request rejected")`.
- Do not add per-handler duplicate logs for errors already returned through
  `ApiError` unless the handler has additional safe context to record.

## Tests Required

For new HTTP error behavior, assert:

- HTTP status code;
- `ErrorResponse.code`;
- public message redacts raw secrets, provider bodies, database details, local
  paths, and bearer values;
- `401` responses include `WWW-Authenticate: Bearer`.

## Wrong vs Correct

### Wrong

```rust
Err(anyhow::anyhow!("missing item"))?
```

### Correct

```rust
Err(NakoError::NotFound {
    entity: "media_item",
    id: item_id.to_string(),
})?
```

The HTTP boundary can map `NakoError` into a stable status code and public
`ErrorResponse`.
