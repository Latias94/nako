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

## Active Task

- Task ID: DWI-050
- Owner: unassigned
- Files:
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-api/src/admin_contract.rs`
  - `crates/taru-server/src/http/admin.rs`
  - `crates/taru-server/src/http/tests`
  - `apps/admin-web/src/adminApi`
- Validation:
  - `cargo nextest run -p taru-api admin_contract --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - `npm run check` from `apps/admin-web`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: READY
- Review: Admin diagnostics/read model must remain Admin-only, redacted, typed,
  and must not change Public Client API or `taru-client-protocol`.

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

## Blockers

- None for DWI-050.

## Next Recommended Action

Execute DWI-050 with TDD:

1. add Admin API contract tests for listing intake candidate diagnostics and
   triggering/readback of watch-folder discovery diagnostics;
2. add Admin HTTP tests proving redaction and auth/admin-only boundaries;
3. sync the Admin web typed API/client without changing Public Client API or
   `taru-client-protocol`;
4. run the Admin API, HTTP, web, and public-client-boundary gates before moving
   to DWI-060 closeout.
