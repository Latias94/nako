# Addon Library File Write Policy

Status: Proposed
Last updated: 2026-05-18

This workstream owns the subtitle, NFO, and sidecar-asset Library File Write
follow-on split from `addon-protected-writes`. APW proved Addon Side Effect
apply semantics with Canonical Metadata. This lane decides how addon-initiated
file writes enter Taru through storage/VFS, NFO Round Trip, backup retention,
and redacted diagnostics instead of raw path writes.

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
