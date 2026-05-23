# Addon Source Catalog And Marketplace

Status: Completed
Last updated: 2026-05-24

This workstream delivered the source catalog and marketplace discovery layer after
the completed Addon Manager lifecycle lane. Nako now knows how to confirm
operator-owned install/update/remove intent; this lane defines how addon
sources are listed, resolved, and presented as installable catalog entries.

The shipped first slice exposes a redaction-safe built-in official source,
catalog entries, and install descriptor resolution for the official metadata
scraper without registering addons, creating jobs, executing manager intent,
downloading packages, signing packages, or supervising sidecar processes.

Package signing, provider breadth, rollback/update execution, authenticated
outbound task dispatch credentials, official-addon task-path smoke, and direct
process/container supervision are split follow-ons.

Authoritative docs:

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and Gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)
