# Error Handling

`nako-api` owns error response shapes and constants when they are part of the
wire contract. `nako-server` maps `NakoError` into HTTP responses.

## Rules

- Keep public error bodies stable and version-aware per ADR 0023.
- Do not put adapter-specific errors, SQL strings, provider payloads, local
  paths, raw tokens, or playback tickets into error DTOs.
- New API-visible error fields require route tests in `nako-server` and
  contract/generator tests in `nako-api`.
- Admin-only diagnostics can be richer than Public Client errors, but must
  remain redaction-safe.

## Wrong vs Correct

### Wrong

```rust
pub struct ErrorResponse {
    pub sql: String,
    pub local_path: String,
}
```

### Correct

```rust
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}
```

## Evidence

- ADR 0023
- `crates/nako-server/src/http/error.rs`
- `crates/nako-api/src/public_client.rs`
