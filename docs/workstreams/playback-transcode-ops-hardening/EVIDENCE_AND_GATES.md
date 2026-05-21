# Playback Transcode Ops Hardening — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo fmt --all -- --check
cargo check -p taru-transcode --tests
cargo nextest run -p taru-transcode --no-fail-fast
cargo nextest run -p taru-streaming --no-fail-fast
cargo check -p taru-api --tests
cargo nextest run -p taru-api admin_playback --no-fail-fast
cargo nextest run -p taru-api admin_contract --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::system --no-fail-fast
git diff --check
git diff --name-only -- crates/taru-client-protocol
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
- `crates/taru-transcode/src/hardware.rs`
- `crates/taru-transcode/src/profile.rs`
- `crates/taru-transcode/src/ffmpeg.rs`
- `crates/taru-streaming/src`
- `crates/taru-server/src/app/playback`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/tests/system.rs`
- `crates/taru-api/src/admin.rs`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | PTOH-010 | `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Scope is runtime/readiness/diagnostics only; implementation starts at PTOH-020. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-020 | `cargo nextest run -p taru-transcode hardware --no-fail-fast`; `cargo nextest run -p taru-server admin_v1_playback_runtime --no-fail-fast`; `cargo nextest run -p taru-api admin_playback --no-fail-fast`; `cargo check -p taru-api --tests`; `cargo check -p taru-server --tests`; `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. `taru-transcode` hardware tests: 9 passed. Admin playback runtime HTTP tests: 2 passed. Admin playback DTO tests: 2 passed. Admin contract tests: 5 passed. API/server test checks passed. Formatting passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | PTOH-030 | `cargo nextest run -p taru-transcode --no-fail-fast`; `cargo nextest run -p taru-streaming --no-fail-fast`; `cargo nextest run -p taru-server playback --no-fail-fast`; `cargo nextest run -p taru-api admin_playback --no-fail-fast`; `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/playback-transcode-ops-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. `taru-transcode`: 35 passed. `taru-streaming`: 10 passed. Focused server playback/app/admin runtime coverage: 49 passed. Admin playback DTO tests: 2 passed. Admin contract tests: 5 passed. Formatting and JSON validation passed. Public client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. |

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
