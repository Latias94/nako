# Addon Install Guide Generation

Status: Active
Last updated: 2026-05-22

This workstream productizes **Addon Install Guide** generation for already
registered **Addon Sidecars**.

The lane deliberately stops before **Addon Manager**. Taru may generate
operator-facing Docker Compose, systemd, Secret Reference, health-check, and
registration verification instructions, but it must not install, start, stop,
update, remove, supervise, or inspect Addon Sidecar processes.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
