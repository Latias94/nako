# Post-RPD Product Hardening — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

The post-RPD product roadmap is open as an umbrella. It chooses
`metadata-provider-breadth` as the first execution lane and records NFO/link,
playback/transcode, managed import, network, AI, and addon distribution as
ordered follow-ons. `metadata-provider-breadth` is complete. `nfo-link-authority`
is complete with VFS link dry-run diagnostics, Source Duplicate Relationship
filesystem-link suggestions, NFO authority preview, and the link apply split
decision. `managed-import-staging` is complete as a non-mutating staging and
promotion-preview lane. `link-apply-and-import-promotion` is complete after
implementing accepted promotion apply, VFS-mediated target creation, catalog
commit ordering, duplicate evidence, and cleanup-complete/cleanup-pending
audit. LAIP-070 split NFO sidecar import/export mutation to
`nfo-sidecar-promotion-apply`. PRPH-080 selected
`nfo-sidecar-promotion-apply` as the next mainline execution lane. NFO Sidecar
Promotion Apply is now complete with accepted import/export apply, VFS
write/restore, canonical metadata/local authority, hierarchy confirmation,
rollback/repair, idempotent replay, and redacted diagnostics. PRPH-090 selects
Playback/Transcode Ops Hardening as the next mainline lane, and PRPH-100 opens
`playback-transcode-ops-hardening`. Playback/Transcode Ops Hardening is now
complete with Admin runtime readiness, pre-session validation, failure
taxonomy, and bounded Admin support evidence. PRPH-110 selects
downloads/watch-folder intake as the next mainline lane. PRPH-120 opens
`downloads-watch-folder-intake`. DWI-020 durable intake candidate
domain/persistence and DWI-030 app-service intake / Managed Import handoff are
complete. DWI-040 watch-folder discovery, DWI-050 Admin-only intake
diagnostics/read model, and DWI-060 closeout are complete.
`network-access-boundary` is complete with policy/config validation, HTTP
boundary enforcement, Admin readiness diagnostics, and closeout. PRPH-150
selected `ai-assisted-library-ops` next. AI Assisted Library Ops is complete
with Generated Artifact proposal/readiness, Admin diagnostics, explicit
accept/reject planning, and closeout. PRPH-170 opened
`addon-runtime-and-distribution` as the current mainline lane.

## Active Task

- Task ID: `addon-runtime-and-distribution` ARD-030
- Owner: codex
- Files: `crates/taru-addon-client`, `crates/taru-api/src/extension.rs`,
  `crates/taru-server/src/app`, `crates/taru-server/src/http/addons.rs`,
  `docs/workstreams/addon-runtime-and-distribution`
- Validation: focused addon-client/app/Admin runtime readiness tests; `cargo
  nextest run -p taru-server addons --no-fail-fast`; `cargo fmt --all -- --check`;
  `git diff --check`; `git diff --name-only -- crates/taru-client-protocol`.
- Status: READY
- Review: keep runtime readiness diagnostics separate from Addon Manager
  discovery/install/update, marketplace, package signing, process supervision,
  Native Plugin ABI, direct library writes, Public Client API churn, and
  `taru-client-protocol` changes.
- Evidence: PRPH-170 lane scoring in `DESIGN.md`, AI Assisted Library Ops
  closeout evidence, Addon Architecture Deepening, Admin Addon Operations MVP,
  Downloads / Watch-Folder Intake, Network Access Boundary, and completed
  side-effect / proposal / intake boundaries. ARD-020 completed package/install
  descriptor validation and redacted Admin install-guide preview.

## Decisions Since Last Update

- Do not implement a generic downloads lane first.
- Treat downloads as `managed-import-staging` after metadata and local file
  authority are stronger.
- Start Wave 1 with metadata provider capability, matching, and conflict
  explanation rather than UI or AI breadth.
- Close Wave 1 before downloads/import because provider identity and ambiguity
  must be explicit first.
- Choose `nfo-link-authority` as the next mainline lane because it is the
  remaining high-risk local data-loss boundary.
- Open `nfo-link-authority` with VFS link dry-run diagnostics as the first
  non-destructive slice.
- Record filesystem-link evidence as suggested Source Duplicate Relationships
  without merging Media Sources or Media Items.
- Add non-mutating NFO authority preview before sidecar writes.
- Defer actual hardlink/symlink apply to a follow-on after managed import
  staging defines promotion, rollback, cleanup, audit, and source duplicate
  confirmation semantics.
- Managed Import Staging completed durable artifact records, redacted
  diagnostics, and non-mutating promotion preview.
- Actual promotion apply is split to `link-apply-and-import-promotion` because
  it requires operator confirmation, plan revalidation, durable audit,
  rollback/cleanup, VFS-only mutation, and catalog consistency.
- `link-apply-and-import-promotion` is complete with fresh closeout evidence for
  accepted promotion apply, VFS-mediated target creation, catalog commit
  ordering, duplicate evidence, and cleanup-complete/cleanup-pending audit.
- NFO sidecar import/export mutation is split to
  `nfo-sidecar-promotion-apply` because it is a separate accepted **Library
  File Write** and metadata-authority workflow with backup, retention,
  field-lock, hierarchy-confirmation, rollback/repair, idempotency, and
  redacted audit requirements.
- Keep playback/transcode ops hardening as a parallel sidecar candidate only
  if it stays diagnostic/runtime-focused and avoids NFO/import write scope.
- PRPH-080 selects `nfo-sidecar-promotion-apply` as the next mainline lane.
  Playback/transcode ops remains the safest parallel sidecar. Downloads and
  watch-folder acquisition remain downstream of accepted promotion and NFO
  sidecar apply boundaries.
- NFO Sidecar Promotion Apply is complete. PRPH-090 now selects
  Playback/Transcode Ops Hardening as the next mainline lane. Downloads/watch
  folder may be re-scored afterward, but must still enter through staged
  artifacts, promotion apply, and sidecar apply boundaries.
- `playback-transcode-ops-hardening` is opened. The first executable task is
  PTOH-020, which should harden runtime readiness diagnostics without changing
  Public Client API behavior.
- `playback-transcode-ops-hardening` is complete. It stayed Admin-only and
  runtime/diagnostic-focused, with Public Client API and `taru-client-protocol`
  unchanged.
- PRPH-110 selects downloads/watch-folder intake next. The correct shape is
  staged artifact acquisition and watch-folder candidate discovery; protocol
  downloaders, direct library writes, network traversal, AI writes, and Addon
  runtime are separate lanes.
- PRPH-120 opens `downloads-watch-folder-intake`.
- DWI-020 durable intake candidate domain/persistence is complete.
- DWI-030 app-service intake and Managed Import handoff is complete.
- DWI-040 watch-folder discovery is complete.
- DWI-050 Admin-only intake diagnostics/read model is complete with typed Admin
  web contract/client support and no Public Client protocol changes.
- DWI-060 Downloads / Watch-Folder Intake closeout is complete.
- PRPH-130 selects Network Access Boundary as the next recommended mainline
  lane. The first slice should harden endpoint, reverse-proxy/trusted-header,
  external URL, and tunnel-provider policy/readiness, not ship built-in NAT
  traversal runtime.
- PRPH-140 opened `network-access-boundary`.
- NAB-020 network policy domain/config validation is complete.
- NAB-030 HTTP boundary enforcement is complete with trusted proxy/source
  checks, origin enforcement, CORS preflight handling, health compatibility, and
  auth-order preservation.
- NAB-040 Admin-only network readiness diagnostics and NAB-050 closeout are
  complete.
- PRPH-150 selected AI Assisted Library Ops after Network Access Boundary
  closeout.
- PRPH-160 opened `ai-assisted-library-ops`.
- AILO-020 Generated Artifact proposal/readiness, AILO-030 Admin proposal
  diagnostics, AILO-040 explicit accept/reject planning, and AILO-050 closeout
  are complete.
- PRPH-170 selected and opened `addon-runtime-and-distribution` as the next
  mainline lane. The first slice is package/install descriptor and redacted
  install-guide readiness, not Addon Manager automation.

## Blockers

- None for ARD-030.

## Next Recommended Action

- Execute `addon-runtime-and-distribution` ARD-030.
- Add Admin-only runtime readiness diagnostics that classify sidecar
  reachability, protocol mismatch, manifest mismatch, grant gaps, missing
  Secret References, network policy blockers, and unsafe responses without
  echoing raw network errors or sidecar payloads.
- Keep Addon Manager discovery/install/update, marketplace hosting, package
  signing trust root, process/container supervision, Native Plugin ABI,
  downloader protocol adapters, local AI runtime, Public Client API changes,
  direct library writes, and `taru-client-protocol` changes as separate
  follow-ons.
