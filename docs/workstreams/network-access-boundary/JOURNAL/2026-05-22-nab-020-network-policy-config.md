# NAB-020 Network Policy Domain And Config Validation

Date: 2026-05-22
Task: NAB-020
Status: DONE

## Summary

Added the first Network Access Boundary implementation slice. Taru now has a
server config policy model for local-only, private-network, reverse-proxy, and
tunnel-provider exposure modes, plus config-check validation that rejects unsafe
or incomplete remote access shapes before startup.

## Implementation

- Added `NetworkAccessConfig` to `TaruServerConfig`.
- Added `NetworkExposureMode` values:
  - `local_only`
  - `private_network`
  - `reverse_proxy`
  - `tunnel_provider`
- Added tunnel-provider declarations with provider kind, public URL, and
  `token_env` reference.
- Added config preflight checks for:
  - reverse-proxy/tunnel external base URL requirements;
  - auth requirement for private-network, reverse-proxy, and tunnel modes;
  - trusted proxy headers requiring trusted proxy sources;
  - explicit non-wildcard HTTP(S) browser origins;
  - tunnel providers requiring public URL and non-empty token environment value.
- Updated self-hosted deployment docs and example configs.
- Updated existing test fixtures to use the default network policy.

## Boundary Notes

- This slice records policy/readiness only. It does not start cloudflared,
  ngrok, Tailscale, relay services, TURN/STUN, or any built-in NAT traversal.
- Public Client API and `taru-client-protocol` were not changed.
- Config-check diagnostics report categories/counts/environment variable names,
  not bearer token values or tunnel credentials.

## TDD Notes

- Red gate: the first reverse-proxy config-check test failed to compile because
  the server config had no network policy field or exposure/tunnel types.
- Implementation stayed in config/preflight and deployment docs; request-time
  HTTP forwarded-header/CORS behavior is intentionally left for NAB-030.

## Verification

- `cargo nextest run -p taru-server config --no-fail-fast` — pass, 36 passed,
  203 skipped.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast` — pass,
  19 passed, 220 skipped.
- `cargo fmt --all -- --check` — pass.
- `python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json`
  — pass.
- `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`
  — pass.
- `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`
  — pass.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/taru-client-protocol` — no output.

## Next

Continue with NAB-030 request-time HTTP boundary enforcement for trusted
forwarded headers and CORS/origin behavior.