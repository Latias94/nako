# Release Packaging And Distribution

Status: Active
Last updated: 2026-05-21

This workstream turns the verified self-hosted release baseline into a concrete
operator-facing distribution path.

The previous `self-hosted-release-readiness` lane proved Taru can be verified
locally with SQLite/PostgreSQL gates, API/SDK/redaction checks, deployment
examples, backup/restore guidance, and self-host smoke tests. The next gap is
packaging: a self-hosted operator still needs a predictable artifact, container
shape, startup contract, config validation story, and release checklist that
turns source-built Taru into something installable and upgradable.

Authoritative docs:

- [Design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)
