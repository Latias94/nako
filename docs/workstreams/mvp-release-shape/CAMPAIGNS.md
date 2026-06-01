# MVP Implementation Campaigns

Status: MRS-050 campaign integration ledger
Last updated: 2026-06-01

This file records the implementation campaigns produced by the MVP release
shape workstream. It is an execution overlay, not a replacement for each lane's
own task ledger.

## Parallelism Policy

- Keep one upper planner terminal responsible for sequencing, result intake,
  review routing, and follow-on splits.
- Workers may edit only their assigned lane scopes and must not mutate active
  task ledgers owned by other lanes.
- Every worker returns through `integrate-lane-results` with final status
  `DONE`, `DONE_WITH_CONCERNS`, `BLOCKED`, or `NEEDS_CONTEXT`.
- Stop for ADR changes, schema migrations, public/Admin API contract changes,
  related-repo changes, or dirty unrelated files.
- Do not copy GPL reference source, tests, schemas, generated code, comments,
  or assets from `repo-ref/jellyfin`.

## Campaign A - Playback Runtime Closeout

Status: integrated on `main`

Owner lane: `playback-transcode`

Workstream: `docs/workstreams/playback-transcode-jellyfin-class-hardening/`

Current task: `PTJCH-220`

Worker:

- Agent nickname: `Lorentz`
- Agent id: `019e8110-66ab-7e90-93d7-4082b9a020ae`
- Worktree:
  `F:/SourceCodes/Rust/nako-worktrees/nako-ptjch-130-ffmpeg-adapter-split`
- Branch: `work/ptjch-220-playback-runtime-boundary`

Integration:

- Worker result: `DONE_WITH_CONCERNS`, accepted after integrator review and a
  small `cancel_requested` HLS runner permit fix.
- Branch commit: `d6712d14 fix(playback): harden HLS supersede admission`.
- Main merge: `chore(integration): merge PTJCH-220 playback runtime`.
- Branch gate: `cargo nextest run -p nako-server hls playback --no-fail-fast`
  passed 153/153 on final rerun.
- Post-merge gate:
  `cargo nextest run -p nako-server hls_playlist_playback_seek --no-fail-fast`
  passed 2/2 on `main`.

Goal:

Clarify Playback Runtime ownership for sessions, admission, reuse, supersede,
cancel, failure classification, and diagnostics without moving transcode
planning back into `nako-server`.

Allowed scopes:

- `crates/nako-server/src/app/playback`
- playback HTTP/app integration directly required by `PTJCH-220`
- `docs/workstreams/playback-transcode-jellyfin-class-hardening/`
- `docs/architecture/PLAYBACK.md` only if the ownership map changes

Required gates:

```text
cargo nextest run -p nako-server hls playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

MVP decision:

Resolved for MVP planning. `PTJCH-310` artifact I/O pressure remains a
follow-on unless release-candidate playback gates escalate it.

## Campaign B - Web/Public Client Live MVP Smoke

Status: integrated on `main`

Owner lane: `web-product`

Workstream slug: `web-mvp-live-smoke`

Worker:

- Agent nickname: `Beauvoir`
- Agent id: `019e811e-e155-7651-859f-1e1a895a9ea2`
- Worktree:

```text
F:/SourceCodes/Rust/nako-worktrees/nako-web-mvp-live-smoke
```

- Branch:

```text
work/web-mvp-live-smoke
```

Integration:

- Worker result: `DONE`, accepted after integrator review and fresh Web gates.
- Branch commit: `8d0a05b0 test(web): add MVP live smoke coverage`.
- Main merge: `chore(integration): merge web MVP live smoke`.
- Post-merge gates:
  `npm --prefix web run test -- src/test/mvp-live-smoke.test.tsx`,
  `npm --prefix web run check`, and
  `npm --prefix web run build:budget` passed on `main`.

Goal:

Make the browser/Web MVP path reproducible as release evidence: library browse,
item detail, browser playback ticket, native video render, playback heartbeat,
and no secret/raw-path exposure.

Allowed scopes:

- `web/src/api/public`
- `web/src/features/media`
- `web/src/test`
- Web smoke scripts or Playwright/browser evidence paths if already used by
  the repo
- a new `docs/workstreams/web-mvp-live-smoke/` workstream

Stop conditions:

- Public Client or Admin API contract changes.
- Backend route changes.
- Tauri/native desktop runtime decisions.
- Any need to promote desktop/native playback into MVP.

Required gates:

```text
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
```

Browser smoke must cover:

- `/media`
- `/media/library?id=<library_id>`
- `/media/detail?id=<item_id>&type=<media_type>`
- browser playback ticket creation for a source;
- native video/subtitle rendering;
- heartbeat via `playback_session_id`;
- no console errors or raw secret/path exposure.

MVP decision:

Resolved for MVP planning as a deterministic Web/Public Client smoke. Keep
the Gate 3 Web commands in the release ladder; broaden only if release
candidate browser QA finds a real gap.

## Campaign C - Release Ladder Wrapper

Status: optional split

Owner lane: `operations-release`

Suggested workstream slug: `mvp-release-gate-wrapper`

Goal:

Wrap the documented MVP validation ladder into a repo-owned command after the
team decides that one-command proof is required for the alpha release.

Allowed scopes:

- `scripts/release-gate.ps1`
- `scripts/release-gate.sh`
- `docs/deployment/`
- `docs/workstreams/mvp-release-gate-wrapper/` if opened

Stop conditions:

- Changing package contents or published artifact contracts without release
  review.
- Making Docker/PostgreSQL mandatory without a release decision.
- Hiding skipped optional gates as passed.

Required gates:

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode fast
git diff --check -- scripts docs/deployment
```

MVP decision:

Useful but not automatically required. The documented ladder is sufficient for
planning; a wrapper becomes P0 only if release management requires one command.

## Campaign D - Official Addon Alpha Smoke

Status: conditional

Owner lane: `addons-automation`

Related repo: `F:/SourceCodes/Rust/nako-official-addons`

Goal:

Run or repair the official addon alpha smoke only if the MVP release claims an
official Addon Sidecar proof beyond protocol/reference behavior.

Required gate:

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

Stop conditions:

- Any change in `nako-official-addons` without verifying branch, dirty state,
  and sync point first.
- Addon Manager lifecycle, package install/update, marketplace, or process
  supervision work.
