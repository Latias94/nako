# Playback Subtitle Language Default Policy - TODO

Status: Completed
Last updated: 2026-05-30

## Task Ledger

### PSLD-010 - Open lane and freeze first-slice policy

Status: Completed
Owner: codex
Depends on: none

Scope:

- `docs/workstreams/playback-subtitle-language-default-policy`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

Goal:

- Create the durable workstream.
- Freeze the first slice to request-scoped subtitle language/default selection.
- Keep persisted user settings, UI controls, OCR/burn-in/ASS shaping, addon
  late-subtitle readiness, and LL-HLS/DASH/DRM out of this lane.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-subtitle-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Evidence:

- `docs/workstreams/playback-subtitle-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-subtitle-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: This lane is open and linked from playback architecture indexes.
- The first executable task is PSLD-020.

### PSLD-020 - Model request-scoped subtitle language preference

Status: Completed
Owner: codex
Depends on: PSLD-010

Scope:

- `crates/nako-playback`
- server playback selection adapters if needed
- focused playback policy tests

Goal:

- Extend playback preference vocabulary with ordered preferred subtitle
  languages.
- Implement deterministic subtitle selection precedence: explicit stream,
  preferred language match, fallback.
- Keep the selection result explainable and independent of HLS playlist
  authoring.

Validation:

```text
cargo nextest run -p nako-playback subtitle --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-playback`
- `crates/nako-server/src/app/playback`
- `docs/workstreams/playback-subtitle-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Playback owns explicit-stream, preferred-language, and fallback
  subtitle selection.
- DONE: Request identity normalizes preferred subtitle language values.
- DONE: HLS/server adapters compile with the new preference field while keeping
  HTTP query input empty until PSLD-030.
- PSLD-030 should make the behavior wire-visible through HLS request parsing
  and playlist/default-rendition assertions.

### PSLD-030 - Surface policy through HLS subtitle rendition defaults

Status: Completed
Owner: codex
Depends on: PSLD-020

Scope:

- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs` if request query parsing is needed
- `crates/nako-api` only if public DTO/query contracts change

Goal:

- Use the selected policy subtitle stream as the HLS subtitle rendition default.
- Preserve explicit `requested_subtitle_stream` precedence.
- Add or update browser/public playback request parsing if the first slice
  needs a wire-level preferred-language input.
- Preserve request identity and HLS route behavior.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- HLS subtitle default tests.
- Public/browser request parsing tests if contracts change.
- `docs/workstreams/playback-subtitle-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: The public HLS playlist route accepts `preferred_subtitle_language` as
  a comma-separated ordered language list.
- DONE: HLS subtitle rendition authoring marks the selected policy subtitle
  stream as the only generated `DEFAULT=YES` subtitle rendition.
- DONE: Explicit `subtitle_stream` still overrides preferred subtitle language
  selection.
- DONE: Normalized language preference input reuses the same HLS request and
  transcode session identity.
- Split persisted user settings, UI controls, OCR/burn-in/ASS shaping, addon
  readiness, LL-HLS, DASH, DRM, and offline sync instead of adding them here.

### PSLD-040 - Verify, document, and close or split follow-ons

Status: Completed
Owner: planner
Depends on: PSLD-030

Scope:

- `docs/workstreams/playback-subtitle-language-default-policy`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Goal:

- Run fresh focused gates.
- Record final evidence and residual risks.
- Close the lane or split follow-ons for persisted preferences, UI controls,
  OCR/burn-in/ASS shaping, addon readiness, LL-HLS, DASH, DRM, or offline sync.

Validation:

```text
cargo nextest run -p nako-playback subtitle --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Review:

- Use `review-workstream` and `verify-rust-workstream` before closeout.

Evidence:

- `docs/workstreams/playback-subtitle-language-default-policy/EVIDENCE_AND_GATES.md`
- `docs/workstreams/playback-subtitle-language-default-policy/HANDOFF.md`
- `docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json`

Handoff:

- DONE: Fresh focused gates passed and evidence is recorded.
- DONE: Architecture and workstream docs mark the shipped request-scoped
  subtitle language/default policy slice as complete.
- DONE: Persisted settings, UI controls, OCR/burn-in/ASS shaping, addon
  readiness, LL-HLS, DASH, DRM, and offline sync remain explicit follow-ons.
