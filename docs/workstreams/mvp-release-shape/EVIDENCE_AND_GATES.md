# MVP Release Shape - Evidence And Gates

Status: Active
Last updated: 2026-06-01

## Source Coverage Audit

| Source | State | Evidence path | Impact | Required action |
| --- | --- | --- | --- | --- |
| User goal | COVERED | Current planning conversation | User approved opening an MVP release-shape planning lane. | None. |
| Domain glossary | COVERED | `CONTEXT.md` | Defines Video-First Phase, Addon Sidecar, Remote Access Endpoint, Hardware Capability Report, and Single-Admin Mode. | Keep terminology aligned. |
| Product context | COVERED | `PRODUCT.md` | Defines Admin Web, Media Web, desktop, mobile, addon surfaces, and self-hosted clarity. | Use for MVP scope. |
| Architecture map | COVERED | `docs/ARCHITECTURE.md` | Confirms North Star and FFmpeg/Addons/control-plane principles. | Use as target-state guardrail. |
| Lane map | COVERED | `docs/architecture/LANES.md` | Active queue and lane ownership are known. | Keep MVP lane planner-only. |
| Active workstreams | COVERED | PTJCH, GAMA, CSAPA docs | Active tails affect MVP convergence. | Verify in MRS-020/MRS-040. |
| Related repo | COVERED | `F:/SourceCodes/Rust/nako-official-addons` status and inventory | Official addons repo is clean and has no active workstream. | Do not assign cross-repo work until a blocker is proven. |
| Jellyfin reference | COVERED | `repo-ref/jellyfin` status and AGENTS guidance | Reference source is available for behavior pressure only. | Do not copy GPL source, tests, schemas, or generated files. |
| Code evidence | COVERED | `crates/*`, `web/*`, deployment scripts | MRS-020 verified the MVP cut against selected code/test inventory, Web Public Client usage, deployment scripts, and release docs. | MRS-030 must convert this evidence into a fresh smoke/gate ladder. |

## Planning Gates

### MRS-010

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
git diff --check -- docs/workstreams/mvp-release-shape docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md
```

### MRS-020

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
git diff --check -- docs/workstreams/mvp-release-shape
```

### MRS-030

```text
git diff --check -- docs/workstreams/mvp-release-shape docs/deployment docs/architecture
```

### MRS-040

```text
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
```

Result should show no MVP-blocking readiness drift before implementation
campaigns are assigned.

## MVP Validation Ladder

Status: MRS-050 campaign-integrated release-gate baseline

Run the ladder from a clean release-candidate worktree with FFmpeg/FFprobe,
`cargo-nextest`, Rust, Node/npm, and the Web dependencies available. Docker and
PostgreSQL gates are conditional on the artifact being advertised for those
operators, but a public release should record whether they ran or were skipped.

### Gate 0 - Planner, Docs, And Redaction Preflight

Required for every release candidate.

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
git diff --check -- docs/workstreams/mvp-release-shape docs/deployment docs/architecture
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
```

This proves the MVP lane is internally consistent, architecture/deployment docs
have no whitespace errors, formatting is checked by the release gate, and the
redaction inventory runs.

### Gate 1 - Core Release Preflight

Required for every release candidate.

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
```

This is the existing one-command local release preflight. It covers `cargo fmt`,
`git diff --check`, redaction inventory, DB/API/SDK checks, managed-artwork
focused gates, and `nako-server` self-host smoke. It does not by itself prove
the full video-first media journey, so Gate 2 through Gate 5 remain required.

### Gate 2 - Server MVP Journey Focused Tests

Required until these checks are folded into a stronger release-gate mode.

```text
cargo nextest run -p nako-server self_host_smoke --no-fail-fast
cargo nextest run -p nako-server scan_library --no-fail-fast
cargo nextest run -p nako-server metadata --no-fail-fast
cargo nextest run -p nako-server storage --no-fail-fast
cargo nextest run -p nako-server addons --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server user_playback --no-fail-fast
```

This proves install/startup, local scan, NFO/provider metadata, storage
diagnostics, Addon Sidecar server paths, Direct/Remux/HLS server behavior, and
playback-state persistence at the server layer.

### Gate 3 - Web/Public Client MVP Smoke

Required for MVP. The deterministic `web-mvp-live-smoke` gate now represents
the first committed Web/Public Client release smoke; broader Web tests remain
part of this gate.

```text
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
```

Browser smoke must cover:

- `/media` loading through Public Client data sources;
- library list and `/media/library?id=<library_id>` item browse;
- `/media/detail?id=<item_id>&type=<media_type>` detail rendering;
- browser playback ticket creation for a source;
- native `VideoPlayer` media/subtitle URL rendering;
- playback heartbeat through `playback_session_id`;
- no console errors and no secret/raw path exposure in the checked surfaces.

Routing: if this gate fails, keep fixes inside `web-product` unless the
failure proves a backend/Public Client contract gap.

### Gate 4 - Playback Runtime Closeout

Required for MVP. `PTJCH-220` is now integrated on `main`; the release
candidate still needs fresh playback gate evidence for:

- session ownership and lifecycle;
- admission/reuse/supersede/cancel behavior;
- failure classification and operator diagnostics;
- Direct/Remux/HLS runtime behavior;
- missing FFmpeg diagnostics and CPU fallback;
- no raw source locator, token, or path leakage in playback diagnostics.

Routing: keep regressions on `playback-transcode`. Treat `PTJCH-310` artifact
I/O pressure as a follow-on unless the release-candidate playback gates prove
it is unsafe for MVP.

### Gate 5 - Package And Container Shape

Required for packaged releases; Docker gate is required when publishing or
documenting container artifacts.

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1 -WhatIf
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container
```

For a real release candidate, replace `-WhatIf` with the actual package command
and verify the produced manifest and checksums using
`docs/deployment/RELEASE_CHECKLIST.md`.

### Gate 6 - Official Addon Alpha Smoke

Required if the MVP release claims an official Addon Sidecar proof.

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

Default to the published official addon binary for release validation. Use
`-AddonBinarySource workspace` only when the planner has approved a clean
`nako-official-addons` candidate worktree.

### Gate 7 - PostgreSQL Compatibility

Conditional: required when the release claims PostgreSQL-ready behavior beyond
documented preview support.

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode postgres -PostgresUrl $env:NAKO_TEST_POSTGRES_URL
```

If `NAKO_TEST_POSTGRES_URL` is unavailable, record PostgreSQL as skipped rather
than implying it passed.

## Gate Gap Routing

| Gap | MVP decision | Route |
| --- | --- | --- |
| `PTJCH-220` playback runtime ownership | Resolved P0 blocker | Keep Gate 4 playback evidence in the release ladder; route artifact I/O pressure to `PTJCH-310` or PAIP only if gates escalate it. |
| Web/Public Client live release smoke | Resolved P0 gate gap | Keep `web-mvp-live-smoke` plus broader Web gates in Gate 3; broaden only if release-candidate browser QA finds a real gap. |
| Release-gate script does not wrap the full video-first ladder | P0 release-ops gap | Split an `operations-release` task if the team wants one command beyond documented per-area gates. |
| `GAMA-060` Web Generated Artifact apply | Conditional/P1 | Defer unless MVP explicitly exposes Web apply confirmation. |
| `CSAPA-050` desktop playback | P1/deferred | Record deferral in `MRS-040`; do not block browser/web MVP. |
| Vendor hardware acceleration matrix | P1 | Gate missing FFmpeg and CPU fallback for MVP; vendor lab evidence follows. |
| Remote access cookbook polish | P0 docs-only if current cookbook is judged insufficient | Keep built-in tunnel/NAT traversal out of MVP. |

## Evidence Ledger

### MRS-010 - Scope And Evidence Freeze

Status: Done

Evidence collected:

- Initial `MVP.md` release statement.
- Initial `RELEASE_CUT.md`.
- Initial `GAP_MATRIX.md`.
- Lane and workstream-link registration for `mvp-release-convergence`.

Fresh validation:

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
git diff --check -- docs/workstreams/mvp-release-shape docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md
```

Result: passed on 2026-06-01. Workstream inventory found
`mvp-release-shape` active on lane `mvp-release-convergence` with readiness
`ready`. Existing `GAMA` lane drift and `CSAPA` missing context remain known
active-queue issues for MRS-020/MRS-040. `git diff --check` emitted LF/CRLF
working-copy warnings for touched Markdown files and no whitespace errors.

### MRS-020 - Release Cut Verification

Status: Done with concerns

Evidence collected:

- Deployment and release evidence is strong enough for MVP planning:
  `docs/deployment/SELF_HOSTED.md`, `docs/deployment/RELEASE_CHECKLIST.md`,
  `scripts/release-gate.ps1`, compose files, package scripts, and container
  config templates cover local startup, config-check, self-host smoke,
  compose config, release artifacts, and official addon smoke candidates.
- Browser/web is the accepted MVP client path. Public Client route inventory
  and Web usage now show library item browse, browser playback tickets,
  playback session ids, heartbeat routes, and native video rendering tests.
  The remaining P0 risk is a fresh live Web smoke rather than route design.
- Playback Direct/Remux/HLS has broad code/test evidence, but `PTJCH-220`
  remains an MVP blocker because runtime session/admission/reuse/cancel/failure
  classification and diagnostics sit on the required journey.
- Scan, source state, local inference, NFO import, provider job planning,
  generated-artifact authority behavior, storage health, and circuit behavior
  have code/test or completed-workstream evidence. Watcher/debounce, cache
  repair, and broad provider polish remain P1 unless a gate proves otherwise.
- Addon Sidecar foundation has evidence from Admin Addon Operations, Addon
  Install Guide Generation, startup scan addon tests, and the clean
  `nako-official-addons` inventory. Official addon breadth is not P0, but
  the release gate should name one minimum sidecar smoke.
- Remote access has enough cookbook/config evidence for P0 if MVP keeps
  built-in tunnel ownership out of scope. `MRS-030` should decide whether the
  current cookbook is sufficient or needs a docs-only release blocker.
- Active queue classification: `PTJCH-220` is P0; `GAMA-060` is conditional
  and likely P1 unless the MVP exposes Web Generated Artifact apply; `CSAPA-050`
  is P1/deferred if browser/web remains the first client path.

Concerns:

- `PTJCH-220` must be finished, split, or explicitly accepted before MVP
  closeout.
- `MRS-030` must turn planning evidence into fresh command evidence; MRS-020
  did not run the full Rust/Web/release gate ladder.
- The GAMA/CSAPA active-lane drift remains visible and should be handled in
  `MRS-040`.

Fresh validation:

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
git diff --check -- docs/workstreams/mvp-release-shape
```

Result: passed on 2026-06-01. `WORKSTREAM.json` is valid, `git diff --check`
found no whitespace errors for the workstream, lane registry, and workstream
README files, and inventory reports `mvp-release-shape` readiness as `ready`
with current task `MRS-030`. Existing GAMA/CSAPA readiness issues remain
outside this workstream and are still routed to `MRS-040`.

### MRS-030 - MVP Gate Plan

Status: Done with concerns

Evidence collected:

- Existing `scripts/release-gate.ps1 -Mode fast` is a good core preflight but
  does not prove the full video-first journey by itself.
- Focused server gates are needed for scan, metadata, storage, addons,
  playback/HLS, and user playback state until they are wrapped by a stronger
  release-gate mode.
- Web/Public Client tests exist, but release still needs a reproducible live
  browser smoke that covers library browse, detail, browser playback ticket,
  native video render, and heartbeat.
- Package/container, official addon alpha smoke, and PostgreSQL gates already
  have repo-owned command entry points and can be required conditionally based
  on the artifact being released.

Fresh validation:

```text
git diff --check -- docs/workstreams/mvp-release-shape docs/deployment docs/architecture
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
```

Result: passed on 2026-06-01. `git diff --check` emitted LF/CRLF working-copy
warnings for existing Markdown normalization and no whitespace errors.
`WORKSTREAM.json` is valid. Workstream inventory reports `mvp-release-shape`
readiness as `ready` with current task `MRS-040`; existing GAMA/CSAPA readiness
issues remain unchanged and are routed to `MRS-040`.

### MRS-040 - Active Queue Alignment

Status: Done with concerns

Evidence collected:

- `PTJCH-220` is ready in `playback-transcode-jellyfin-class-hardening` and is
  the only current active-tail P0 blocker. A parallel worker was started for
  this task in the playback worktree.
- `GAMA-060` is the current GAMA task, but it is Web Admin Generated Artifact
  apply workflow. Backend apply through `GAMA-050` is already stable, so the
  remaining Web workflow is conditional/P1 for MVP unless the product cut
  explicitly exposes Web apply.
- `CSAPA-050` is a desktop playback strategy spike. Browser/web is the accepted
  MVP client path, so CSAPA is deferred/P1 for MVP and should not consume P0
  release capacity.
- Web/Public Client live smoke and an operations-release gate wrapper are
  possible `MRS-050` splits, not existing active-tail blockers.

Fresh validation:

```text
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
git diff --check -- docs/workstreams/mvp-release-shape docs/architecture/LANES.md docs/workstreams/README.md
```

Result: passed on 2026-06-01. Inventory reports `mvp-release-shape` readiness
as `ready` with current task `MRS-050`, and `PTJCH-220` remains `ready`.
Existing GAMA/CSAPA readiness issues remain visible but are classified as
non-MVP-blocking by this lane. `WORKSTREAM.json` is valid. `git diff --check`
emitted LF/CRLF working-copy warnings for touched Markdown files and no
whitespace errors.

### MRS-050 - Campaign Integration And RC Validation

Status: In progress

Evidence collected:

- `PTJCH-220` is integrated on `main`. The accepted playback runtime slice
  centralizes HLS supersede discovery/cancellation, adds bounded replacement
  admission, keeps `cancel_requested` runners in the active supersede set, and
  synchronizes superseded HLS playback-session cancellation.
- `web-mvp-live-smoke` is integrated on `main`. The deterministic Web smoke
  covers Public Client library browse/detail/playback ticket, native
  media/subtitle rendering, heartbeat, and unsafe text redaction.
- `CAMPAIGNS.md` now records Campaign A and Campaign B as integrated, while
  keeping the operations release wrapper optional and official addon alpha
  smoke conditional.
- The operations release wrapper remains optional until the team requires a
  single command for the full video-first ladder.
- Release-candidate validation has started on `main`. Gate 0 and Gate 1 pass;
  Gate 2 passes after aligning the external acquisition runner catalog resolve
  test with the official optional `transmission_password` Secret Reference
  declaration.

Fresh validation:

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
cargo nextest run -p nako-server hls_playlist_playback_seek --no-fail-fast
npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
git diff --check
```

Result: post-merge validation passed on 2026-06-01. `WORKSTREAM.json` is
valid. The PTJCH post-merge seek gate passed 2/2 on `main`. The Web smoke
focused test passed 2/2, TypeScript check passed, and bundle budget passed
with media route JS at 43.68 KiB raw / 12.07 KiB gzip and total JS at
1132.92 KiB raw / 331.89 KiB gzip. `git diff --check` passed after trimming
new-file EOF blanks from the Web smoke docs; it emitted LF/CRLF working-copy
warnings only.

Release-candidate validation:

```text
python -m json.tool docs/workstreams/mvp-release-shape/WORKSTREAM.json
python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .
git diff --check -- docs/workstreams/mvp-release-shape docs/deployment docs/architecture
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
cargo nextest run -p nako-server self_host_smoke scan_library metadata storage addons playback hls user_playback --no-fail-fast
```

Result: Gate 0, Gate 1, and Gate 2 passed on 2026-06-01. Workstream inventory
still reports `mvp-release-shape`, `playback-transcode-jellyfin-class-hardening`,
and `web-mvp-live-smoke` as ready; the known `GAMA-060` and `CSAPA-050`
readiness issues remain non-MVP-blocking. The first Gate 1 run exposed stale
Admin Web mock fixtures, fixed by `d1c8550d`. The first Gate 2 run exposed a
stale Addon catalog resolve assertion, fixed by `94385f98`, and a playback
permit timing failure that passed on focused rerun. The final Gate 2 combined
server run passed 341/341 with 169 skipped.
