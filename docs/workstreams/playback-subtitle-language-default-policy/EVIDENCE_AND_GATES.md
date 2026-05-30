# Playback Subtitle Language Default Policy - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Smallest Current Repro

The current proof is PSLD-020 request-scoped subtitle language preference
modeling.

```bash
cargo nextest run -p nako-playback subtitle --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
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

## Evidence Anchors

- `docs/workstreams/playback-subtitle-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-subtitle-language-default-policy/TODO.md`
- `docs/workstreams/playback-subtitle-language-default-policy/MILESTONES.md`
- `crates/nako-playback`
- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-api`

Fresh verification is required before marking a task, Codex goal, or lane
complete.
