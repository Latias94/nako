# Addon Library File Write Policy

Status: Completed
Last updated: 2026-05-19

## Why This Lane Exists

Subtitle, NFO, and sidecar-asset writes are more dangerous than Canonical
Metadata writes because they cross from database state into Media Library
storage. Taru already has NFO Round Trip, VFS write modes, atomic replace, and
backup policy workstreams. Addon-initiated file writes must reuse those seams
instead of accepting raw addon-provided paths.

## Problem

Accepted Addon Side Effects can now record and apply Canonical Metadata, but
Library File Writes still need explicit rules for:

- deriving target sidecar locations without exposing Source Locators or paths;
- deciding overwrite, atomic replace, and backup behavior;
- preserving NFO Round Trip content where applicable;
- representing subtitle imports versus NFO exports versus arbitrary sidecar
  assets;
- safely reporting write outcomes without leaking local paths or remote handles;
- idempotent replay after file writes.

## Target State

- Addon file-write side effects move through intake and explicit apply outcome
  state.
- Taru derives the target Library File Write from a media target, file role,
  and policy, not from an addon-provided absolute path.
- NFO writes preserve NFO Round Trip and backup policy.
- Subtitle and sidecar writes use storage/VFS write modes with bounded
  diagnostics.
- Responses and audit summaries expose safe codes and IDs, not raw Source
  Locators, filesystem paths, remote storage handles, or full payloads.

## In Scope

- Audit NFO import/export, VFS write, backup, subtitle, and Addon Side Effect
  seams.
- Decide the first Library File Write target: subtitle import, NFO export, or a
  narrower sidecar asset.
- Define target/path derivation and backup policy.
- Define redacted write reports and idempotent replay semantics.
- Update HTTP API docs and workstream evidence for shipped behavior.

## Out Of Scope

- Direct addon filesystem or remote storage access.
- Addon Manager lifecycle automation.
- Public Client write routes.
- Artwork/Managed Artwork behavior.
- General storage backend redesign outside the selected first write path.

## Architecture Direction

Reuse the APW three-stage model:

1. Addon runtime route authenticates, validates permission/library/target,
   persists the side-effect record, and returns redacted summaries.
2. File-write validation derives a Taru Library File Write command from a media
   target, sidecar role, content type, and policy.
3. Domain apply calls NFO/storage/VFS services, records backup/report summary
   safely, and stores an apply outcome.

If the write can block on remote storage, backup pruning, or large payloads,
prefer a queued Addon Task or durable job over a synchronous runtime request.

### Core Architecture Alignment

Addon-initiated NFO behavior must reuse the first-party NFO boundaries. If an
accepted addon request imports or applies NFO-derived canonical metadata, it
should route through `taru-nfo` planning and
`MetadataRepository::commit_nfo_import` /
`NfoImportPersistenceCommit`; it must not reintroduce ordered calls to media
item upsert, field-lock upsert, hierarchy confirmation, catalog hydration, or
search refresh from the Addon handler.

Addon-initiated file writes that create, replace, or reclassify discoverable
library files must reuse first-party scan/indexing boundaries. If a file-write
apply path needs to update Media Source, Source State, Local Inference
Evidence, Library Item State, failure resolution, or search projection, it
should route through `ScanRepository::commit_library_scan_source`,
`LibraryIndexRepository`, or a new first-party commit unit. It must not invent a
parallel Addon-specific sequence of source, state, evidence, and projection
writes.

NFO export remains an NFO/VFS concern: derive the sidecar target inside Taru,
write through `StorageWriteRequest`/backup policy, and trigger any follow-on
NFO import or catalog update through the existing NFO service path rather than
duplicating parser or persistence logic in the Addon handler.

### ALFW-020 Selected First Target

The first ALFW-030 apply target is an addon-initiated, MediaSource-targeted
Taru-owned NFO Export. An accepted `library_file_write` side effect may request
an NFO export for an existing Media Source, but the addon may not provide a
filesystem path, Source Locator, remote storage handle, or raw NFO content.
Taru derives the sidecar URI from the Media Source, library, file role, and
write policy, then delegates the write to first-party NFO/VFS code.

The first payload shape should be typed around intent, for example NFO export
with create-missing or replace-existing-preserving policy. Create-missing skips
existing sidecars. Replace-existing-preserving must use NFO Round Trip rendering,
`StorageWriteRequest::atomic_replace`, existing-file backup, and backup
retention diagnostics.

Do not mark a side effect `applied` merely because a durable job was queued. If
ALFW-030 uses the existing NFO export job path, it must add truthful queued/job
association semantics or a redacted apply-report boundary before reporting the
write as applied. If it stays synchronous, it must only mark `applied` after the
NFO/VFS write has completed.

The apply response and stored report should expose only redacted facts:
side-effect ID, target IDs, file role, policy, status, optional job ID, safe
error code, and aggregate backup/pruning counts. It must not expose raw
`StorageWriteReport`, `StorageUri`, Source Locator, filesystem path, remote
handle, backup URI, or payload content.

Subtitle import and arbitrary sidecar asset writes are deliberately deferred.
Subtitle needs a first-party subtitle/track model and source semantics before
addons can safely submit subtitle files. Arbitrary sidecar assets need a
content-type and target-derivation matrix broader than NFO export.

## Closeout Condition

This lane can close when:

- current subtitle/NFO/storage seams are audited;
- one bounded Library File Write path is implemented or deliberately split;
- target derivation, write mode, backup policy, idempotency, and redaction are
  tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.

## Closeout Outcome

This lane is closed after ALFW-040. ALFW-020 selected MediaSource-targeted
Taru-owned NFO Export as the first bounded Library File Write target, and
ALFW-030 implemented it through the existing Addon Side Effect, NFO, VFS,
backup, and redaction seams.

The shipped path is intentionally narrow:

- the only accepted file role is `nfo`;
- the only accepted target kind is `media_source`;
- the addon supplies policy intent, never paths, Source Locators, remote
  handles, backup URIs, or raw NFO XML;
- `create_missing` skips existing sidecars;
- `replace_existing_preserving` uses NFO Round Trip rendering, VFS atomic
  replace, existing-file backup, and retention diagnostics;
- responses expose only IDs, safe status fields, and aggregate redacted
  counters.

Remaining subtitle, broader NFO, arbitrary sidecar asset, and queued
file-write execution behavior is deliberately deferred. Each follow-on should
open with its own target derivation, content validation, write policy, and
redaction matrix instead of widening this completed first-slice lane.
