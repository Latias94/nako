# M1 Ladder Evidence Matrix

Status: Product-Operator M1 release evidence baseline
Last updated: 2026-06-06

Use this matrix when preparing or reviewing M1 release-candidate evidence. It
maps `scripts/m1-release-ladder.ps1` modes to the product journey, required
tooling, evidence state, and follow-up routing. It does not replace the
operator release checklist; it explains what each ladder mode proves.

## Scope

The M1 ladder is Product-Operator first. A release candidate must show that one
self-hosted operator can configure a Media Library, scan/index media, browse
catalog entries, play video, and diagnose or repair common failures from Admin
surfaces.

The runner is an orchestrator. Detailed assertions remain owned by
`scripts/release-gate.ps1`, `scripts/m1-operator-journey-smoke.ps1`,
`scripts/self-host-smoke.ps1`, focused Rust tests, and focused Admin Web tests.

## Recording Rules

- Record the exact command, date, git revision, operator host, and result.
- Record `passed`, `failed`, or `skipped`; never treat skipped as passed.
- For skipped environment-dependent gates, record the missing tool or external
  dependency and the owner who will rerun it.
- Do not paste PostgreSQL URLs, bearer tokens, playback tickets, local media
  paths, Source Locators, playback output paths, source fingerprints, content
  hashes, or secret environment values into evidence notes.
- Use `<provided>`, `<redacted>`, or an environment variable name for sensitive
  values.

## Mode Matrix

| Mode | Command | Delegates to | Required tooling | Evidence proved | Current evidence state | Run when | Failure or skip route |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `docs` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs` | `release-gate.ps1 -Mode docs` | PowerShell, Rust toolchain, `cargo`, `cargo fmt`, `rg` | Formatting, whitespace, release docs hygiene, and redaction inventory unless explicitly skipped | Passed without `-SkipRedactionInventory` in `.trellis/tasks/archive/2026-06/06-07-m1-rc-closeout-evidence/`; earlier local runner evidence passed with inventory skipped | Every local closeout and release candidate | operations-release |
| `smoke` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode smoke` | `m1-operator-journey-smoke.ps1 -Mode fast` | PowerShell, Rust toolchain, `cargo nextest`, Node/npm, Admin Web dependencies | Focused M1 operator journey smoke: server self-host, Admin Web route/media coverage, and docs gate | Covered by `fast` evidence in runner task | When validating only the product smoke without release-fast checks | web-product, control-plane, storage-vfs, playback-transcode by failing step |
| `fast` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1` | `release-gate.ps1 -Mode docs` plus `m1-operator-journey-smoke.ps1 -Mode fast -SkipDocsGate` | PowerShell, Rust toolchain, `cargo nextest`, Node/npm, Admin Web dependencies | Default local M1 confidence path: docs-safe hygiene plus Product-Operator M1 smoke | Passed in runner task with `-SkipRedactionInventory` | Every M1 feature closeout that claims release-journey confidence | Failing delegated gate owner; do not route blindly to Media Web |
| `release-fast` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode release-fast` | `release-gate.ps1 -Mode fast` | PowerShell, Rust toolchain, `cargo nextest`, Node/npm, generated SDK tooling | Technical release preflight: DB/API/SDK checks, managed-artwork focused gates, server self-host smoke | Passed in `.trellis/tasks/archive/2026-06/06-06-m1-release-fast-evidence-run/` | Release-candidate validation or broad API/server changes | operations-release plus owning crate lane |
| `playback` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode playback` | `release-gate.ps1 -Mode playback` | PowerShell, Rust toolchain, `cargo nextest`, FFmpeg, FFprobe | Playback readiness: FFmpeg/FFprobe, transcode hardware report, HLS, and server self-host smoke | Passed in `.trellis/tasks/archive/2026-06/06-06-m1-playback-evidence-run/` | Release candidate, playback/runtime changes, or host playback support review | playback-transcode |
| `container` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode container` | `release-gate.ps1 -Mode container` | PowerShell, Rust toolchain, `cargo nextest`, Docker Compose | Container config shape for SQLite and PostgreSQL compose stacks plus server config tests | Passed in `.trellis/tasks/archive/2026-06/06-06-m1-container-evidence-run/` | Any release claiming container install support | operations-release |
| `postgres` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode postgres -PostgresUrl <provided>` | `release-gate.ps1 -Mode postgres` | PowerShell, PostgreSQL-compatible test database or default harness tooling | PostgreSQL managed-artwork contract harness evidence | Passed in `.trellis/tasks/archive/2026-06/06-06-m1-postgres-evidence-run/`; broader `all-contracts` PostgreSQL harness also passed there | Release candidate claiming PostgreSQL readiness beyond preview support | storage-vfs or nako-db depending on failing contract |
| `workspace` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode workspace` | `release-gate.ps1 -Mode workspace` | PowerShell, full Rust workspace toolchain, `cargo nextest` | Full workspace Rust check and nextest sweep | Passed after repair in `.trellis/tasks/archive/2026-06/06-06-m1-workspace-evidence-run/`; repair commit `7769bc47` stabilized HLS timing gates | Release-candidate closeout or broad cross-crate changes | owning crate lane identified by first failure |
| `all` | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode all` | `fast`, `release-fast`, `playback`, `container`, `postgres`, and `workspace` | All tooling for every mode | Full scripted M1 ladder evidence except future live-browser/package-publication proof | Available; expensive and environment-dependent | Release-candidate validation or CI-style evidence | split by failing delegated mode |

## M1 Journey Coverage

| M1 quality area | Minimum ladder evidence | Current evidence source | Follow-up if weak |
| --- | --- | --- | --- |
| Install/config readiness | `docs`, `release-fast`, `container` when publishing containers | `scripts/release-gate.ps1`; `docs/deployment/RELEASE_CHECKLIST.md`; compose config gates | operations-release |
| Library scan and source identity | `fast` and `smoke` | `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/`; scan-originated source hash evidence | storage-vfs or control-plane |
| Catalog and media browse | `fast` and `smoke` | Admin Web `App.test.tsx`; Media surface tests recorded in M1 smoke evidence | web-product |
| Playback readiness | `fast` plus `playback` before release-ready claims | M1 smoke evidence; `release-gate.ps1 -Mode playback`; hardware report output | playback-transcode |
| Admin diagnostics and repair | `fast`; follow with a diagnostics/repair audit if coverage is unclear | Admin Web route tests and source duplicate reconciliation evidence | `m1-admin-diagnostics-repair-coverage-audit` |
| Redaction and support safety | `docs` without skipping inventory for release candidates; `fast` for checked surfaces | redaction inventory; M1 smoke redaction assertions | operations-release or owning feature lane |
| PostgreSQL readiness | `postgres` with a redacted URL or default harness | PostgreSQL contract harness | nako-db or storage-vfs |
| Workspace regression confidence | `workspace` | full workspace check and nextest sweep | owning crate lane |

## Skip Classification

| Skip reason | Acceptable for local feature closeout | Acceptable for release candidate | Required note |
| --- | --- | --- | --- |
| Docker unavailable for `container` | Yes | No, unless release does not claim container support | Record Docker version or missing Docker reason |
| PostgreSQL URL unavailable for `postgres` | Yes | Only if release does not claim PostgreSQL readiness beyond preview support | Record `NAKO_TEST_POSTGRES_URL` unavailable, not the URL |
| FFmpeg/FFprobe unavailable for `playback` | No for playback changes | No for video-first M1 | Record missing binary and host OS |
| Full workspace gate too expensive | Yes for narrow feature closeout | No for release candidate | Record focused gates that did run |
| Redaction inventory skipped | Yes for repeated local iteration | No for release candidate | Record why it was skipped and when it will run |

## Evidence Template

```text
M1 ladder evidence
Date:
Git revision:
Host:
Command:
Result: passed | failed | skipped
Skipped gates:
Artifacts:
Follow-up owner:
Notes:
```

## Drift Guard

When `scripts/m1-release-ladder.ps1` adds, removes, or changes a mode, update
this matrix in the same task. At minimum, compare this document against the
runner's `ValidateSet` and update the mode matrix, journey coverage, skip
classification, release checklist, operations architecture, and Trellis spec
guidance together.
