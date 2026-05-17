# Playback Source Selection Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- M42 resolved the shallow `CatalogHydrationPort` lookup Interface at the
  public seam.
- `crates/taru-streaming/src/selection.rs` currently decides playback from
  container and codec compatibility only.
- `crates/taru-server/src/app/playback` still owns execution orchestration and
  must remain the place that runs remux/HLS/transcode work.
- `CONTEXT.md` defines **Playback Source Selection**, **Playback Runtime**,
  **Client Application**, **Source Variant**, **Playback Transcode**,
  **Optimized Version**, **Transcode Profile**, **Library Access**, and
  **Remote Access Endpoint** as separate domain concepts.

## Focused Gates

Run these as the implementation reaches each slice:

```powershell
cargo fmt --all -- --check
cargo check -p taru-streaming --tests
cargo nextest run -p taru-streaming --no-fail-fast
cargo check -p taru-server --tests
cargo nextest run -p taru-server http::tests::playback --no-fail-fast
cargo check -p taru-api --tests
```

If Public Client API DTOs or generated SDK output changes:

```powershell
npm run generate --prefix sdk/typescript
npm run check --prefix sdk/typescript
```

## Closeout Gates

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M43.
- 2026-05-17: `PlaybackSelectionRequest`, `PlaybackSelectionContext`,
  `PlaybackSelectedSource`, and `PlaybackExecutionPlan` added to
  `taru-streaming`.
- 2026-05-17: Server playback app now calls `select_playback_source` and
  executes remux/HLS decisions from the returned execution plan.
- 2026-05-17: Public Client API DTO mapping keeps the existing playback
  response shape; internal selection fields do not enter
  `taru-client-protocol`.
- 2026-05-17: Focused gates passed:
  `cargo fmt --all -- --check`,
  `cargo check -p taru-streaming --tests`,
  `cargo nextest run -p taru-streaming --no-fail-fast` with 8 tests passed,
  `cargo check -p taru-server --tests`,
  `cargo nextest run -p taru-server http::tests::playback --no-fail-fast`
  with 16 tests passed,
  `cargo check -p taru-api --tests`, and
  `cargo nextest run -p taru-api --no-fail-fast` with 12 tests passed.
- 2026-05-17: Closeout gates passed:
  `cargo fmt --all -- --check`,
  `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast` with 292 tests passed, and
  `git diff --check`.
