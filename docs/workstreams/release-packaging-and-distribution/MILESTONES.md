# Release Packaging And Distribution — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Scope And Contract

Exit criteria:

- [ ] Workstream is opened with durable docs.
- [x] Current packaging/deploy/release baseline is recorded.
- [x] Artifact contract for this lane is explicit.
- [x] Workstream is opened with durable docs.

## M1 — Startup And Config Preflight

Exit criteria:

- [x] Packaged runs have a redaction-safe config validation path.
- [x] Common operator mistakes produce actionable diagnostics.
- [x] Validation is covered by focused tests.

## M2 — Container And Compose

Exit criteria:

- [x] Nako server has a container build path or explicitly documented blocker.
- [x] Compose examples include durable volumes and safe local defaults.
- [x] Docker/compose config checks are recorded.

## M3 — Artifact Script And CI Shape

Exit criteria:

- [x] A repo-owned script can build or dry-run release artifacts.
- [x] Artifact metadata/checksums are emitted.
- [x] CI workflow shape calls repo-owned scripts.

## M4 — Operator Release Docs

Exit criteria:

- [x] Install and first-start docs exist.
- [x] Upgrade/rollback links to backup/restore guidance.
- [x] Logs, diagnostics, checksums, and support bundle expectations are clear.

## M5 — Future Lane Decision

Exit criteria:

- [x] Metadata, NFO/link, Playback/transcode, Downloads, Network traversal, and
  AI are compared by value/risk/dependencies.
- [x] Downloads has a precise first-safe-slice definition or is deferred.
- [x] The next product lane recommendation is recorded.

## M6 — Closeout

Exit criteria:

- [x] Workstream TODO is complete or residual work is split.
- [x] Evidence proves the release packaging contract.
- [x] `WORKSTREAM.json` status is completed or closed.
