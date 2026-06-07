# Jellyfin Comparison: Configuration and Task Management vs Nako Playback Runtime Settings

## Reference Studied

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ConfigurationController.cs`
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`

## Findings

Jellyfin exposes server configuration through explicit Admin HTTP routes. The
configuration controller supports reading the server configuration, replacing
the full configuration, reading named configuration blocks, and updating named
configuration blocks. It also exposes scheduled-task operations separately:
listing tasks, starting/stopping a task, and updating triggers.

Nako should not copy Jellyfin's broad configuration replacement model for
playback runtime policy. Nako already has a narrower route that accepts a typed
`AdminPlaybackRuntimeSettingsPayload`, validates domain invariants, and reports
whether a restart is required.

## Nako Decision

Keep the existing narrow playback runtime settings contract. The architecture
lesson is that operational settings routes should be explicit management-plane
contracts, not hidden server-only endpoints. Since Nako already has the typed
backend contract and the Settings page already owns related diagnostics, the
route should be generated and consumed through Admin Web's typed client/data
source.

## Redaction Boundary

The UI may render numeric worker counts, retention windows, cleanup booleans,
policy enums, setting source, effect, and update timestamps. It must not render
raw storage paths, backend URLs, FFmpeg command lines, hardware device nodes,
tokens, proxy values, local filesystem paths, or process internals.
