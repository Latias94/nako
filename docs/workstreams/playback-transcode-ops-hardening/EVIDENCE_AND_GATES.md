# Playback Transcode Ops Hardening — Evidence And Gates

Status: Complete
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo fmt --all -- --check
cargo check -p nako-transcode --tests
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-streaming --no-fail-fast
cargo check -p nako-api --tests
cargo nextest run -p nako-api admin_playback --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo check -p nako-server --tests
cargo nextest run -p nako-server http::tests::system --no-fail-fast
git diff --check
git diff --name-only -- crates/nako-client-protocol
```

For planning-only changes, validate JSON and diff hygiene:

```powershell
python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/transcode-runtime/README.md`
- `docs/workstreams/playback-streaming/README.md`
- `docs/workstreams/admin-playback-runtime-diagnostics/DESIGN.md`
- `crates/nako-transcode/src/hardware.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-streaming/src`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`
- `crates/nako-api/src/admin.rs`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | PTOH-010 | `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Scope is runtime/readiness/diagnostics only; implementation starts at PTOH-020. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-020 | `cargo nextest run -p nako-transcode hardware --no-fail-fast`; `cargo nextest run -p nako-server admin_v1_playback_runtime --no-fail-fast`; `cargo nextest run -p nako-api admin_playback --no-fail-fast`; `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. `nako-transcode` hardware tests: 9 passed. Admin playback runtime HTTP tests: 2 passed. Admin playback DTO tests: 2 passed. Admin contract tests: 5 passed. API/server test checks passed. Formatting passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-030 | `cargo nextest run -p nako-transcode --no-fail-fast`; `cargo nextest run -p nako-streaming --no-fail-fast`; `cargo nextest run -p nako-server playback --no-fail-fast`; `cargo nextest run -p nako-api admin_playback --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. `nako-transcode`: 35 passed. `nako-streaming`: 10 passed. Focused server playback/app/admin runtime coverage: 49 passed. Admin playback DTO tests: 2 passed. Admin contract tests: 5 passed. Formatting and JSON validation passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-040 | `cargo nextest run -p nako-server playback --no-fail-fast`; `cargo nextest run -p nako-server http::tests::system --no-fail-fast`; `cargo nextest run -p nako-core transcode_failure_category_maps_support_boundaries --no-fail-fast`; `cargo nextest run -p nako-api transcode_session_response_ --no-fail-fast`; `cargo nextest run -p nako-db nako_database_sqlite_lists_transcode_sessions_with_filters_and_pagination --no-fail-fast`; `cargo nextest run -p nako-db sqlite_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Server playback scope: 52 passed. Server system scope: 15 passed. Core taxonomy test: 1 passed. Public client session redaction tests: 2 passed. SQLite transcode session list test: 1 passed. SQLite contract transcode lifecycle test: 1 passed. Formatting passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-050 | `cargo nextest run -p nako-api admin_playback --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server http::tests::system --no-fail-fast`; `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests`; `cargo fmt --all -- --check`; `npm run check` from `apps/admin-web`; `npm test` from `apps/admin-web`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Admin playback DTO tests: 4 passed. Admin contract tests: 5 passed. Server system/Admin HTTP tests: 17 passed. API/server test checks passed. Admin web TypeScript check passed. Admin web tests: 9 passed. Formatting passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. New evidence route is Admin-only, redacts paths/source references/FFmpeg command or stderr/output artifacts/credentials, rejects mismatched session/source context, and does not persist support bundles. |
| 2026-05-22 | PTOH-060 | `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Closeout docs and parent umbrella JSON are valid. Formatting passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. Runtime code gates are recorded in PTOH-020 through PTOH-050; PTOH-060 changed only workstream and umbrella routing docs. |

## Redaction Checklist

Every implementation task must prove that Admin support surfaces do not expose:

- raw local file paths;
- raw Source Locators;
- `ffmpeg_path` or `ffprobe_path`;
- FFmpeg command argv;
- transcode session output paths;
- raw stderr or logs;
- storage credentials;
- provider secrets;
- private environment variable values.

## Notes

Fresh verification is required before marking PTOH-020 or later tasks
complete. Do not use the planning gate as evidence that runtime behavior
shipped.
