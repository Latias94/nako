# nako-server Backend Development Guidelines

These specs document only patterns backed by current `crates/nako-server`
examples and matching architecture decisions.

## Pre-Development Checklist

- Read [HTTP API Routes and Auth](./http-api-patterns.md) before changing Axum routes, route handlers, auth, or access checks.
- Read [Error Handling](./error-handling.md) before returning new `NakoError` variants through HTTP.
- Read [Logging Guidelines](./logging-guidelines.md) before adding request spans or background task logs.
- Read [Quality Guidelines](./quality-guidelines.md) before adding server tests or choosing test commands.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [HTTP API Routes and Auth](./http-api-patterns.md) | Axum route assembly, handler signatures, auth, admin checks, access checks | Filled from code and ADRs |
| [Error Handling](./error-handling.md) | `ApiResult`, `ApiError`, public error bodies, HTTP status mapping | Filled from code and ADR 0023 |
| [Logging Guidelines](./logging-guidelines.md) | `tracing` setup, handler instrumentation, secret-safe skip fields | Filled from code and ADR 0053 |
| [Quality Guidelines](./quality-guidelines.md) | Server-focused Rust test patterns and gates | Filled from code and repo gates |
| [Directory Structure](./directory-structure.md) | App service, HTTP route, runtime, config, and test module layout | Filled from code and ADR 0019/0053 |
| [Database Guidelines](./database-guidelines.md) | Server persistence usage through repositories and app services | Filled as SQL non-ownership boundary |

## Authority / Evidence

- ADR 0019: thin server composition root and explicit runtime supervisors.
- ADR 0023: public API version and error envelope.
- ADR 0024: inbound bearer-token auth boundary.
- ADR 0027: versioned Admin API boundary.
- ADR 0036: short-lived browser playback ticket exception.
- ADR 0037: local credential and session auth.
- ADR 0053: control-plane boundary, diagnostics, pagination, redaction.
- `crates/nako-server/src/http.rs`
- `crates/nako-server/src/http/auth.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/access.rs`
- `crates/nako-server/src/http/error.rs`
- `crates/nako-server/src/http/tests/mod.rs`
- `crates/nako-server/src/http/catalog.rs`
- `crates/nako-server/src/main.rs`
- `crates/nako-server/src/app/job_runtime.rs`
- `crates/nako-server/src/app/artwork.rs`
- `crates/nako-server/src/app/playback/resource.rs`
