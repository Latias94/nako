# Phase 7.3: Playback Error Mapping

Status: active; first stable HTTP mappings implemented.

## Goal

Make playback and remote-storage failures return stable API error codes and
safe public messages instead of exposing raw backend details.

## Implemented Shape

- Added fixed HTTP/API mapping for:
  - `staging_budget_exhausted` -> `507`;
  - `staging_validation_mismatch` -> `502`;
  - `storage_timeout` -> `504`;
  - `storage_unauthorized` -> `502`;
  - `storage_rate_limited` -> `503`;
  - `ffmpeg_error` -> `502`.
- Kept generic `storage_error` and `provider_error` fallbacks for unmapped
  failures.
- Public messages for mapped storage and FFmpeg failures are stable and do not
  include backend URLs, credentials, or raw process output.

## Validation

- `cargo nextest run -p taru-server api_errors_map_playback_storage_categories direct_stream_rejects_unsatisfiable_and_multi_ranges remux_stream_route_maps_in_flight_duplicate_to_conflict hls_segment_route_rejects_unfinished_session`

## Remaining Gaps

- Core errors are still classified from current `TaruError` variants and
  message patterns; a typed playback/storage error enum is still needed.
- Route-level tests should cover real app paths for budget exhaustion,
  backend authorization failures, and timeout-like storage failures.
- HTTP API docs still need a fuller error taxonomy table before M7
  stabilization.
