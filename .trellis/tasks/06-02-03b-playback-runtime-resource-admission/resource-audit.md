# Playback Runtime Resource Admission Audit

## Current Behavior

- Direct Play local sources serve local files without playback runtime admission.
- Direct Play remote sources acquire the per-library remote stream permit from
  `LibraryStorageBackend` and hold it until the response body is dropped.
- Remux source starts acquire `remux_process` admission permits before process
  work starts; remote remux inputs also consume host-owned staging capacity via
  the staging backend.
- HLS source starts acquire `cpu_transcode` or `gpu_transcode` plus
  `hls_artifact_io` admission permits before process-backed work starts; remote
  HLS inputs also consume host-owned staging capacity via the staging backend.
- HLS supersede uses configured-capacity validation before cancellation and
  then waits briefly for released permits before starting replacement work.
- Admin runtime diagnostics expose configured and active playback resource
  pressure for remote stream/stage, remux, CPU/GPU transcode, and HLS artifact
  I/O without local paths or request keys.

## Selected Slice

The bounded slice is remote Direct Play stream admission. Before this change,
remote Direct Play waited indefinitely on the per-library stream semaphore when
all stream permits were held. That kept Direct Play first, but the pressure
behavior was hidden inside storage backend waiting and did not produce a stable
client-safe admission result.

The new behavior keeps Direct Play first and does not fall back to Remux or
HLS. When the remote stream budget is exhausted, the app service now returns a
typed `NakoError::Conflict` with the safe playback resource class
`remote_stream`.

## Follow-Ups

- HLS artifact reads still use manifest-backed local file serving; per-artifact
  read/write queueing should remain a separate lane because the current
  `hls_artifact_io` permit is held by the HLS runtime session.
- Remote worker scheduling, LL-HLS/CMAF, subtitle burn-in, and GPU vendor
  scheduling remain out of scope for this slice.
