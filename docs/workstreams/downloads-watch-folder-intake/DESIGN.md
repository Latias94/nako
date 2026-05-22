# Downloads / Watch-Folder Intake Design

Status: Complete
Last updated: 2026-05-22

## Why This Lane Exists

Nako has now proven the high-risk foundations that must precede acquisition
breadth:

- explainable metadata matching and provider conflict review;
- NFO/local authority preview and link evidence;
- durable Managed Import artifacts and non-mutating promotion planning;
- accepted promotion apply with VFS-mediated target creation, catalog commit,
  duplicate evidence, rollback/cleanup, and audit;
- accepted NFO sidecar import/export apply with backup, retention,
  rollback/repair, idempotency, and redacted diagnostics;
- playback/transcode readiness, validation, failure taxonomy, and bounded Admin
  support evidence.

Those boundaries make acquisition safe to open, but only if acquisition remains
an intake lane. A downloader/watch-folder feature that writes directly into
library roots would bypass the very file-write and authority gates Nako just
built.

## Relevant Authority

- ADRs:
  - `docs/adr/0002-internal-vfs-before-os-mounting.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Existing docs:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/managed-import-staging`
  - `docs/workstreams/link-apply-and-import-promotion`
  - `docs/workstreams/nfo-sidecar-promotion-apply`
  - `docs/workstreams/playback-transcode-ops-hardening`
  - `docs/workstreams/storage-vfs`

## Problem

Operators need practical acquisition entry points:

- watch a folder where files appear after manual copy or an external downloader;
- add an operator-provided file or URL candidate for later staging;
- explain whether a candidate is safe, duplicate, incomplete, unsupported, or
  ready for promotion planning;
- hand the result into existing Managed Import and accepted apply workflows.

The risk is that Nako could blur four boundaries:

- a watched candidate is not yet a **Media Source**;
- a discovered file is not automatically accepted for promotion;
- an external downloader's output is not trusted simply because it is present on
  disk;
- a downloader protocol is not part of core Nako until intake, staging,
  permission, retry, and security policy are proven.

## Target State

When this lane closes:

- Nako has an explicit acquisition intake vocabulary for watch-folder and
  operator-submitted candidates.
- Candidate identity is stable and idempotent enough to prevent duplicate intake
  records across repeated scans.
- Watch-folder scans classify candidates without directly creating Media
  Sources or writing library files.
- Candidate diagnostics are redacted: no raw host paths, credentials, secret
  query strings, or private downloader internals in Admin-facing evidence.
- Accepted candidates create or link to Managed Import artifacts and reuse
  existing promotion preview/apply and NFO sidecar apply boundaries.
- Admin-only read models expose intake state and blockers without changing the
  Public Client API or `nako-client-protocol`.
- Protocol-specific downloader clients, remote tunnel behavior, AI generation,
  and Addon runtime/distribution are split follow-ons.

## In Scope

- Workstream docs and task ledger for downloads/watch-folder intake.
- Core acquisition intake domain records, IDs, states, and repository traits.
- SQLite/PostgreSQL schema and backend-neutral repository contracts.
- Watch-folder candidate discovery through Nako storage/VFS listing and stat
  primitives, not direct OS path traversal in server business logic.
- Idempotent intake from discovered candidates into Managed Import artifacts.
- Redacted Admin diagnostics and route/contract sync for intake state.
- Focused tests proving no direct library writes, no Media Source creation, and
  no Public Client API changes.

## Out Of Scope

- Torrent, Usenet, browser, RSS, or download-client protocol integrations.
- Built-in NAT traversal, reverse proxy, or Network Tunnel Provider runtime.
- AI model execution or autonomous metadata/file writes.
- Addon manager, Addon runtime distribution, or in-process plugins.
- Automatic promotion apply.
- Direct NFO sidecar writes or promotion target writes outside existing apply
  workflows.
- Public Client API, mobile/offline downloads, or generated Public Client SDK
  changes.
- Durable Optimized Versions or playback transcode caching.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Managed Import artifacts are the right downstream object for acquisition output. | High | `managed-import-staging`, `link-apply-and-import-promotion`, and `nfo-sidecar-promotion-apply` closeouts | If the artifact model is missing fields, extend it explicitly rather than introducing a parallel import path. |
| Watch-folder discovery should use VFS/storage boundaries. | High | ADR 0002 and existing `nako-vfs::StorageBackend::list/stat` | If a local filesystem watcher becomes necessary, wrap it behind a storage/intake adapter and keep path safety/redaction centralized. |
| The first slice should be polling/discovery, not background protocol download. | High | Post-PTOH re-score and Managed Import staging non-goals | If external download clients are urgent, split a protocol adapter lane that submits candidates into this intake boundary. |
| Admin-only read models are enough for the first intake surface. | Medium | ADR 0027 and existing Admin API pattern | If operator UX requires full UI, split an Admin web workflow task after API/read-model evidence is stable. |
| Candidate idempotency can start from library, source kind, normalized redacted locator, size, and fingerprint facts. | Medium | Existing Managed Import source lookup and Source Fingerprint semantics | If false positives appear, add a stronger candidate key contract before enabling automatic accept. |

## Architecture Direction

Keep acquisition as a producer of Nako-owned artifacts, never as a bypass around
promotion/apply:

```text
nako-core
  Owns acquisition candidate IDs, source kinds, states, candidate evidence, and
  repository traits. It may reference Managed Import artifacts but should not own
  storage mutation.

nako-db
  Owns SQLite/PostgreSQL schema, repository adapters, and backend-neutral
  contract tests for candidate idempotency and state transitions.

nako-vfs / storage adapters
  Own listing/stat/probe primitives and path safety. Watch-folder scans consume
  these primitives instead of walking host paths directly in app code.

nako-server::app::managed_import / acquisition intake
  Owns scan orchestration, candidate classification, redacted diagnostics,
  Managed Import artifact creation/linking, and handoff to promotion preview.

nako-api::admin and nako-server::http::admin
  Own Admin-only DTOs and routes for intake diagnostics. They must not leak into
  `nako-client-protocol`.
```

### Vocabulary

```text
Acquisition Intake
  The Nako-owned boundary that receives discovered files, operator candidates,
  or future external downloader outputs before they become Managed Import
  artifacts.

Watch Folder
  A Media Library-scoped storage location that Nako scans for candidates. It is
  not a trusted library root and does not imply automatic promotion.

Intake Candidate
  A discovered or submitted object with source evidence, lifecycle state,
  duplicate hints, and redacted diagnostics.

Intake Acceptance
  The explicit or policy-controlled step that creates or links a Managed Import
  artifact from an Intake Candidate. It does not promote into a Media Library.
```

### First Slice Recommendation

Open with a durable intake/domain slice:

1. Add candidate IDs, source kind, state, repository trait, and SQLite/PostgreSQL
   schema for intake candidates.
2. Add idempotent candidate upsert/list/filter contract tests.
3. Add app service methods to record a watch-folder candidate and convert an
   accepted candidate into a Managed Import artifact without library writes.
4. Keep actual directory polling and Admin HTTP surfaces for later vertical
   slices once the candidate lifecycle is stable.

## Closeout Condition

This lane can close when:

- [x] intake candidates have durable backend-neutral persistence;
- [x] watch-folder discovery produces redacted, idempotent candidate evidence;
- [x] accepted candidates create or link Managed Import artifacts;
- [x] Admin diagnostics expose intake state safely;
- [x] tests prove no direct Media Source creation, no direct library file
  writes, and no Public Client API or `nako-client-protocol` changes;
- [x] protocol downloader integrations, network access, AI, Addon runtime, UI
  polish, and automatic apply behavior are either split or explicitly
  deferred.

## Closeout Result — 2026-05-22

This lane is complete. Nako now has a narrow acquisition-intake boundary that
can record watch-folder/operator candidates, classify discovery results through
storage/VFS list/stat, expose redacted Admin diagnostics, and explicitly hand
accepted candidates into Managed Import artifacts.

The following remain outside this lane by design:

- torrent, Usenet, browser, RSS, or download-client protocol adapters;
- background watch-folder scheduling or OS file watcher runtime;
- full Admin UI workflows beyond typed diagnostics;
- automatic promotion apply or NFO sidecar mutation shortcuts;
- remote access/tunnel behavior;
- AI-generated artifact proposals or autonomous writes;
- Addon runtime/distribution.
