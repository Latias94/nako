# feat: Admin operator readiness drilldown read model

## Goal

Add a backend Admin read model that lets a self-hosted operator drill from the
existing operator readiness overview into redaction-safe setup, Media Library
scan, playback, durable job, storage, network, and backup evidence.

## What I Already Know

- `GET /admin/v1/overview` already returns `operator_readiness` with seven
  checks: setup, Media Library scan, playback, durable jobs, storage, network,
  and backup.
- Existing checks already carry stable `area`, `status`, `reason`,
  `source_reason`, `attention_count`, and optional `action` route guidance.
- `nako-server` already builds the relevant safe facts while creating the
  overview: queue pressure, source fingerprint hash summary, library scan
  posture, startup/watch-folder coverage, VFS cache repair pressure, network
  readiness, playback runtime readiness, and storage backend summary.
- The strategic roadmap asks for operator-facing backend read models that
  expose setup, scan, playback, storage, network, backup, durable-job, and
  repair pressure without raw paths, Source Locators, credentials, tokens,
  provider payloads, FFmpeg commands, backend URLs, or raw durable job payloads.

## Requirements

- R1. Add a read-only Admin API endpoint for operator readiness drilldown.
- R2. The response must include the same top-level readiness summary semantics
  as the overview so Admin clients can link overview checks to the detailed
  read model.
- R3. The response must expose per-area detail sections for setup, Media
  Library scan, playback, durable jobs, storage, network, and backup.
- R4. Detail sections must be derived from existing redaction-safe summaries
  and route/action guidance, not raw persistence rows, local config secrets, raw
  job input, raw FFmpeg command lines, Source Locators, or local paths.
- R5. The endpoint must stay read-only: no scan enqueue, repair execution,
  cache refresh, retry, config mutation, or runtime start/stop behavior.
- R6. The Admin contract generator must include the new route and DTOs, and the
  generated Admin Web TypeScript contracts must be refreshed from `nako-api`.
- R7. Existing `/admin/v1/overview` behavior must remain compatible and should
  not grow into an unbounded diagnostic payload.

## Acceptance Criteria

- [ ] `GET /admin/v1/operator-readiness` is implemented under the existing
  Admin route guard and returns `Cache-Control: no-store`.
- [ ] The response includes Admin/Public API versions, a summary, and bounded
  per-area detail facts for all seven readiness areas.
- [ ] Degraded/unavailable playback and network states include their existing
  typed readiness checks without exposing unsafe host material.
- [ ] Durable job detail exposes queue-pressure aggregates only, including
  counts, claimable/delayed retry counts, resource class, and safe timestamps.
- [ ] Media Library scan detail exposes configured library count, scan posture,
  source fingerprint hash pressure, and watch-folder coverage diagnostics only.
- [ ] Storage detail exposes backend counts and VFS cache repair pressure
  without raw target refs, URIs, paths, backend URLs, etags, fingerprints, or
  durable job input.
- [ ] Setup and backup detail expose booleans/counts/source reasons, not token
  env names, raw database URLs, or exact listener/origin values.
- [ ] `nako-api` admin contract tests and focused `nako-server` HTTP tests cover
  route inventory, response shape, Admin-only access, no-store cache policy, and
  redaction.

## Technical Approach

Add named Admin DTOs in `nako-api` for an
`AdminOperatorReadinessResponse`. Reuse the existing `AdminOperatorReadiness*`
summary/check/action types and add narrow detail DTOs for each readiness area.

Add `GET /admin/v1/operator-readiness` in `nako-server::http::admin`. Build the
read model from the same safe intermediate values already used by
`admin_overview_response` so readiness and detail cannot drift. Keep the HTTP
handler thin and return `no_store_json`.

Add the route to `ADMIN_ROUTE_SUFFIXES` in `admin_contract.rs`, regenerate the
Admin Web generated contracts, and add focused route/API tests.

## Decision (ADR-lite)

**Context:** The existing overview check list is good for the dashboard, but it
cannot carry enough detail for operator drilldown without bloating every
overview fetch.

**Decision:** Add a separate read-only Admin drilldown route that composes
existing safe diagnostics and keeps overview as a compact entry point.

**Consequences:** Admin Web and future agents get a stable backend product
surface for operator support. The route adds a new Admin contract, so generated
contracts and route inventory tests must be kept in sync.

## Out of Scope

- No new mutation routes or repair/scan execution commands.
- No destructive VFS repair policy.
- No new schema migrations or repository scans for raw job/source detail.
- No public client route, OpenAPI public route, or Public SDK change.
- No Admin Web UI implementation in this slice beyond generated contract
  refresh if the backend contract changes it.
- No broad incident bundle changes.

## Technical Notes

- Task follows `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`,
  especially U1 Operator Readiness And Control-Plane Audit.
- Control-plane authority: `docs/architecture/CONTROL_PLANE.md` and ADR 0053.
- API contract authority: `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`.
- Server route authority: `.trellis/spec/nako-server/backend/http-api-patterns.md`.
- Local research notes: `research/code-context.md`.
