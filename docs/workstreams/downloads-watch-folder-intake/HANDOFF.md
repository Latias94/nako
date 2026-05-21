# Downloads / Watch-Folder Intake — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is open as the next mainline child of `post-rpd-product-hardening`
after Playback/Transcode Ops Hardening closed.

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

## Active Task

- Task ID: DWI-060
- Owner: unassigned
- Files:
  - `docs/workstreams/downloads-watch-folder-intake`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Validation:
  - `verify-rust-workstream` records fresh final evidence
  - `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`
  - `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`
  - `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`
  - `cargo nextest run -p taru-api admin_contract --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - `npm run check` from `apps/admin-web`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: READY
- Review: Closeout must split protocol downloader integration, Admin UI polish,
  background scan scheduling, network traversal, AI, and Addon runtime instead
  of hiding them in this lane.

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

## Blockers

- None for DWI-060.

## Next Recommended Action

Execute DWI-060 closeout:

1. run final focused gates with fresh evidence;
2. decide whether downloads protocol adapters, background watch scheduling, and
   Admin UI polish become follow-on workstreams or backlog notes;
3. re-score the post-RPD umbrella so network access boundary, AI-assisted
   library ops, and Addon runtime/distribution have a clear next-lane order;
4. close this lane only if review finds no boundary leaks.
