# HTTP Library Scan Trace Context First Slice

## Goal

Propagate the existing safe HTTP request trace context into public and Admin
library scan enqueue routes so queued `disk.scan` jobs and eventual
`LibraryScanned` events can correlate to `x-request-id`.

## Requirements

- Public `POST /libraries/{library_id}/scan` must convert the typed
  `HttpTraceContext` into `LibraryScanTraceContext` and enqueue the scan with
  trace context.
- Admin `POST /admin/v1/libraries/{library_id}/scan` must do the same.
- Persisted job input may contain only the normalized safe `request_id` already
  accepted by the root HTTP trace middleware.
- Existing untraced app-service calls must keep their current behavior.
- Do not add new Admin/Public API response fields, generated TypeScript
  contracts, schema migrations, logging configuration, or diagnostics routes.
- Do not propagate raw headers, paths, local roots, tokens, URLs, or request
  bodies into job input, summaries, events, or logs.
- Add focused tests for public and Admin scan routes proving:
  - response still returns `202 Accepted`;
  - job input is still hidden from job DTO responses;
  - stored job input includes normalized `trace_context.request_id`;
  - raw inbound casing and local paths are not leaked.
- Update relevant server spec and control-plane architecture notes.

## Acceptance Criteria

- [ ] Public library scan route enqueues with `LibraryScanTraceContext`.
- [ ] Admin library scan route enqueues with `LibraryScanTraceContext`.
- [ ] Persisted scan job input contains only normalized safe `request_id`.
- [ ] Public job response still redacts raw input JSON.
- [ ] Existing app-level untraced enqueue/scan paths remain available.
- [ ] Focused server tests, `cargo fmt --all -- --check`, `cargo check`, `git
      diff --check`, and Trellis validation pass.

## Definition Of Done

- Rust code and tests are committed.
- Specs and control-plane architecture docs reflect the HTTP scan trace context
  boundary.
- Task evidence records verification commands.
- Task is archived.

## Technical Notes

- Existing HTTP trace middleware:
  `crates/nako-server/src/http/trace_context.rs`.
- Existing app trace boundary:
  `crates/nako-server/src/app/jobs.rs`
  `enqueue_library_scan_with_trace_context`.
- Existing app test:
  `background_scan_job_propagates_trace_context_to_library_scanned_event`.
- Likely write scope:
  - `crates/nako-server/src/http/library.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/tests/library.rs`
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `docs/architecture/CONTROL_PLANE.md`
