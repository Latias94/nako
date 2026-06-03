# Targeted Jellyfin Watcher Reference Evidence

Date: 2026-06-03

## Scope

This lane is a behavior-level reference pass for Nako watcher runtime
productization. It is limited to watcher lifecycle, event coalescing, debounce,
planned-write suppression, fallback reconciliation signals, and operator-facing
configuration. It is not an implementation lane.

Approved Jellyfin source paths:

- `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/ServerConfiguration.cs`

Research artifacts:

- `research/lifecycle-events-debounce.md`
- `research/suppression-fallback-config-licensing.md`

## Behavior Summary

### Lifecycle

- Jellyfin exposes an explicit monitor contract with start/stop, planned change
  begin/complete, and changed-path reporting.
- Watcher lifetime is tied to application lifecycle and library root changes,
  not individual request handling.
- Root selection is per-library and removes redundant child roots when a parent
  is already watched.
- Watcher failures are handled coarsely: permission failures are distinct, while
  other watcher errors remove the watcher in the approved reference path.

### Event Handling And Coalescing

- Created, deleted, renamed, and changed notifications all enter a common
  changed-path pipeline. The approved watcher layer does not preserve detailed
  event-kind semantics as authoritative media lifecycle facts.
- Events are filtered before refresh scheduling, including static ignores,
  ignore-file rules, and planned-write suppression.
- Coalescing merges equal paths, child paths, parent paths, and sibling bursts
  into a broader delayed refresh scope.

### Debounce

- The approved Jellyfin configuration exposes a global library monitor delay;
  the local reference default is 60 seconds.
- The delayed refresher restarts its one-shot timer as more affected paths are
  added or the refresh scope is widened.
- The delay is debounce, not stable-file proof. The approved reference does not
  show repeated unchanged evidence, closed-file proof, or checksum proof before
  refresh.

### Planned Host Writes

- Jellyfin models planned writes as monitor-level suppression boundaries.
- A suppressed path and related parent/child paths are ignored during the
  suppression window.
- Completion removes suppression after a wait and may optionally report the
  path as changed through the normal pipeline.

### Fallback / Reconciliation

- The approved Jellyfin watcher files do not show a full-library scan being
  enqueued directly after watcher error, overflow, missed events, or restart.
- `FileRefresher` does show local upward reconciliation when a changed/deleted
  path no longer maps directly to an item.
- Nako should treat watcher errors and unreliable events as explicit degraded or
  reconciliation-pending states, then route fallback through Nako's supervised
  scan/control-plane path.

## Nako Decision Implications

- Treat watcher notifications as hints that select reconciliation scope, not as
  authoritative source lifecycle facts.
- Keep Nako's stable-candidate evidence contract. Debounce delay alone is not
  enough for slow copies, NAS paths, WebDAV, stale cache, or remote-like local
  mounts.
- Design coalescing in Source Locator / `StorageUri` terms: same source, child
  source, parent scope, sibling burst, library root, and unknown/degraded scope.
- Add explicit host-owned write suppression for NFO, artwork, sidecar, import,
  and VFS write workflows if 06a needs to prevent self-triggered watcher loops.
  Suppression should include scope, owner, reason, TTL, completion semantics,
  optional reconciliation intent, and redaction-safe diagnostics.
- Model watcher capability separately from operator enablement. A library may
  request realtime monitoring while a backend is unsupported, degraded,
  permission-blocked, or only weakly reliable.
- Fallback policy should be Nako-native:
  - source-scoped when the changed locator is reliable;
  - library-scoped when watcher topology or backend capability is unreliable;
  - repair-scoped when tombstones, moves, or source identity ambiguity are
    suspected.
- Admin/operator diagnostics should expose safe facts such as library id,
  backend kind, capability class, degraded reason, pending reconciliation count,
  next due time, and last safe failure class. They must not expose raw local
  paths, credentials, raw source locators, or provider errors.

## What Nako Should Decide Independently

- Public/operator shape for realtime-monitor enablement.
- Debounce delay defaults and whether delay is event-window based,
  observation-window based, or both.
- Stable observation count/window per backend capability.
- Suppression TTL and whether completion should always enqueue
  reconciliation.
- Which watcher errors require a source-scoped re-observe versus a library scan.
- How watcher capability appears in storage/VFS diagnostics and Admin surfaces.

## Licensing / Do-Not-Copy Note

Jellyfin reference material under `repo-ref/` is GPL-family reference material.
This evidence is behavior-level only. Do not copy, translate, port, or derive
Nako source code, comments, tests, schemas, generated artifacts, naming, or
control flow from these files. Future Nako implementation must be original and
written against Nako's own domain model, VFS/storage boundaries, stable-candidate
evidence, durable scan/tombstone behavior, and ADR 0053 control-plane boundary.

## Verification

- Research artifacts are limited to the task directory.
- No Nako implementation code or architecture docs were changed.
- Manual licensing review was performed against `docs/legal/LICENSING.md`.
- `git diff --check` should be run before closeout.
