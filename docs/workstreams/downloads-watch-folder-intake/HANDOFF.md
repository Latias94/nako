# Downloads / Watch-Folder Intake — Handoff

Status: Complete
Last updated: 2026-05-22

## Current State

This completed lane was opened as the next mainline child of
`post-rpd-product-hardening` after Playback/Transcode Ops Hardening closed.

The prerequisites are complete:

- `metadata-provider-breadth` made provider capability, match ambiguity, and
  cross-provider conflict review explainable.
- `nfo-link-authority` made local NFO/link authority and duplicate evidence
  non-mutating and explicit.
- `managed-import-staging` added durable Managed Import artifacts and
  non-mutating promotion preview.
- `link-apply-and-import-promotion` added accepted promotion apply, VFS-mediated
  target creation, catalog commit, duplicate evidence, and cleanup/rollback
  audit.
- `nfo-sidecar-promotion-apply` added accepted NFO sidecar import/export apply,
  backup, retention, rollback/repair, and redacted diagnostics.
- `playback-transcode-ops-hardening` added playback readiness, validation,
  failure taxonomy, and bounded Admin support evidence.

DWI-010 is complete. The lane is scoped to acquisition intake and watch-folder
candidate discovery, not built-in downloader protocols or direct library writes.

DWI-020 is complete. It added acquisition intake candidate IDs, source kinds,
states, records, repository traits, SQLite/PostgreSQL migrations and adapters,
facade dispatch, backend capability flags, and a backend-neutral contract for
round-trip, idempotent source-key lookup, state transitions, Managed Import
artifact linking, and list filters. It does not create Media Sources, run
promotion apply, or write library files.

DWI-030 is complete. It added `AcquisitionIntakeAppService`, TaruApp service
composition wiring, redacted candidate diagnostics, idempotent candidate
record/list behavior, explicit existing Managed Import artifact linking,
same-source artifact reuse, and new Managed Import artifact creation. Tests
prove no promotion apply, Media Source creation, or library file mutation.

DWI-040 is complete. It added watch-folder discovery through configured
storage/VFS list/stat boundaries, ready/incomplete/unsupported classification,
idempotent intake record writes, and redacted discovery diagnostics. Tests prove
no Managed Import artifact creation, Media Source creation, promotion apply, or
Library File Write.

DWI-050 is complete. It added Admin API v1 DTOs/routes for acquisition-intake
candidate diagnostics and watch-folder discovery, safe root URI parsing errors,
synchronized the generated Admin TypeScript contract, and updated the Admin web
typed client, mocks, data source, and console surface. Tests prove the routes
are Admin-only, redacted, and do not change Public Client API or
`taru-client-protocol`.

DWI-060 is complete. Final closeout gates passed, this workstream is marked
complete, protocol downloader/background scheduling/UI/network/AI/Addon
follow-ons were split rather than hidden in this lane, and the next lane
decision returned to `post-rpd-product-hardening`.

## Closeout State

- Task ID: DWI-060
- Status: DONE
- Final scope:
  - `docs/workstreams/downloads-watch-folder-intake`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Review result: no blocking findings. The target state is met, and remaining
  work is split into follow-ons rather than hidden in this lane.

## Decisions Since Opening

- The lane name is Downloads / Watch-Folder Intake, but the first implementation
  slice is acquisition-intake domain/persistence.
- Watch folders are candidate sources, not trusted library roots.
- Intake candidates are not Media Sources.
- Intake acceptance creates or links Managed Import artifacts; promotion apply
  and NFO sidecar apply remain separate accepted workflows.
- VFS/storage list/stat primitives should own path safety for watch-folder
  discovery.
- Admin diagnostics are allowed; Public Client API and `taru-client-protocol`
  changes are not.
- Protocol-specific download clients, network traversal, AI, Addon runtime, UI
  polish, background scheduling, and automatic apply behavior are follow-ons
  unless explicitly opened.
- DWI-020 kept the boundary persistence-only. Candidate acceptance links a
  Managed Import artifact at the repository level, but app-service semantics
  for creating or reusing artifacts belong to DWI-030.
- DWI-030 made app-service acceptance an explicit handoff boundary: it can link
  a requested existing artifact, reuse a same-source existing artifact, or
  create a new `Proposed` Managed Import artifact. It never applies promotion or
  writes catalog/library state.
- DWI-040 made watch-folder discovery read-only and storage-owned: it uses
  configured storage/VFS list/stat, writes only acquisition-intake candidate
  records, and leaves artifact creation to explicit DWI-030 acceptance.
- DWI-050 made intake diagnostics Admin-only and contract-owned. It exposes
  redacted references and fingerprints, not raw source URIs, source keys,
  display names, intended locators, diagnostics JSON, raw root URIs, or
  downloader internals. Public Client API and `taru-client-protocol` remain
  unchanged.
- DWI-060 closed the lane. Torrent/Usenet/download-client adapters,
  background scan scheduling, Admin UI workflow polish, remote network access,
  AI-generated artifacts, Addon runtime/distribution, automatic promotion
  apply, and NFO sidecar mutation shortcuts remain separate workstreams or
  backlog follow-ons.

## Blockers

- None.

## Next Recommended Action

Return to `post-rpd-product-hardening`.

Recommended next mainline lane: open `network-access-boundary`. It is now the
highest-value product hardening lane because metadata, local authority,
Managed Import, accepted promotion/NFO apply, playback supportability, and
acquisition intake boundaries are proven. Scope the first slice to endpoint,
trusted proxy, and tunnel-provider policy; do not add built-in NAT traversal
runtime until the policy/readiness boundary is explicit.

Split follow-ons if needed:

- protocol downloader adapters that submit candidates into Acquisition Intake;
- background watch-folder scheduling / OS watcher runtime;
- Admin web intake workflow polish beyond typed diagnostics;
- AI generated artifact proposal queues and acceptance workflow;
- Addon runtime/distribution and side-effect permission UX.
