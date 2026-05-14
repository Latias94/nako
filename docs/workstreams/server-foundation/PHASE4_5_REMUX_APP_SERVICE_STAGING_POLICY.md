# Phase 4.5: Remux App Service Integration and Local Staging Policy

## Goal

Move remux orchestration behind an application service and define the local
staging policy for remux outputs. This phase prepares the server for a future
HTTP remux playback route without letting handlers own FFmpeg process details,
temporary paths, cancellation, timeout, or cleanup behavior.

## Implemented Shape

### Application Service Boundary

`taru-server::app` owns a remux-facing application service that composes:

- source lookup;
- playback/remux decision context;
- `taru-transcode` command planning and runner execution;
- staging path allocation;
- duplicate request behavior;
- application-safe error mapping.

HTTP handlers can later call this service instead of directly touching
FFmpeg plans, process runners, or temporary files.

### Local Staging Policy

The staging policy now defines:

- configured staging root;
- deterministic output directory keyed by source ID and remux container;
- final output extension based on planned remux container;
- temporary output naming remains owned by `taru-transcode`;
- output parent creation before FFmpeg starts;
- path normalization that prevents escaping the staging root.

Remote-source staging remains out of scope. This phase only makes the local
staging contract explicit enough that remote storage can later plug into it.

### Duplicate Requests

Duplicate remux requests should not spawn unbounded equivalent FFmpeg work.
This phase chooses and tests this behavior:

- reuse an existing matching staged output when it is complete;
- reject duplicates with a stable conflict/pending error.

The first implementation can be conservative, but it must be deterministic and
documented.

### Error Mapping

The app boundary should map lower-level errors into stable categories:

- invalid source or unsupported source;
- invalid remux request;
- staging path failure;
- runner failure;
- cancellation;
- timeout;
- resource limit exceeded.

Process stderr and command details should remain diagnostic data, not public
HTTP response shape.

## Config

`taru-server` now accepts:

```toml
remux_staging_root = "taru-cache/remux"
```

Relative paths are resolved by the server process. Production deployments
should choose a staging root outside watched media roots so staged outputs are
not indexed as library items.

## Non-Goals

- No public remux playback HTTP route yet.
- No HLS playlist or segment serving.
- No hardware acceleration detection.
- No remote-source staging/cache behavior.
- No persisted transcode session table unless the service boundary cannot be
  kept clean without it.
- No long-term cache eviction policy beyond safe cleanup of session outputs.

## Validation

Expected coverage:

- staging paths stay under the configured staging root;
- output names are deterministic for equivalent requests;
- completed outputs are reused without spawning FFmpeg again;
- in-flight duplicate requests return a stable conflict error;
- the app service can run the FFmpeg runner through a fake process;
- no public HTTP remux route is exposed.

Validation used for this phase:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
