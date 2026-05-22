# NFO Round Trip Preservation Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- ADR 0008 treats NFO as a local metadata boundary behind `nako-nfo`.
- ADR 0007 rejects unconditional local metadata replacement when local/user
  authority is involved.
- `MovieNfoCodec::render` currently generates a fresh `<movie>` document from
  Nako-known fields only.
- `NfoService::export_source` skips existing sidecars unless `force` is true;
  when forced, it renders and overwrites the whole XML file.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
```

## Closeout Gates

```powershell
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M47.
- 2026-05-17: Added `NfoPreservedRender`, `NfoPreservationReport`,
  `NfoFieldConflict`, and `NfoFieldConflictReason` to `nako-nfo`.
- 2026-05-17: Added `NfoCodec::render_preserving` and implemented
  preservation-aware movie NFO rendering in `MovieNfoCodec`.
- 2026-05-17: Forced export over an existing sidecar now reads the existing XML
  and writes preservation-aware output. Missing sidecar creation still uses
  fresh rendering.
- 2026-05-17: Focused validation passed:
  - `cargo fmt --all -- --check`.
  - `cargo check -p nako-nfo --tests`.
  - `cargo nextest run -p nako-nfo --no-fail-fast`: 12 tests passed.
  - `cargo check --workspace --tests`.
- 2026-05-17: Closeout validation passed:
  - `cargo check --workspace --tests`.
  - `cargo nextest run --workspace --no-fail-fast`: 298 tests passed.
  - `git diff --check`: passed with Git CRLF normalization warnings only.

## Closeout Evidence

- Codec tests prove unknown XML fields survive preservation-aware update.
- Codec tests prove Nako-owned fields update from `NfoDocument`.
- Codec tests prove duplicate/alias owned fields are reported as conflicts.
- Export service tests prove forced export preserves existing unknown sidecar
  fields while updating Nako-owned fields.
- Import-then-forced-export service test proves unknown sidecar fields survive
  the full local metadata round trip.
- Existing import and new-sidecar export behavior remains compatible.
