# NFO Storage Write Policy

Status: Completed

Goal: M48 NFO storage write policy and persistence diagnostics.

This workstream builds on M47's XML preservation model. M47 made existing NFO
XML semantically safer to update; M48 makes the file write boundary safer and
more diagnosable for local storage.

The first slice keeps storage policy out of `taru-nfo`'s codec. The codec owns
XML preservation. VFS/storage owns write mechanics. The NFO workflow chooses the
appropriate write policy and records item-level diagnostics for parse,
preservation, conflict, and write failures.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
