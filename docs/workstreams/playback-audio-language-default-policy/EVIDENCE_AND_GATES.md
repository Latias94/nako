# Playback Audio Language Default Policy - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Smallest Current Repro

The current proof is PALD-040 closeout after PALD-030 HLS request/default
rendition integration.

```bash
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-audio-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Proves workstream docs are syntactically valid and whitespace-clean.

### Playback Policy Gate

```bash
cargo nextest run -p nako-playback audio --no-fail-fast
```

Proves audio preference vocabulary, explicit-stream precedence, language match,
and fallback behavior in playback policy code.

### Server Playback Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

Proves server playback integration, selected audio propagation, route behavior,
and broader playback regression coverage.

### HLS Integration Gate

```bash
cargo nextest run -p nako-server hls --no-fail-fast
```

Proves HLS audio rendition default flags and playlist/session behavior.

### API Contract Gate

```bash
cargo nextest run -p nako-api --no-fail-fast
```

Required only if this lane changes public DTOs, generated contracts, or request
query contract surfaces.

### Final Closeout Gate

```bash
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json
git diff --check
```

Use broader workspace gates only if public API contracts or shared playback
types change outside the focused surfaces above.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to a review
note.

## Evidence Log

| Date | Task | Evidence | Status | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-29 | PALD-010 | Workstream opened | Passed | Fresh gates: `python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-audio-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`. |
| 2026-05-29 | PALD-020 | Request-scoped audio language preference modeled | Passed | Fresh gates: `cargo nextest run -p nako-playback audio --no-fail-fast` (4 passed, 19 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (133 passed, 343 skipped); extra HLS coverage `cargo nextest run -p nako-server hls --no-fail-fast` (54 passed, 422 skipped); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`; `git diff --check`. Review: no blocking findings. API gate not run because PALD-020 did not change public DTOs or request query contracts. |
| 2026-05-29 | PALD-030 | HLS audio rendition default policy surfaced through public request path | Passed | Fresh gates: `cargo nextest run -p nako-server preferred_audio_language --no-fail-fast` (3 passed, 475 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (56 passed, 422 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (135 passed, 343 skipped); `cargo nextest run -p nako-api --no-fail-fast` (69 passed, 0 skipped). The public HLS route accepts `preferred_audio_language`; explicit `audio_stream` overrides it; normalized ordered language input reuses the same HLS transcode session; OpenAPI and generated TypeScript/Kotlin SDKs expose the query. Review: no blocking findings; corrected the stale ADR 0023 path while auditing workstream authority links. Final hygiene: `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`; `git diff --check`. |

## Evidence Anchors

- `docs/workstreams/playback-audio-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-audio-language-default-policy/TODO.md`
- `docs/workstreams/playback-audio-language-default-policy/MILESTONES.md`
- `crates/nako-playback`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-api`
- `sdk/typescript/src/index.ts`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
- `docs/api/HTTP_API.md`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
