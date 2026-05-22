# Phase 23.0: API, HTTP, and DB Boundary Cleanup

## Summary

M23 keeps the modular monolith shape clean before the next feature wave. The
goal is to prevent the public API, HTTP router, and SQLite repository layer from
accidentally becoming hidden coupling points as metadata, NFO, automation, and
client work expand.

## API DTO Boundary

High-frequency HTTP responses now use field-level API DTOs instead of exposing
core aggregate structs directly. The first enforced surface covers:

- libraries and library sources;
- catalog list/detail/search responses;
- people, tags, genres, credits, images, and source probe responses;
- playback decision source/probe wrappers and persisted transcode sessions;
- ingestion failure diagnostics.

The API crate may still reuse stable ID and enum value types while the project
is a single Rust workspace. New response bodies should not embed mutable domain
aggregates such as `MediaItem`, `MediaSource`, `Library`,
`IngestionFailureRecord`, or `TranscodeSessionRecord` directly.

## HTTP Router Boundary

`nako-server::http` keeps the root router as composition only. Route
registration lives beside the handlers for each bounded context:

- `system::routes`
- `library::routes`
- `catalog::routes`
- `metadata::routes`
- `playback::routes`
- `webhooks::routes`
- `automation::routes`
- `addons::routes`
- `jobs::routes`

Handlers remain responsible for request extraction and response translation.
Application services remain responsible for orchestration and repository calls.

## SQLite Repository Boundary

`nako-db/src/lib.rs` should define the `SqliteStore`, module graph, and shared
codec exports only. Repository implementations and their private lookup helpers
belong in the bounded-context module that owns the table behavior.

The cleanup moved the remaining cross-cutting helpers out of `lib.rs` and into
the job, automation, playback, webhook, media, and catalog modules.

## Forward Rules

- New public response shapes should add explicit DTOs in `nako-api`.
- DTO tests should cover serialization for new response shapes that could leak
  persistence-only fields, paths, secrets, or internal JSON columns.
- `http.rs` should not accumulate route registrations directly; add a module
  `routes()` function instead.
- `nako-db/src/lib.rs` should not regain repository methods.
- When a new API needs internal model fields, copy the required fields into a
  DTO intentionally instead of embedding the core record.

## Validation

Close-out validation:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```
