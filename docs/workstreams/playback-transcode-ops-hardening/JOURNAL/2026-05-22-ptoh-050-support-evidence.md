# PTOH-050 — Admin Playback Support Evidence

Date: 2026-05-22

## Summary

Implemented a bounded Admin-only playback support evidence read model at
`GET /admin/v1/playback/support`.

The route accepts optional `session_id` and `source_id` query context. It
returns runtime readiness evidence plus safe session/source facts when present:

- session state, kind, failure category, active/terminal flags, timestamps, and
  a SHA-256 request key fingerprint instead of the raw request key;
- source id/library/item ids, source scheme, file name, size, and fingerprint
  presence without raw source references or fingerprint values;
- readiness, FFmpeg probe summary, transcode/remux/remote/staging budgets, and
  narrowed hardware support evidence;
- explicit redaction evidence flags.

The route rejects mismatched `session_id` + `source_id` contexts rather than
silently returning evidence for a different source.

## Boundary Decisions

- Kept the route Admin-only and read-only.
- Did not persist or export support evidence bundles.
- Did not change Public Client API or `taru-client-protocol`.
- Updated Admin TypeScript contract and Admin web typed client/mocks because
  this is an Admin API surface.
- Avoided raw Source Locator terminology in the external Admin contract. The
  DTO uses `source_scheme` and `source_references_redacted` instead.
- Kept full UI workflows, downloadable bundles, retention policy, adaptive HLS,
  optimized versions, downloader/watch-folder, network, AI, and addon runtime
  outside this lane.

## Verification

- `cargo nextest run -p taru-api admin_playback --no-fail-fast` — pass, 4
  tests.
- `cargo nextest run -p taru-api admin_contract --no-fail-fast` — pass, 5
  tests.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast` —
  pass, 17 tests.
- `cargo check -p taru-api --tests` — pass.
- `cargo check -p taru-server --tests` — pass.
- `cargo fmt --all -- --check` — pass.
- `npm run check` from `apps/admin-web` — pass.
- `npm test` from `apps/admin-web` — pass, 9 tests.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/taru-client-protocol` — no output.

## Review Notes

Workstream compliance: satisfied PTOH-050 goal, dependencies, Admin ownership,
redaction checklist, and public client stability.

Code quality: support evidence composition reuses the existing runtime
diagnostics path, keeps source/session context lookup in the playback app
boundary, keeps HTTP mapping in the Admin HTTP boundary, and keeps DTO redaction
tests in `taru-api`.

Residual risks are follow-ons, not blockers:

- no downloadable support bundle or retention policy;
- no Admin UI workflow for inspecting support evidence beyond typed client
  support;
- no support evidence aggregation across multiple sessions.

Continue with PTOH-060 closeout and parent umbrella re-score.
