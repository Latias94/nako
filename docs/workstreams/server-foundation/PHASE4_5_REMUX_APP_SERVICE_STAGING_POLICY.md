# Phase 4.5: Remux App Service Integration and Local Staging Policy

## Goal

Move remux orchestration behind an application service and define the local
staging policy for remux outputs. This phase prepares the server for a future
HTTP remux playback route without letting handlers own FFmpeg process details,
temporary paths, cancellation, timeout, or cleanup behavior.

## Proposed Shape

### Application Service Boundary

`taru-server::app` should own a remux-facing application service that composes:

- source lookup;
- playback/remux decision context;
- `taru-transcode` command planning and runner execution;
- staging path allocation;
- duplicate request behavior;
- application-safe error mapping.

HTTP handlers should eventually call this service instead of directly touching
FFmpeg plans, process runners, or temporary files.

### Local Staging Policy

The staging policy should define:

- configured staging root;
- per-session or deterministic output directory;
- final output extension based on planned remux container;
- temporary output naming;
- cleanup rules for failure, timeout, cancellation, and abandoned outputs;
- path normalization that prevents escaping the staging root.

Remote-source staging remains out of scope. This phase only makes the local
staging contract explicit enough that remote storage can later plug into it.

### Duplicate Requests

Duplicate remux requests should not spawn unbounded equivalent FFmpeg work.
This phase should choose and test one behavior:

- reuse an existing matching staged output when it is complete;
- attach to an existing in-flight session when the request key matches;
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
- duplicates follow the selected reuse/attach/reject behavior;
- runner failure, cancellation, timeout, and invalid request errors map to
  stable application errors;
- handlers remain thin once a public route is added later.

Required gates for the implementation phase:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```
