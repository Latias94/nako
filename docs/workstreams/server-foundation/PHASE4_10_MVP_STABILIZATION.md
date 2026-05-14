# Phase 4.10: MVP Stabilization

## Goal

Stabilize the local video-library playback MVP before expanding into extension,
automation, or remote-storage work.

## Proposed Shape

- Audit HTTP API documentation against implemented routes and response shapes.
- Audit error mapping for playback, session, scan, metadata, and NFO routes.
- Document runtime configuration for scan, metadata, remux, HLS, hardware
  policy, and resource budgets.
- Add focused tests for uncovered edge cases discovered during the audit.
- Document known MVP limitations and intentional non-goals.
- Check performance-sensitive paths for bounded concurrency and avoid
  unbounded file or artwork loading.

## Non-Goals

- No new provider implementation.
- No Flutter/client work.
- No remote storage backend implementation.
- No addon runtime implementation.

## Validation

Expected coverage:

- docs match implemented API routes and config fields;
- validation gates pass for the workspace;
- route errors are stable and safe for clients;
- MVP limitations are explicitly documented;
- no known unbounded expensive playback or scan path remains undocumented.
