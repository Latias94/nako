# M1 Operator Journey Smoke Evidence

Status: Phase 2.1 implementation complete.
Date: 2026-06-06

## Entry Point

The repeatable smoke artifact is:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1
```

Default `-Mode fast` composes existing gates rather than adding product
runtime behavior:

- `scripts/release-gate.ps1 -Mode docs -SkipRedactionInventory`
- `scripts/self-host-smoke.ps1`
- `npm run test --prefix apps/admin-web -- App.test.tsx src/surfaces/media/mediaSurface.test.tsx`

Available focused modes:

- `-Mode docs` runs only the docs-safe release gate.
- `-Mode server` runs docs-safe release gate plus backend self-host smoke.
- `-Mode admin-web` runs docs-safe release gate plus Admin Web route/media
  tests.
- `-Mode fast` runs all of the above focused M1 smoke checks.

## Product-Operator M1 Mapping

| M1 journey step | Concrete smoke coverage |
| --- | --- |
| One Media Library is configured and visible | `apps/admin-web/src/App.test.tsx` covers `/libraries`, `/libraries/:libraryId`, mock fallback, unsafe field exclusion, and library management detail. |
| Scan/index work can be requested or observed | `App.test.tsx` covers confirmed library commands and source inventory; `crates/nako-server/src/http/tests/self_host_smoke.rs` covers scan enqueue in the server operator smoke. |
| Media/catalog browse exposes entries or source inventory | `App.test.tsx` covers catalog governance, media catalog, item detail, and source inventory; `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx` covers Media Web library/detail/search browse. |
| Playback readiness is visible | `mediaSurface.test.tsx` covers source selection, playback decision preview, ticketed browser player, HLS fallback behavior, and safe player retry states; server `self_host_smoke` covers playback decision/range streaming. |
| Admin diagnostics and repair surfaces remain available | `App.test.tsx` covers playback sessions, storage staging diagnostics, catalog governance repair context, Jobs filters, and route fallback behavior. |
| Redaction stays safe | `App.test.tsx`, `mediaSurface.test.tsx`, and server `self_host_smoke` assert unsafe tokens, paths, source locators, output paths, fingerprints, and bearer-ticket material are not rendered or returned in checked surfaces. |

## Historical Evidence Kept Linked

- `docs/workstreams/mvp-release-shape/CLOSEOUT.md`
- `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md`
- `docs/workstreams/self-hosted-release-readiness/EVIDENCE_AND_GATES.md`

These remain historical release/Web evidence. The new script is the focused M1
operator journey smoke entry point and does not reopen any legacy workstream.

## Scope Guard

No schema, generated contract, public/admin API route shape, release artifact,
source hash automation, duplicate merge automation, or runtime behavior change
is introduced by this slice.

## Spec Update

- Phase 3 spec-update review completed.
- `.trellis/spec/nako-server/backend/quality-guidelines.md` now records the
  executable contract for `scripts/m1-operator-journey-smoke.ps1`, including
  modes, validation behavior, redaction constraints, and wrong/correct examples.
- The spec update was required because this task added a new repeatable
  PowerShell smoke command surface.

## Commands Run

- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 -Mode fast`
  passed.
  - Nested docs-safe release gate passed: `cargo fmt --all -- --check` and
    `git diff --check`.
  - Nested server smoke passed: `cargo nextest run -p nako-server
    self_host_smoke --no-fail-fast`, 1/1 test passed and 651 skipped.
  - Nested Admin Web smoke passed: `npm run test --prefix apps/admin-web --
    App.test.tsx src/surfaces/media/mediaSurface.test.tsx`, 2 files and
    111 tests passed.
- `python ./.trellis/scripts/task.py validate 06-06-m1-operator-journey-smoke`
  passed: `implement.jsonl` and `check.jsonl` are valid.
- PowerShell parser check for `scripts/m1-operator-journey-smoke.ps1` passed.
- `git -c core.autocrlf=false diff --no-index --check -- /dev/null
  scripts/m1-operator-journey-smoke.ps1` passed.
- `git -c core.autocrlf=false diff --no-index --check -- /dev/null
  .trellis/tasks/06-06-m1-operator-journey-smoke/evidence.md` passed.

## Residual Gaps

- This smoke composes existing deterministic coverage; it does not prove an
  actual live operator browser session against a running self-hosted instance.
- Source hash scheduling, source duplicate operator flow, Media Web player
  hardening, and one-command release ladder packaging remain separate M1 queue
  tasks.
