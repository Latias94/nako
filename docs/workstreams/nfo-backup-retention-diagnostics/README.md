# NFO Backup Retention And Diagnostics

Status: Completed

Goal: M50 NFO backup retention and admin diagnostics.

This workstream builds on M49. M49 creates same-directory backups before
overwriting existing NFO sidecars. M50 bounds those backups with a retention
policy and makes backup/pruning results inspectable through internal and admin
diagnostics without changing the public client protocol.

The completed slice keeps responsibilities separated:

- VFS/local storage owns backup naming, backup listing, and pruning mechanics.
- `taru-nfo` owns export workflow decisions and per-item summary diagnostics.
- `taru-api`/`taru-server` keep diagnostics admin-facing through existing job
  summary passthrough.
- `taru-client-protocol` remains unchanged.

Close-out evidence:

- VFS keep-latest retention is covered by local backend tests for pruning,
  unrelated-file preservation, zero-retention pruning, and prune failure
  reporting.
- NFO forced export records backup creation and retention diagnostics.
- Admin job response preserves retention summary JSON without public protocol
  changes.
- Workspace validation passed with 315 nextest tests.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
