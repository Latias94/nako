# Addon Library File Write Policy

Status: Completed
Last updated: 2026-05-19

This workstream owns the subtitle, NFO, and sidecar-asset Library File Write
follow-on split from `addon-protected-writes`. APW proved Addon Side Effect
apply semantics with Canonical Metadata. This lane decides how addon-initiated
file writes enter Taru through storage/VFS, NFO Round Trip, backup retention,
and redacted diagnostics instead of raw path writes.

Closeout outcome: Taru now has the first concrete addon-initiated Library File
Write path. Accepted MediaSource-targeted `library_file_write` side effects can
request Taru-owned NFO Export with a typed intent payload, synchronous
first-party NFO/VFS execution, redacted aggregate `apply_report`, and
idempotent replay. Subtitle writes, broader NFO import/export behavior, queued
file-write execution, and arbitrary sidecar asset writes remain follow-on
scope.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../addon-protected-writes/`
- `../nfo-round-trip-preservation/`
- `../nfo-storage-write-policy/`
- `../nfo-sidecar-backup-policy/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Define the first safe addon-initiated Library File Write path.
- Route subtitle, NFO, and sidecar-asset writes through Taru-owned target
  derivation, storage/VFS write modes, backup policy, and diagnostics.
- Preserve Addon Token, accepted permission, Media Library grant, idempotency,
  audit, and redaction behavior from APW.
- Ensure Addon Sidecars never receive raw Source Locators, filesystem paths,
  remote storage handles, or unredacted file-write reports.

## Non-Goals

- No direct addon path writes.
- No Public Client write API expansion.
- No general storage/VFS rewrite unless the audit proves the existing policy is
  insufficient.
- No artwork-specific Managed Artwork work; that belongs to
  `addon-managed-artwork-artifacts`.

## Follow-Ons

- Subtitle file writes need a first-party subtitle/track model before addons
  can submit or replace subtitle sidecars.
- Broader NFO behavior should stay on first-party NFO import/export seams and
  must use `commit_nfo_import` for NFO-derived Canonical Metadata.
- Arbitrary sidecar asset writes need their own content-type, target
  derivation, backup, and redaction matrix.
- Queued or durable Library File Write execution needs truthful queued/job
  association semantics before `apply_status` can represent deferred work.
