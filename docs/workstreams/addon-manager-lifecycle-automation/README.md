# Addon Manager Lifecycle Automation

Status: Completed
Last updated: 2026-05-23

This workstream delivered the first Addon Manager control-plane slice after the
completed manual-sidecar alpha loop. Nako now has a manager-owned registry/plan
slot that combines addon registration detail, Addon Health Check, Addon Token
summaries, accepted grants, Addon Install Guide output, and explicit
operator-confirmed install/update/remove lifecycle intent.

Marketplace hosting, package signing, provider breadth, rollback policy, and
direct container or process supervision are split follow-ons.

Authoritative docs:

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and Gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [Machine-readable summary](WORKSTREAM.json)
