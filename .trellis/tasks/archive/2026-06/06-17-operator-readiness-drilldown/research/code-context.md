# Code context: operator readiness drilldown

## Existing backend shape

- `crates/nako-api/src/admin.rs` defines `AdminOverviewResponse` and existing
  `AdminOperatorReadinessSummary`, `AdminOperatorReadinessCheck`,
  `AdminOperatorReadinessAction`, `AdminOperatorReadinessStatus`,
  `AdminOperatorReadinessArea`, and `AdminOperatorReadinessReason`.
- `crates/nako-server/src/http/admin.rs` builds the overview in
  `admin_overview_response`.
- `operator_readiness_summary` already composes seven checks from safe
  intermediate facts:
  setup, Media Library scan, playback, durable jobs, storage, network, backup.
- The overview code already fetches the likely detail inputs:
  storage backend diagnostics, job queue pressure, runtime diagnostics, source
  fingerprint hash overview summary, library scan posture aggregate, VFS cache
  repair readiness pressure, network readiness diagnostics, playback readiness
  diagnostics, startup summary, and latest watch-folder ticks.

## Product gap

The overview returns an actionable top-level signal, but operators still need a
single backend surface that explains the signal without making the dashboard
response carry every diagnostic payload. A dedicated read-only Admin route fits
the current Admin API pattern and keeps the existing overview stable.

## Safe data sources to reuse

- Setup: auth enabled flag, has token reference boolean, network exposure mode
  classification, and system-config route guidance.
- Media Library scan: configured library count, source fingerprint hash summary,
  library scan posture aggregate, and watch-folder runtime coverage summary.
- Playback: `AdminPlaybackReadinessDiagnostics` from
  `admin_playback_runtime_diagnostics`.
- Durable jobs: `JobQueuePressureSummary` projected through existing safe Admin
  queue pressure DTOs.
- Storage: `AdminOverviewStorageSummary` plus
  `VfsCacheRepairReadinessPressure`.
- Network: `AdminNetworkReadinessDiagnostics` from
  `network_readiness_diagnostics`.
- Backup: durable database boolean and backup runbook source reason.

## Redaction boundary

The route must not expose raw Source Locators, local paths, backend URLs,
credentials, token env names, raw database URLs, raw durable job input/summary
JSON, FFmpeg command lines, stderr, or provider payloads. It should expose
typed enums, booleans, counts, status, route keys, templated route paths, and
safe timestamps only.

## Candidate implementation

- Add `AdminOperatorReadinessResponse` and detail DTOs in `nako-api`.
- Add `GET /admin/v1/operator-readiness` in `nako-server::http::admin`.
- Extract a shared internal builder that returns both `AdminOverviewResponse`
  inputs and detailed readiness facts, or create a focused helper to collect
  safe facts once for the new endpoint.
- Add the route key `operatorReadiness` to `admin_contract.rs`.
- Regenerate `apps/admin-web/src/adminApi/generated/contract.ts` and
  `web/src/api/admin/generated/contract.ts` if the generator targets both.

## Focused tests

- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server operator_readiness --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
