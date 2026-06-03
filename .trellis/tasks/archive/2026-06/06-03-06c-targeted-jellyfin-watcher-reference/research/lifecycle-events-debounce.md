# Research: Jellyfin watcher lifecycle, events, coalescing, and debounce

- Query: Behavior-level Jellyfin watcher reference for lifecycle, event kinds,
  event coalescing, debounce/delay, planned-write suppression, and fallback
  reconciliation, limited to the approved reference files.
- Scope: mixed
- Date: 2026-06-03

## Findings

### Files found

- `.trellis/tasks/06-03-06c-targeted-jellyfin-watcher-reference/prd.md`:
  task scope, non-goals, and GPL/no-copy constraints for this targeted pass.
- `docs/architecture/LIBRARY_PIPELINE.md`: Nako authority for the media intake
  chain; marks watcher/debounce as weak but backed by stable-candidate evidence.
- `docs/architecture/STORAGE_VFS.md`: Nako storage/VFS reliability boundary and
  source-locator/capability constraints.
- `docs/architecture/CONTROL_PLANE.md`: Nako background/runtime work must stay
  inside supervised control-plane boundaries.
- `docs/legal/LICENSING.md`: reference-code policy; GPL/Jellyfin material may be
  studied for behavior but not copied or translated into Nako.
- `.trellis/tasks/archive/2026-06/06-03-05d-library-watcher-debounce-intake-stability/evidence.md`:
  prior Nako stable-candidate foundation and deferred watcher/runtime scope.
- `.trellis/spec/nako-library/backend/index.md`: library scan/intake spec entry
  point.
- `.trellis/spec/nako-library/backend/directory-structure.md`: keeps stable
  candidate evidence in `intake.rs`, not in watcher runtimes or schedulers.
- `.trellis/spec/nako-library/backend/quality-guidelines.md`: requires repeated
  unchanged intake observations before watcher candidates become stable.
- `.trellis/spec/nako-library/backend/logging-guidelines.md`: requires
  redaction-safe library diagnostics.
- `.trellis/spec/nako-vfs/backend/index.md`: VFS adapter spec entry point.
- `.trellis/spec/nako-vfs/backend/quality-guidelines.md`: requires StorageUri
  authority, accurate backend capabilities, explicit stale/cache behavior, and
  redacted diagnostics.
- `.trellis/spec/nako-server/backend/index.md`: server/control-plane spec entry
  point for runtime-supervised work.
- `crates/nako-library/src/intake.rs`: Nako stable-candidate evidence helper.
- `crates/nako-server/src/app/watch_folder_runtime.rs`: current supervised
  watch-folder polling runtime and scan enqueue behavior.
- `crates/nako-server/src/app/acquisition_intake.rs`: current watch-folder
  candidate discovery/classification behavior.
- `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs`:
  Jellyfin monitor contract for start/stop, planned-change boundaries, and
  direct path-change reporting.
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs`:
  Jellyfin watcher lifecycle, event handlers, suppression, and coalescing
  entrypoint.
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs`:
  Jellyfin delayed refresh timer and affected-item reduction.
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/ServerConfiguration.cs`:
  Jellyfin operator-facing watcher delay default.

### Internal Nako patterns

- Nako already models stable intake with an observation key and a required
  count of two unchanged observations before `Stable` is returned
  (`crates/nako-library/src/intake.rs:3`,
  `crates/nako-library/src/intake.rs:31`,
  `crates/nako-library/src/intake.rs:45`).
- Nako watch-folder classification maps stable intake to `Ready`, resets to
  `Inspecting` while evidence is still changing, and records redaction-safe
  diagnostics instead of mutating library state directly
  (`crates/nako-server/src/app/acquisition_intake.rs:829`,
  `crates/nako-server/src/app/acquisition_intake.rs:842`,
  `crates/nako-server/src/app/acquisition_intake.rs:848`,
  `crates/nako-server/src/app/acquisition_intake.rs:814`).
- Nako's current watch-folder runtime is supervised through the server runtime,
  checks per-library `realtime_monitor`, limits itself to local watch roots, and
  enqueues a library scan only after newly-ready candidates appear
  (`crates/nako-server/src/app/watch_folder_runtime.rs:45`,
  `crates/nako-server/src/app/watch_folder_runtime.rs:55`,
  `crates/nako-server/src/app/watch_folder_runtime.rs:107`,
  `crates/nako-server/src/app/watch_folder_runtime.rs:124`).
- Nako discovery walks VFS backends rather than raw host paths, respects
  `max_depth`, records failures, redacts root URIs, and does not create managed
  import artifacts or apply promotions during discovery
  (`crates/nako-server/src/app/acquisition_intake.rs:321`,
  `crates/nako-server/src/app/acquisition_intake.rs:347`,
  `crates/nako-server/src/app/acquisition_intake.rs:355`,
  `crates/nako-server/src/app/acquisition_intake.rs:372`,
  `crates/nako-server/src/app/acquisition_intake.rs:442`).

### Jellyfin monitor contract

- The monitor contract exposes explicit start/stop lifecycle methods, planned
  filesystem-change begin/complete methods, and a direct changed-path reporting
  method (`repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs:11`,
  `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs:16`,
  `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs:22`,
  `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs:29`,
  `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs:35`).

### Lifecycle behavior

- `LibraryMonitor` registers `Start` on host application start and `Stop` on
  host application stopping, so watcher lifetime is tied to process lifetime
  rather than a request path
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:66`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:67`).
- `Start` subscribes to library add/remove events, collects root library
  physical locations whose library options enable realtime monitoring, removes
  redundant child paths when a parent is already watched, and starts one watcher
  per selected path
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:118`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:123`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:126`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:134`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:140`).
- Runtime library additions/removals update watcher coverage only for aggregate
  library roots: added roots start watching and removed roots stop watching
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:159`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:172`).
- `StartWatchingPath` skips missing directories, avoids duplicate watchers,
  creates the watcher asynchronously, watches subdirectories, uses a larger
  watcher buffer, registers created/deleted/renamed/changed/error handlers, and
  then enables events
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:218`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:226`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:232`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:236`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:238`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:239`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:248`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:256`).
- `Stop` unsubscribes library events, disposes all active watchers, clears the
  watcher map, and disposes pending refreshers; `Dispose` delegates to `Stop`
  once (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:446`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:451`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:456`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:457`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:485`).
- Watcher errors are differentiated only coarsely in the allowed code:
  permission errors are logged and return, while other watcher errors dispose
  and remove that watcher
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:317`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:322`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:328`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:330`).

### Event kinds and normalization

- Jellyfin registers the same changed-path handler for created, deleted,
  renamed, and changed events; error events go through a separate error handler
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:248`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:249`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:250`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:251`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:252`).
- The registered notification filters include creation time, directory name,
  file name, last write, size, and attributes, but the downstream handler does
  not preserve which filter or event kind fired
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:240`).
- `OnWatcherChanged` reports only the event full path to
  `ReportFileSystemChanged`; old rename paths and event-kind-specific semantics
  are not handled in this layer
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:338`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:342`).

### Filtering and planned-write suppression

- Direct path reporting first rejects global ignore patterns, then applies the
  `.ignore` rule handler before any coalescing is attempted
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:351`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:355`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:360`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:361`).
- Planned host writes are suppressed by adding a path to a temporary ignored
  set at begin time; direct changes to the ignored path, its descendants, or
  its immediate parent are ignored while the entry is present
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:71`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:75`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:367`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:369`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:377`).
- Planned-write completion waits 45 seconds before removing the temporary
  ignored path. If requested, it then reports the path as changed through the
  normal pipeline
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:79`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:86`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:88`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:90`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:94`).

### Coalescing behavior

- `CreateRefresher` keeps a list of active path refreshers under a lock. It
  merges an incoming path into an existing refresher when paths are equal, when
  the existing refresher path is a parent, when the incoming path is a parent,
  or when the two paths are siblings
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:388`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:392`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:397`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:404`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:411`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:418`).
- Equal paths restart the existing timer; child paths are appended to the
  parent refresher; parent and sibling cases rebase the refresher path upward
  so a broader library item can be refreshed
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:399`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:406`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:413`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:421`).
- A new `FileRefresher` is created only when no active refresher can absorb the
  path; completion removes and disposes the refresher
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:426`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:427`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:432`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:439`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs:460`).
- `FileRefresher` keeps a distinct list of affected paths and restarts its
  delay timer whenever paths are added or the base path is reset
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:21`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:41`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:51`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:60`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:88`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:103`).

### Debounce and delayed refresh

- The delay length is the server `LibraryMonitorDelay` setting. The allowed
  configuration file shows a default of 60 seconds
  (`repo-ref/jellyfin/MediaBrowser.Model/Configuration/ServerConfiguration.cs:172`).
- The refresher timer is one-shot: it is created or changed to fire after
  `LibraryMonitorDelay` seconds and does not repeat automatically
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:63`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:77`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:79`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:83`).
- When the timer fires, it snapshots affected paths, disposes the timer, signals
  completion, and then processes affected paths
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:106`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:110`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:117`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:118`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:122`).
- Processing maps affected paths to distinct library item IDs, skips aggregate
  folders, and calls each item as externally changed
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:130`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:132`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:136`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:138`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:140`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:149`).
- Deleted paths are reconciled to an existing affected item by walking from the
  changed path toward a known library item, then climbing to an existing owner
  or parent if the item path no longer exists
  (`repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:163`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:167`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:169`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:174`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:177`,
  `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs:179`).

### Nako implications

- Treat Jellyfin's watcher as a useful pressure test, not a design to port.
  Nako should keep the existing separation: watcher/runtime coordination in the
  control plane, stable evidence in `nako-library`, storage access through VFS,
  and actual library mutation through existing scan/import authorities.
- Jellyfin's event-kind normalization supports a Nako design where watcher
  notifications are hints. Nako should not rely on create/delete/rename events
  as authoritative media lifecycle facts; source identity and move/delete
  reconciliation should remain scan/fingerprint/tombstone work.
- Jellyfin's fixed delay is a debounce window, not stable-file proof. Nako
  should keep repeated unchanged observation evidence and decide separately
  whether to add a configurable debounce interval before or between
  observations.
- Jellyfin's path-tree coalescing maps well to a Nako `StorageUri`-tree
  coalescer, but Nako should implement equality and parent checks through
  normalized VFS identity, not raw OS path string rules.
- Jellyfin's planned-write suppression suggests Nako needs an explicit
  host-owned write-suppression ticket/window for NFO, artwork, import, and
  sidecar writes. That ticket should be bounded, redaction-safe, StorageUri
  scoped, observable, and independent of arbitrary async callbacks.
- Jellyfin does not show full scan fallback after watcher overflow/error in the
  allowed files. For Nako, watcher unreliability should enqueue or mark a
  reconciliation scan through the durable/supervised scan path rather than
  silently disabling coverage.
- Jellyfin refreshes a nearest known item after deletion. Nako should express
  this in its own terms: source tombstone reconciliation and parent/library
  rescan, not direct item refresh callbacks.
- Jellyfin's per-library realtime monitor flag and monitor-delay setting imply
  Nako should expose operator policy separately for enablement, delay/backoff,
  and fallback scan behavior, with safe defaults for local, NAS, and remote-like
  roots.

### External references

- No internet or third-party documentation lookup was used.
- Reference behavior is from local Jellyfin files under `repo-ref/jellyfin/`.
  The Jellyfin commit/version was not identified from the allowed files.

### Related specs

- `.trellis/spec/nako-library/backend/directory-structure.md`
- `.trellis/spec/nako-library/backend/quality-guidelines.md`
- `.trellis/spec/nako-library/backend/logging-guidelines.md`
- `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
- `.trellis/spec/nako-server/backend/index.md`

### Licensing / do-not-copy note

Jellyfin reference material under `repo-ref/` is GPL-family reference material.
This note is behavior-level only. Do not copy, translate, port, or derive Nako
source code, comments, tests, schemas, generated artifacts, names, or control
flow from these files. Nako implementation work must be original and expressed
through Nako's domain model, VFS/storage boundaries, stable-candidate evidence,
durable scan/tombstone behavior, and ADR 0053 control-plane boundary.

## Caveats / Not Found

- The allowed Jellyfin files do not show an explicit full-library reconciliation
  scan after watcher errors, buffer overflow, missed events, or process restart.
- The allowed Jellyfin files do not show a watcher restart/backoff loop after a
  non-permission watcher error; the watcher is disposed and removed in the
  visible code path.
- The allowed Jellyfin files do not provide stable-size, closed-file, checksum,
  or repeated-observation proof before refresh. They rely on a delay before
  item refresh.
- The allowed Jellyfin files do not define the `EnableRealtimeMonitor` option;
  they only consume it while deciding whether a library is watched.
- The allowed Jellyfin files do not show what `ChangedExternally()` ultimately
  scans, persists, or refreshes.
- The sibling coalescing branch uses direct parent-string comparison in the
  visible code path. Nako should not infer cross-platform path semantics from
  that detail.
- Open Nako questions for 06a:
  - What is the public/operator shape for realtime-monitor enablement and
    debounce delay?
  - Should debounce be event-window based, observation-interval based, or both?
  - What event conditions should enqueue a durable reconciliation scan rather
    than only re-observe a candidate?
  - How should host-owned write suppression be represented for NFO/artwork/import
    writes across local and VFS-backed storage?
  - Which diagnostics are safe and useful when watcher events are ignored,
    coalesced, delayed, dropped, or escalated to scan?
