# Operations And Release Architecture

Last updated: 2026-06-05

This document maps deployment, release, diagnostics, backup, and operational
readiness for a self-hosted media server.

## Target Chain

```text
Config
  -> startup validation
  -> runtime readiness diagnostics
  -> resource budgets
  -> release gates
  -> backup/restore/upgrade
  -> operator-visible troubleshooting
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Self-hosted install docs | Shipped foundation | `docs/deployment/SELF_HOSTED.md` | HTTPS/tunnel/reverse proxy cookbook. |
| Backup/restore docs | Shipped foundation | `docs/deployment/BACKUP_RESTORE_UPGRADE.md` | Include future offline sync/artifact classes. |
| Release checklist | Shipped foundation plus playback hardware report evidence, Product-Operator M1 ladder entry, and M1 ladder evidence matrix | `docs/deployment/RELEASE_CHECKLIST.md`; `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` | Optional live-browser M1 ladder mode and container device pass-through evidence. |
| Release gate scripts | Shipped playback mode, hardware report baseline, Product-Operator M1 ladder runner, and release evidence matrix | `scripts/release-gate.*`; `scripts/m1-release-ladder.ps1`; `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`; `.trellis/tasks/06-04-06-04-playback-release-gate-mode-first-slice/`; `.trellis/tasks/06-05-06-05-playback-release-hardware-report/`; `.trellis/tasks/archive/2026-06/06-06-m1-release-ladder-runner/` | Optional live-browser M1 ladder mode and container device pass-through evidence. |
| PostgreSQL contract harness | Shipped foundation | `scripts/postgres-contract-harness.*` | Keep new persistence contracts covered. |
| FFmpeg/ffprobe config | Shipped foundation | self-hosted docs; playback diagnostics lanes | Packaging and container device diagnostics. |
| Hardware readiness diagnostics | Shipped report baseline | ADR 0045-0048; admin playback runtime diagnostics; `target/release-gate/playback-hardware-report.json` | Optional one-frame device smoke and container pass-through matrix. |
| Runtime budgets | Partial | `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md` | Unified playback resource scheduler. |
| Config mutation authority | Partial | admin settings lanes | Hot-apply/restart-required model. |
| Observability | Partial | diagnostics lanes | Metrics/tracing/export profile. |

## Workstream Evidence

Use `docs/architecture/WORKSTREAM_LINKS.md#operations-and-release` as the
consolidated index for deployment, release, diagnostics, backup, and packaging
workstreams. Keep this document focused on operator readiness.

## Next Work Lanes

### playback-release-hardware-matrix

Goal: Make release gates prove playback dependencies are present and
diagnosable across supported deployment modes.

Status: Playback release-gate mode and host hardware report evidence shipped as
of 2026-06-05. Product-Operator M1 ladder orchestration is shipped through
`scripts/m1-release-ladder.ps1`, and M1 ladder evidence recording is documented
in `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`.

Scope:

- FFmpeg/ffprobe presence checks are covered by `scripts/release-gate.*`
  `playback` mode;
- CPU fallback smoke is covered through existing HLS and self-host playback
  gates;
- FFmpeg hardware capability reports are written to
  `target/release-gate/playback-hardware-report.json` without requiring GPU
  devices;
- optional VAAPI/NVENC/QSV one-frame smoke probes when devices are present;
- Docker device/pass-through documentation;
- operator-safe failure messages.
- Product-Operator M1 default ladder that composes docs-safe hygiene and the
  focused M1 operator journey smoke while leaving playback, container,
  PostgreSQL, workspace, and all-mode validation explicit.
- M1 ladder evidence matrix that records required tooling, current evidence,
  skipped-gate rules, and follow-up routing for each ladder mode.

Exit criteria:

- release gate has a playback smoke mode;
- docs explain how to verify hardware acceleration and find the generated
  report;
- missing acceleration degrades or fails according to configured policy.

### self-hosted-remote-access-cookbook

Goal: Document safe remote access without baking a tunnel provider into Nako
core.

Scope:

- reverse proxy;
- HTTPS;
- DDNS;
- Tailscale/Cloudflare Tunnel examples as operator guidance;
- playback ticket and public URL caveats.

## Risk Register

### FFmpeg Packaging Determines Playback Reality

Command planning can be correct while the host FFmpeg build lacks encoders,
filters, or hardware acceleration. Release gates and diagnostics need to make
that visible.

### Docker Hardware Access Is Operationally Fragile

VAAPI/NVENC/QSV require host drivers, devices, container runtime permissions,
and compatible FFmpeg builds. Documentation and diagnostics must be explicit.

### Generated Artifacts Need Backup Classification

Some artifacts are durable user state, some are reproducible cache, and some
are temporary playback output. Backup docs must classify each new artifact
type.

### Config Hot-Apply Can Violate Runtime Assumptions

Changing FFmpeg paths, hardware policy, database URLs, or staging roots while
sessions run can break runtime invariants. Mark settings as hot-apply or
restart-required.

## Agent Notes

When adding runtime dependencies, add deployment documentation and release-gate
evidence. A playback feature is incomplete if operators cannot diagnose why it
does not run on their host.
