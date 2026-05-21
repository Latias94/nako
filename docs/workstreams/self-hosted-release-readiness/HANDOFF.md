# Self-Hosted Release Readiness — Handoff

Status: Completed
Last updated: 2026-05-21

## Current State

This workstream is complete. SHR-010 completed the initial baseline inventory.
SHR-020 added the first repo-owned local release gate entrypoints:

- `scripts/release-gate.ps1`
- `scripts/release-gate.sh`
- `.config/nextest.toml`

The PowerShell gate has been proven in `docs`, `fast`, and PostgreSQL modes.
SHR-030 added a PostgreSQL contract harness and CI job shape:

- `scripts/postgres-contract-harness.ps1`
- `scripts/postgres-contract-harness.sh`
- `.github/workflows/release-gate.yml`

The local PostgreSQL 17 harness path has been proven with 6/6 PostgreSQL
Managed Artwork contracts and cleans `target/postgres-contract/` after use.
SHR-040 composed Admin/Public API, generated SDK/OpenAPI, Rust client/protocol,
TypeScript SDK, Admin Web contract, and redaction checks into `release-gate`
API mode. SHR-050 added self-hosted SQLite/PostgreSQL deployment examples and
operator configuration guidance. SHR-060 added backup/restore/upgrade
runbooks. SHR-070 added the self-host smoke artifact and folded it into the
release gate. SHR-080 reran docs, fast, PostgreSQL, and workspace gates and
closed the lane.

## Next Recommended Action

No action is required to continue this lane. Start a new workstream for AI,
network traversal/tunneling, Native Plugin ABI, provider breadth, SDK/package
publishing, or release packaging.

## Blockers

None known for this lane.

## Notes

- A Windows `bash` executable existed as the WSL shim in this environment and
  timed out during shell syntax probing; Git Bash was not installed. Re-verify
  `scripts/release-gate.sh` and `scripts/postgres-contract-harness.sh` in
  Linux/CI or Git Bash in CI or a Linux shell.
