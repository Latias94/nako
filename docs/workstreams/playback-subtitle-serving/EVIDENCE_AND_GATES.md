# Playback Subtitle Serving Evidence And Gates

| Gate | Command | Result |
| --- | --- | --- |
| Server subtitle route tests | `cargo nextest run -p nako-server subtitle --no-fail-fast` | Passed: 19 tests |
| Browser ticket tests | `cargo nextest run -p nako-server browser_playback_ticket --no-fail-fast` | Passed: 7 tests |
| Auth bypass route scope | `cargo nextest run -p nako-server playback_ticket_bypass --no-fail-fast` | Passed: 1 test |
| Protocol browser playback tests | `cargo nextest run -p nako-client-protocol browser_playback --no-fail-fast` | Passed: 1 test |
| Protocol route inventory | `cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast` | Passed: 3 tests |
| Public OpenAPI contract | `cargo nextest run -p nako-api openapi --no-fail-fast` | Passed: 9 tests |
| Public SDK contracts | `cargo nextest run -p nako-api typescript_sdk --no-fail-fast`; `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast` | Passed: 5 TypeScript SDK tests and 3 Kotlin SDK tests |
| Rust check | `cargo check -p nako-api -p nako-client -p nako-client-protocol -p nako-server --tests` | Passed |
| Format | `cargo fmt --all -- --check` | Passed |
| Diff check | `git diff --check` | Passed |

## Evidence Log

- 2026-05-28: PSS-010 opened the lane and selected host-owned playback
  subtitle serving as the next bounded slice.
- 2026-05-28: PSS-020 shared sidecar derivation through
  `app::subtitle_sidecar`, so import writes and playback reads derive the same
  safe leaf and storage URI.
- 2026-05-28: PSS-030 added host-owned subtitle serving at
  `/sources/{source_id}/subtitles/{stream_index}` with source play access,
  playback policy checks, size bounds, content type mapping, and redacted
  storage errors.
- 2026-05-28: PSS-040 added browser ticket mode `subtitle`, public route
  inventory/OpenAPI/TypeScript/Kotlin SDK contract updates, and tests ensuring
  subtitle URLs remain opaque and scoped to a stream index.
