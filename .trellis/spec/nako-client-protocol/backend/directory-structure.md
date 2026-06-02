# Directory Structure

`nako-client-protocol` is split into a small route/account root module and a
large catalog/playback DTO module.

## Current Layout

- `src/lib.rs`: version/header constants, route inventory, route filters,
  system/account DTOs, error codes, and pagination.
- `src/catalog.rs`: browse, catalog, management context, source probe, playback,
  renderer, transcode, user playback, playlist, media stream, metadata, image,
  and public string enum DTOs.

## Module Rules

- Keep route inventory and protocol headers in `lib.rs`.
- Keep DTO families in `catalog.rs` while the crate remains simple.
- Split DTO modules only by public API family, not by server implementation
  module.
- Re-export DTOs from `lib.rs` so consumers keep one import surface.

## Naming Rules

- Use `*Response` for API response envelopes.
- Use `*Request` for client request payloads.
- Use `*Dto` for public data transfer objects.
- Use `Client*` prefixes for public enum/domain labels.
- Use `Public*` prefixes for route inventory metadata.

## Anti-Patterns

- Do not mirror server module names.
- Do not hide public route paths inside SDK-only crates.
- Do not create DTOs that expose internal persistence or runtime paths.
