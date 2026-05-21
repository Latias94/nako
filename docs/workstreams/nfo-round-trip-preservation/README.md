# NFO Round Trip Preservation

Status: Completed

Goal: M47 NFO Round Trip preservation model.

This workstream deepens `taru-nfo` before library file write/link policy work.
The current exporter can generate Taru-owned movie XML, but forced export over
an existing sidecar rewrites the whole document. That is risky for self-hosted
libraries because hand-authored fields and fields written by other media
servers can be silently discarded.

M47 makes NFO export preserve unknown XML fields, update only Taru-owned fields,
and report conflicts in a test-visible model. It keeps the scope narrow: no
broad Jellyfin/Kodi/Plex compatibility, no public API change, no database schema
change, and no soft/hard link management.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
