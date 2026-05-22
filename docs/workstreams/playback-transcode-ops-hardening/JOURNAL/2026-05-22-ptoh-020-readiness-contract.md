# 2026-05-22 — PTOH-020 Readiness Contract

Implemented the first runtime supportability slice for
`playback-transcode-ops-hardening`.

## Shipped

- Added `HardwareAccelerationReadiness` in `taru-transcode` with stable status
  and reason categories for CPU requested, requested accelerator ready,
  requested accelerator unavailable with CPU fallback, fail policy, FFmpeg
  probe error, device initialization failure, and smoke probe failure.
- Added Admin API readiness DTOs with:
  - top-level readiness status/reason;
  - check entries for FFmpeg probe, hardware acceleration, selected fallback,
    transcode budget, remote playback budget, and staging.
- Updated `GET /admin/v1/playback/runtime` to compose the readiness contract
  from existing hardware, budget, remote playback, and staging diagnostics.
- Updated Admin TypeScript generated contract and mock runtime data. This is a
  contract sync, not Admin UI implementation.

## Verification

- `cargo nextest run -p taru-transcode hardware --no-fail-fast`: 9 passed.
- `cargo nextest run -p taru-server admin_v1_playback_runtime --no-fail-fast`:
  2 passed.
- `cargo nextest run -p taru-api admin_playback --no-fail-fast`: 2 passed.
- `cargo check -p taru-api --tests`: passed.
- `cargo check -p taru-server --tests`: passed.
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`: 5 passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with only repository CRLF conversion warnings.
- `git diff --name-only -- crates/taru-client-protocol`: no output.

## Review Notes

- Workstream compliance: PTOH-020 remains Admin-only, read-only, and
  runtime/diagnostic-focused.
- Code quality: the hardware readiness logic lives in `taru-transcode`; Admin
  DTO mapping stays in `taru-api`; route composition stays in
  `taru-server::http::admin`.
- Residual risk: deeper request/profile validation is intentionally left for
  PTOH-030.
