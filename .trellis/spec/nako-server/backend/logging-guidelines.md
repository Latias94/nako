# Logging Guidelines

`crates/nako-server` uses `tracing` and `tracing-subscriber`.

## Runtime Setup

- `main.rs` initializes tracing once with
  `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("nako_server=info"))`.
- The default subscriber is `fmt().with_env_filter(env_filter).try_init()`.
- Server startup logs the bound address with
  `info!(listen_addr = %local_addr, "nako HTTP server listening")`.

## Handler Instrumentation

- HTTP handlers commonly use `#[instrument(skip(app))]`.
- Skip principals, request bodies, headers, tickets, and other sensitive or
  high-cardinality fields:
  - `#[instrument(skip(app, principal, request))]`
  - `#[instrument(skip(app, principal, ticket_query, headers))]`
- Do not log raw bearer tokens, playback tickets, URLs containing secrets,
  local filesystem paths, request payloads that may include credentials, or
  provider raw response bodies.
- ADR 0053 requires diagnostics to be operator-useful and redacted.

## Scenario: Durable Job Trace Context

### 1. Scope / Trigger

- Trigger: when request identity is persisted into durable jobs, scheduler
  inputs, outbox payloads, or background diagnostics.
- This is infra/control-plane data. It must be useful for correlation while
  remaining safe to persist and display.

### 2. Signatures

- Use a typed trace-context value, not raw strings passed through job code.
- Accepted request IDs are normalized before persistence.
- Job enqueue APIs that carry trace context should keep the existing untraced
  entry point for legacy callers.

### 3. Contracts

- `request_id` may contain only ASCII alphanumeric characters plus `-`, `_`,
  and `.`.
- Length must be bounded; keep request IDs short enough for logs and payloads.
- Normalize safe request IDs consistently before writing job input or outbox
  payloads.
- Missing trace context means untraced work, not an error.

### 4. Validation & Error Matrix

- Safe request ID -> normalize and persist.
- Missing trace context -> preserve legacy untraced behavior.
- URL, path, token-like, whitespace, or non-ASCII request ID -> reject with a
  fixed redacted error message.
- Malformed persisted trace context -> fail redacted validation; never echo the
  stored raw value.

### 5. Good/Base/Bad Cases

- Good: `Trace-ABC_123` persists as a normalized safe request ID.
- Base: no trace context enqueues the same job input shape as before.
- Bad: `https://host/path?token=secret` is rejected and not shown in the error.

### 6. Tests Required

- Unit test safe normalization.
- Unit test unsafe values are rejected without echoing raw input.
- Deserialization test for persisted malformed context.
- Integration or app test proving the selected job/outbox path carries only the
  normalized request ID.

### 7. Wrong vs Correct

#### Wrong

```rust
input.trace_context = Some(raw_header_value.to_string());
```

This can persist URLs, paths, or tokens as diagnostics.

#### Correct

```rust
let trace_context = DurableJobTraceContext::new(raw_header_value)?;
input.trace_context = Some(trace_context);
```

Typed construction is the validation boundary, and errors must not include the
raw rejected value.

## Log Levels

- `info!`: lifecycle events, startup/shutdown, durable background progress.
- `warn!`: recoverable background failures, rejected requests, cleanup issues.
- `error!`: command-level failure, server errors, unrecoverable task failure.
- `debug!`: internal runtime coordination such as job lease polling or playback
  control state.

## Wrong vs Correct

### Wrong

```rust
#[instrument(skip(app))]
async fn handler(headers: HeaderMap, Json(request): Json<SecretRequest>) { ... }
```

This can capture sensitive arguments.

### Correct

```rust
#[instrument(skip(app, headers, request))]
async fn handler(headers: HeaderMap, Json(request): Json<SecretRequest>) { ... }
```

Keep spans useful without recording secrets or large request payloads.
