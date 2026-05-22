# Self-Hosted Release Readiness

Status: Completed
Last updated: 2026-05-21

This workstream turns Nako's completed server/runtime architecture into a
repeatable self-hosted release baseline.

The current codebase has many completed feature and architecture lanes:
PostgreSQL runtime support, Managed Artwork parity, Addons, Automation,
Webhooks, NFO write policy, Playback Runtime, Admin/Public API contracts, SDK
generation, and Android/Admin Web foundations. The next risk is no longer one
missing feature; it is release trust: operators need a reproducible way to
deploy, upgrade, diagnose, back up, and verify Nako under SQLite and
PostgreSQL.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)
