# PTOH-040 — Session Failure Taxonomy

Date: 2026-05-22
Task: PTOH-040
Status: completed

## Summary

Expanded playback transcode failure classification from a narrow coarse set to
a support-oriented taxonomy that separates probe, plan, staging, budget,
hardware fallback, runner, timeout, storage, stale, cancellation, invalid
request, and unknown boundaries.

## Code Changes

- Added internal `TranscodeFailureCategory` variants for `probe`, `plan`,
  `staging`, `budget`, and `hardware_fallback`.
- Added category mapping in `TranscodeFailureCategory::from_error`.
- Persisted playback app failures now store redacted operator summaries instead
  of raw error strings.
- Public Client playback session DTO conversion maps new internal categories
  back to existing coarse public categories and derives safe public failure
  messages from categories.
- Added DB round-trip/contract coverage for a new persisted category.
- Added app and HTTP tests proving timeout classification, raw stderr/path
  redaction, Admin list category visibility, and Public Client compatibility.

## Verification

- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-server http::tests::system --no-fail-fast`
- `cargo nextest run -p nako-core transcode_failure_category_maps_support_boundaries --no-fail-fast`
- `cargo nextest run -p nako-api transcode_session_response_ --no-fail-fast`
- `cargo nextest run -p nako-db nako_database_sqlite_lists_transcode_sessions_with_filters_and_pagination --no-fail-fast`
- `cargo nextest run -p nako-db sqlite_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --name-only -- crates/nako-client-protocol`

## Follow-up

Continue with PTOH-050: build a bounded Admin-only playback support evidence
read model from the now-stable readiness, session, staging, and hardware facts.
