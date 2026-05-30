# Playback Subtitle Language Default Policy - Evidence And Gates

Status: Completed
Last updated: 2026-05-30

## Smallest Current Repro

The final proof is the closed request-scoped subtitle language/default policy
slice, including playback selection, HLS route behavior, and public API/SDK
contract exposure.

```bash
cargo nextest run -p nako-server preferred_subtitle_language --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
```

## Gate Set

### Planning Gate

```bash
python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-subtitle-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Proves workstream docs are syntactically valid and whitespace-clean.

### Playback Policy Gate

```bash
cargo nextest run -p nako-playback subtitle --no-fail-fast
```

Proves subtitle preference vocabulary, explicit-stream precedence, language
match, and fallback behavior in playback policy code.

### Server Playback Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

Proves server playback integration, selected subtitle propagation, route
behavior, and broader playback regression coverage.

### HLS Integration Gate

```bash
cargo nextest run -p nako-server hls --no-fail-fast
```

Proves HLS subtitle rendition default flags and playlist/session behavior.

### API Contract Gate

```bash
cargo nextest run -p nako-api --no-fail-fast
```

Required only if this lane changes public DTOs, generated contracts, or request
query contract surfaces.

### Final Closeout Gate

```bash
cargo nextest run -p nako-playback subtitle --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json
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
| 2026-05-30 | PSLD-010 | Workstream opened | Passed | Fresh gates: `python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json`; `git diff --check -- docs/workstreams/playback-subtitle-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`. |
| 2026-05-30 | PSLD-020 | Request-scoped subtitle language preference modeled | Passed | Fresh gates: `cargo nextest run -p nako-playback subtitle --no-fail-fast` (4 passed, 23 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (135 passed, 350 skipped); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json`; `git diff --check`. The policy selects preferred subtitle languages after explicit stream selection, falls back to existing first-subtitle behavior when no language matches, and normalizes preferred subtitle language values in request identity. Review gate found no blocking findings; HLS wire/default-rendition behavior remains PSLD-030 scope. API gate not run because PSLD-020 did not change public DTOs or request query contracts. |
| 2026-05-30 | PSLD-030 | HLS subtitle default policy surfaced through public request path | Passed | Fresh gates: RED `cargo nextest run -p nako-server hls_playlist_route_accepts_preferred_subtitle_language_defaults --no-fail-fast` failed because the public HLS route did not expose subtitle renditions for `preferred_subtitle_language`; GREEN `cargo nextest run -p nako-server preferred_subtitle_language --no-fail-fast` (2 passed, 485 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (58 passed, 429 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (137 passed, 350 skipped); focused API contract gate `cargo nextest run -p nako-api -E 'test(public_openapi_image_contract_uses_public_refs_without_raw_locators) | test(typescript_sdk_includes_auth_version_error_and_pagination_runtime) | test(kotlin_sdk_includes_public_constants_paths_and_wire_types)' --no-fail-fast` (3 passed, 67 skipped); `cargo nextest run -p nako-api --no-fail-fast` first failed on stale generated Kotlin SDK output, then passed after regenerating TypeScript/Kotlin SDK package entries (70 passed, 0 skipped). The public HLS route accepts `preferred_subtitle_language`; explicit `subtitle_stream` overrides it; normalized ordered language input reuses the same HLS transcode session; OpenAPI, generated TypeScript/Kotlin SDKs, and HTTP API docs expose the query. |
| 2026-05-30 | PSLD-040 | Closeout verification and follow-on split | Passed | Fresh gates: `cargo nextest run -p nako-playback subtitle --no-fail-fast` (4 passed, 23 skipped); `cargo nextest run -p nako-server hls --no-fail-fast` (58 passed, 429 skipped); `cargo nextest run -p nako-server playback --no-fail-fast` (137 passed, 350 skipped); `cargo nextest run -p nako-api --no-fail-fast` (70 passed, 0 skipped); `cargo fmt --all -- --check`; `python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json`; `git diff --check`. Review: no blocking findings. Architecture and workstream docs now mark this first slice as shipped; persisted settings, UI controls, OCR/burn-in/ASS shaping, addon readiness, LL-HLS, DASH, DRM, and offline sync remain follow-ons. |

## Evidence Anchors

- `docs/workstreams/playback-subtitle-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-subtitle-language-default-policy/TODO.md`
- `docs/workstreams/playback-subtitle-language-default-policy/MILESTONES.md`
- `crates/nako-playback`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-api`
- `docs/api/HTTP_API.md`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
