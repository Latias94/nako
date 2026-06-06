# Next Product Development Lanes

Date: 2026-06-05

## Purpose

This note re-ranks the next Nako product-development lanes after the
cross-lane architecture audit and the completed Admin route inventory parity
gate plus playback output/device capability audit.

The goal is "continue product development first": choose work that improves
the self-hosted media-server experience for Client Applications, operators, or
Addon users, while avoiding pure cleanup that does not unlock a product slice.

## Current Baseline

- The Admin route inventory parity gate is completed. Future Admin/API work
  still needs one contract owner when generated Admin TypeScript contracts or
  route inventory change, but the immediate route-drift gate is no longer the
  top next task.
- The playback output/device capability audit is completed. It recommends
  capability-contract parity before any HEVC/AV1, hardware tone-map, image
  subtitle burn-in, or full profile database work.
- Remote access has shipped safe network policy/readiness and Admin redaction.
  The product gap is operator-facing cookbook and config fixture validation,
  not built-in tunnel runtime.
- Source Fingerprint hash execution, scheduler integration, evidence
  persistence, Admin overview diagnostics, and Jobs drill-down filters are
  shipped. The product gap is trigger/retry/reconciliation policy.
- Addon Protocol and Addon Sidecar foundations are broad enough for product
  work, but host-owned resource flows need one policy language before adding
  more Addon-managed acquisition or Addon Manager behavior.

## Classification Snapshot

| Priority | Direction | Parallelizability |
| --- | --- | --- |
| 1 | Public Client playback capability contract parity | `serial-first` |
| 2 | Remote access cookbook and config fixture gates | `can parallel` |
| 3 | Source Fingerprint hash triggering and reconciliation policy | `architecture-first` |
| 4 | Host-owned Addon resource flow pattern | `architecture-first` |
| 5 | VFS cache non-destructive remediation planning | `can parallel` |
| Defer | HEVC/AV1 execution, hardware tone mapping, built-in tunnel runtime, automatic duplicate mutation, Addon Manager process lifecycle, and broad refactor campaigns | `blocked` |

## Priority 1: Public Client Playback Capability Contract Parity

**Recommended first PR**:
`public-client-playback-capability-contract-parity-gate`.

**Why this is product value, not pure refactor**:

Client Applications already send playback capability facts, and Nako already
uses those facts to choose Direct Play, Remux, or HLS Transcode. The current
contract drift means browser, SDK, client-core, OpenAPI, and docs can disagree
about which playback capability fields exist. Closing that gap directly
improves playback predictability for browser/mobile/TV/native clients and
enables the next profile/device-family slice without changing playback behavior
under users unexpectedly.

**Parallelizability**: `serial-first`.

This should own Public Client playback capability DTO/query/docs scope while it
runs. After it lands, profile skeleton work and Admin support evidence can be
split more safely.

**Likely file scope**:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-client-protocol/src/lib.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-client/src/lib.rs`
- `crates/nako-client-core/src/lib.rs`
- `crates/nako-client-core/src/playback.rs`
- `sdk/typescript/src/index.ts`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
- `docs/api/HTTP_API.md`

**Conflict surface**:

- Any task editing `ClientPlaybackCapabilitiesDto`,
  `BrowserPlaybackCapabilitiesDto`, playback capability query mapping, Public
  OpenAPI playback schemas, generated SDKs, or client playback request builders.
- Do not run with `playback-output-profile-v2-skeleton-contract-only` unless
  one lane owns contract ordering.

**Suggested validation**:

```powershell
cargo nextest run -p nako-client-protocol public_playback
cargo nextest run -p nako-api public_openapi_playback
cargo nextest run -p nako-client playback_decision_ticket_and_session_cancel_paths_are_stable streaming_request_builders_use_stable_paths_methods_headers_and_queries
cargo nextest run -p nako-client-core playback_decision_request_uses_core_route_query_auth_and_redaction playback_targets_follow_mode_without_auth_or_media3_policy
cargo fmt --all -- --check
git diff --check
```

**Next PR after that**:
`playback-output-profile-v2-skeleton-contract-only`: additive optional
`profile_id`, `profile_version`, `device_family`, `player_engine`, profile row,
subtitle, audio, color, and HLS facts, with legacy-flat-field mapping that
keeps existing decisions stable.

## Priority 2: Remote Access Cookbook And Config Fixture Gates

**Recommended first PR**:
`remote-access-cookbook-config-fixtures`.

**Why this is product value, not pure refactor**:

Self-hosted operators need to make Nako reachable outside the LAN safely.
Reverse proxy, HTTPS, DDNS, Tailscale Funnel, Cloudflare Tunnel, ngrok, CORS,
trusted proxy, and playback-ticket caveats are day-one product usability. The
code already has redacted readiness and preflight behavior; docs plus fixture
gates turn that into deployable operator confidence.

**Parallelizability**: `can parallel`.

It is mostly docs/ops and fixture validation. It can run beside playback,
Addon, or source-hash planning if it avoids Admin DTO and server config shape
changes.

**Likely file scope**:

- `docs/deployment/SELF_HOSTED.md`
- `docs/deployment/RELEASE_CHECKLIST.md`
- `docs/deployment/RELEASE_ARTIFACTS.md`
- `deploy/compose/*.yml`
- `scripts/release-gate.ps1`
- `scripts/release-gate.sh`
- `crates/nako-server/src/config.rs` only if a tiny fixture hook is needed
- `crates/nako-server/src/config/preflight.rs` only if fixture coverage reveals
  a missing preflight case

**Conflict surface**:

- Operations/release packaging changes in the same scripts.
- Any concurrent task changing `NetworkAccessConfig`, `config-check`, trusted
  proxy policy, or Admin network DTOs.

**Suggested validation**:

```powershell
cargo nextest run -p nako-server config_preflight
cargo nextest run -p nako-server network_boundary
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
git diff --check
```

**Next PR after that**:
`admin-network-diagnostics-drilldown`, only if Admin contract scope is free and
the page stays read-only.

## Priority 3: Source Fingerprint Hash Triggering And Reconciliation Policy

**Recommended first PR**:
`source-fingerprint-hash-trigger-policy-research`.

**Why this is product value, not pure refactor**:

Nako now has real Source Fingerprint hash execution and evidence persistence.
The next product question is how operators and scans use that evidence: manual
enqueue, retry/requeue, scan-originated enqueue, scheduling policy, and
reviewable Source Duplicate Relationship suggestions. This is storage hygiene
and duplicate management for real libraries, not module cleanup.

**Parallelizability**: `architecture-first`.

The first PR should be policy/research or architecture-map/PRD only. A later
implementation must serialize with scan scheduling, durable job migration,
Admin job DTOs, and source identity repositories.

**Likely file scope for the first PR**:

- `.trellis/tasks/06-05-source-hash-triggering-reconciliation-policy/research/*.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LANES.md`

**Likely implementation scope after policy is accepted**:

- `crates/nako-library/src/source_hash.rs`
- `crates/nako-server/src/app/source_hash.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-core/src/media/source.rs`
- `crates/nako-core/src/repository/media.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-db/src/sqlite/source_duplicate.rs`
- `crates/nako-db/src/postgres/core_catalog.rs`
- Admin Jobs/source diagnostics DTOs only with a contract owner

**Conflict surface**:

- Durable job scheduler migration.
- Library scan scheduling and watcher/debounce productization.
- Source Duplicate Relationship mutation or schema changes.
- Admin/API mutation routes or generated Admin contract refreshes.

**Suggested validation for the first implementation PR**:

```powershell
cargo nextest run -p nako-core source_fingerprint
cargo nextest run -p nako-library source_hash
cargo nextest run -p nako-server source_fingerprint_hash
cargo nextest run -p nako-server admin_v1_jobs_lists_source_fingerprint_hash_filters_without_payload_leaks
cargo fmt --all -- --check
git diff --check
```

**First executable slice after policy**:
Manual Admin enqueue or scan-originated enqueue, but not automatic duplicate
relationship mutation in the same PR.

## Priority 4: Host-Owned Addon Resource Flow Pattern

**Recommended first PR**:
`host-owned-addon-resource-flow-pattern-audit`.

**Why this is product value, not pure refactor**:

Addon Resource Search, subtitle import, and external acquisition are how Nako
becomes extensible without trusting Addon Sidecars with host state. Users will
experience this as "find a resource, choose it, preview/apply safely, and see
diagnostics." A shared host-owned pattern protects that product workflow before
Addon Manager and more resource types multiply divergent selection/apply
semantics.

**Parallelizability**: `architecture-first`.

The audit can run in parallel with playback and remote access work if it stays
server-local and avoids Admin DTO changes. Implementation should serialize with
new Addon resource/subtitle/acquisition product flows.

**Likely file scope for the first PR**:

- `.trellis/tasks/06-05-addon-resource-flow-pattern-audit/research/*.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/LANES.md`
- relevant ADR notes only if the pattern changes durable decisions

**Likely implementation scope after the pattern is accepted**:

- `crates/nako-server/src/app/addons/resource_search.rs`
- `crates/nako-server/src/app/addons/subtitles.rs`
- `crates/nako-server/src/app/addons/external_acquisition.rs`
- `crates/nako-server/src/app/addons/diagnostics.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/admin/addons.rs`
- `apps/admin-web/src/features/addons/AddonsPage.tsx`

**Conflict surface**:

- Addon Manager Admin routes/generated contract work.
- New Resource Search, subtitle, or external acquisition feature slices.
- Addon task/event runtime convergence.
- Any change that tries to push host policy into `nako-addon-protocol`.

**Suggested validation for the first implementation PR**:

```powershell
cargo nextest run -p nako-addon-protocol resource_search subtitle external_acquisition
cargo nextest run -p nako-server addon
cargo nextest run -p nako-server admin_v1
cargo fmt --all -- --check
git diff --check
```

**First executable slice after policy**:
Extract a small server-local selected-reference/apply-plan/safe-error/redaction
helper used by one existing flow first. Do not start with Addon Manager UI or
process lifecycle.

## Priority 5: VFS Cache Non-Destructive Remediation Planning

**Recommended first PR**:
`vfs-cache-non-destructive-remediation-plan`.

**Why this is product value, not pure refactor**:

Remote storage and VFS cache failures are operator-visible failures: stale
fallbacks, refresh failures, unavailable backends, and unknown states affect
scan, probe, playback, and library trust. Nako already has target inventory,
preview, action plan, and selected-target refresh. The next product step is to
give operators a safe remediation plan before durable repair queues or
destructive writes.

**Parallelizability**: `can parallel`, with DTO caveats.

It can run beside playback or Addon planning. It should not run with another
storage/Admin diagnostics task that edits the same DTOs, generated Admin
contract, or Admin Web storage route state.

**Likely file scope**:

- `crates/nako-api/src/admin/storage.rs` or related storage Admin DTO module
- `crates/nako-server/src/http/admin.rs`
- VFS cache app/service modules under `crates/nako-server/src/app/`
- `apps/admin-web/src/features/storage*` or storage settings pages if UI is in
  scope
- `docs/architecture/STORAGE_VFS.md`

**Conflict surface**:

- Admin contract generation and route inventory changes.
- Any source hash Admin/API mutation work.
- Durable repair queue implementation.
- Storage backend health/circuit breaker DTO changes.

**Suggested validation**:

```powershell
cargo nextest run -p nako-server admin_v1_vfs_cache
cargo nextest run -p nako-api admin_contract
pushd apps/admin-web; npm run check; npm run test; popd
cargo fmt --all -- --check
git diff --check
```

**First executable slice**:
A read-only remediation classifier plus plan response that classifies current
unresolved targets and recommends safe next actions. Do not add durable repair
jobs or destructive library writes in the same PR.

## Directions To Avoid For Now

- **HEVC/AV1 executable output**, **hardware tone-map execution**, and
  **image subtitle burn-in execution** before Public Client capability parity
  and the output profile v2 skeleton land.
- **Built-in tunnel provider runtime**, Cloudflare/Tailscale/ngrok process
  supervision, STUN/TURN/relay, or endpoint discovery implementation before
  the remote access cookbook and client endpoint-selection architecture exist.
- **Automatic Source Duplicate Relationship mutation** from hash evidence
  before trigger/reconciliation policy, confidence/staleness rules, operator
  review, and rollback semantics are accepted.
- **Addon Manager process lifecycle**, Docker socket access, auto-update,
  signing, package inventory, or hosted-page credential mediation before the
  host-owned resource-flow pattern and Admin Addon surface owner are decided.
- **Broad fearless refactor campaigns**: mechanical splitting of
  `nako-addon-protocol`, `nako-addon-client`, or `nako-official-addon-catalog`;
  generic disk-scan job executor registry with only two job variants; broad
  playback artifact identity cleanup before a playback product target touches
  the same files; Admin contract generator deepening without active DTO churn.
- **Public Client endpoint discovery implementation** in parallel with playback
  SDK/profile work. It crosses client bootstrap, auth, remote transport,
  playback tickets, renderer/cast transport, SDKs, and docs.

## Recommended Parallel Queue Shape

1. **Contract owner lane**: Public Client playback capability parity.
2. **Low-conflict ops lane**: remote access cookbook and config fixtures.
3. **Storage/control-plane planner lane**: source hash trigger/reconciliation
   policy.
4. **Addon planner lane**: host-owned Addon resource flow pattern.
5. **Optional storage operator lane**: VFS cache non-destructive remediation,
   only if Admin storage DTO scope is free.

This queue keeps product value high while avoiding the shared surfaces most
likely to serialize all agents: generated contracts, Admin DTOs, Public Client
playback DTOs, durable job scheduler policy, and source identity mutation.
