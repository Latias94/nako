# NFO Sidecar Backup Policy

Status: Completed

Goal: M49 NFO sidecar backup and write conflict policy.

This workstream builds on M47 and M48. M47 protects the XML meaning of existing
sidecars. M48 makes local sidecar writes safer with explicit atomic replace.
M49 adds a storage-owned backup boundary for overwrites so users have a local
rollback artifact when Taru updates an existing NFO sidecar.

The first slice keeps backup mechanics out of the NFO codec. The codec owns XML
round trip behavior. VFS/storage owns local file backup mechanics. The NFO
workflow chooses when a backup is required and records internal diagnostics.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
