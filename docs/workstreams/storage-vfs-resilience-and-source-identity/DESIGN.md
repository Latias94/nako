# Storage/VFS Resilience And Source Identity — Design

Status: Active
Last updated: 2026-05-29

## Problem

Nako's storage architecture is directionally correct: local and WebDAV-backed
libraries go through VFS boundaries, scan state is durable, staging manifests
exist, and **Media Source** identity is scoped by **Media Library**. The next
failure mode is subtler: callers still see source identity as mostly a locator
plus an optional fingerprint string, while storage failures are handled close to
the current operation.

That shape is too shallow for a Jellyfin/Plex-class self-hosted library:

- a renamed file should preserve metadata and **User Playback State** when
  evidence is strong;
- stale VFS cache should not tombstone a valid source;
- slow NAS, WebDAV, SMB/NFS, and rclone-like mounts need bounded failure
  semantics;
- scan, probe, NFO, artwork, managed import, addon Library File Write, and
  playback input staging should share storage diagnostics vocabulary.

## Target State

Storage/VFS exposes a deeper host-owned workflow:

1. collect layered **Source Fingerprint** evidence without forcing expensive
   full-file hashes during normal scan;
2. classify storage failures into stable, redaction-safe categories;
3. reconcile moved or renamed **Media Sources** when evidence is strong enough;
4. create or update **Source Duplicate Relationships** when evidence suggests
   duplicate bytes without merging source identity automatically;
5. publish Admin diagnostics for stale cache, timeout, unavailable, permission,
   rate-limit, and partial-staging cleanup pressure.

## Scope

- `crates/nako-core`: source identity evidence records, repository contracts, or
  policy value types when needed.
- `crates/nako-vfs`: backend capability/error classification and bounded VFS
  behavior where adapter-owned.
- `crates/nako-library`: scan and ingestion workflow changes for source
  evidence and reconciliation.
- `crates/nako-db`: SQLite/PostgreSQL schema and repository parity if durable
  source evidence or diagnostics require new persistence.
- `crates/nako-server`: storage diagnostics, startup cleanup coordination, and
  scan job integration.
- Docs under `docs/architecture/` and this workstream.

## Non-Goals

- No Web UI or Media Web implementation.
- No HLS runtime feature work.
- No new storage backend such as SMB/NFS/S3 unless needed as a tiny fake adapter
  for tests.
- No mandatory full-file hashing for every source.
- No automatic duplicate merge into one **Media Item**.
- No built-in **Network Tunnel Provider** runtime.
- No provider breadth for TMDB, Douban, Bangumi, or addons.

## Architecture Direction

Prefer deepening the existing modules before adding crates. The likely target is
a workflow seam around source identity evidence and reconciliation, owned close
to `nako-library`/`nako-vfs` but expressed through `nako-core` records only when
durable data is required.

Keep the evidence policy layered:

- cheap evidence: library ID, source locator, size, modified time, backend ETag,
  object ID where available;
- media evidence: duration and stream facts after probe;
- stronger evidence: partial hash or full hash only when policy requires it.

The first slice should prove behavior through tests before schema breadth:

- stale cache avoids false tombstone;
- rename with strong evidence preserves item/source state;
- weak evidence creates a review/duplicate hint rather than merging;
- storage timeout is redaction-safe and does not block unrelated libraries.

## Risks

- Full-file hashes can overload NAS/cloud libraries. Keep them opt-in or
  escalation-only.
- Treating weak evidence as identity can corrupt libraries. Default to
  suggestions, not merges.
- OS mounts can block executor threads. Keep blocking filesystem operations
  behind explicit runtime budget and timeout policy where practical.
- Diagnostics can leak paths, URIs, ETags, fingerprints, or credentials. Expose
  booleans, safe categories, and fingerprints of sensitive values only.

