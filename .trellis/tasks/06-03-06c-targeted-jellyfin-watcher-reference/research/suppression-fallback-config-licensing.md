# Research: suppression, fallback, config, and licensing

- Query: Behavior-level Jellyfin watcher reference for planned-write
  suppression, unreliable watcher fallback, operator configuration, and
  licensing constraints.
- Scope: strictly limited to the PRD-approved Jellyfin watcher files plus
  Nako architecture/spec/licensing context.
- Date: 2026-06-03

## Approved Jellyfin Source Paths

- `repo-ref/jellyfin/MediaBrowser.Controller/Library/ILibraryMonitor.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/LibraryMonitor.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/IO/FileRefresher.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/ServerConfiguration.cs`

No other Jellyfin source files are used in this artifact.

## Planned Host-Write Suppression

- `ILibraryMonitor.cs` exposes behavior-level operations for monitor start,
  stop, reporting that a filesystem change began, reporting completion, and
  reporting a changed path.
- `LibraryMonitor.cs` keeps a temporary ignored-path set for host-owned writes.
  Events for the suppressed path and related parent/child paths are ignored
  while the suppression entry is active.
- Completion waits before removing the suppressed path and can optionally report
  the path as changed through the normal changed-path pipeline.

## Fallback And Watcher Degradation

- `LibraryMonitor.cs` has a watcher error path. Permission errors are treated as
  a distinct failure class, while other watcher errors dispose and remove that
  watcher.
- The approved Jellyfin files do not show a direct full-library scan being
  enqueued after watcher error, overflow, missed events, or process restart.
- `FileRefresher.cs` does show local reconciliation after a changed path no
  longer maps directly: it searches upward toward a known library item or owner
  before refreshing.

## Operator Configuration And Capability Signals

- `ServerConfiguration.cs` exposes a global library monitor delay in seconds;
  the local reference default is 60 seconds.
- `LibraryMonitor.cs` consumes per-library realtime monitor enablement when
  deciding which library roots to watch.
- Watcher support is implicit in the approved files: missing paths are skipped,
  watcher creation can fail, recursive watching is attempted, and errors can
  remove a watcher. The files do not expose a separate capability model.

## Nako Implications

- Nako should represent planned writes as first-class, bounded suppression
  scopes with owner, reason, expiry, and redaction-safe diagnostics.
- Suppression should use Nako `StorageUri` / Source Locator semantics, not raw
  host path string ownership.
- Watcher errors should become explicit product states such as degraded,
  unsupported, permission-required, or reconciliation-pending rather than
  silently removing coverage.
- Fallback reconciliation should be designed in Nako-native terms: source-scoped
  when the locator is reliable, library-scoped when watcher topology is
  unreliable, and repair-scoped when tombstones or moves are suspected.
- Operator configuration should distinguish per-library realtime enablement,
  per-backend watcher capability, debounce delay, stable observation
  requirement, suppression TTL, and fallback scan policy.

## Licensing / Do-Not-Copy Note

- `docs/legal/LICENSING.md` allows behavior and architecture study from
  reference repositories but forbids copying source files, functions, comments,
  migrations, tests, schemas, generated code, and line-by-line translations.
- This artifact is behavior-level only. Future Nako implementation must be
  original and expressed through Nako's own domain model, VFS capability model,
  source/tombstone contracts, control-plane runtime, and tests.

## Caveats / Not Found

- The approved Jellyfin files do not show watcher error/overflow directly
  enqueuing a full scan.
- The approved files consume per-library realtime monitor enablement but do not
  define its UI/API shape.
- The approved files are local-filesystem watcher references; Nako must design
  WebDAV, NAS, stale-cache, and unsupported-backend behavior independently.
