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
