# Addon Notification Provider Adapters

Status: Complete
Last updated: 2026-05-25

## Purpose

Define and implement sidecar-owned notification provider adapters after the
ACK-only notification bridge proof. This lane exists so provider credentials,
message templates, outbound provider calls, and provider-specific retry behavior
remain outside Nako core.

## Authoritative Files

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current State

This lane is closed. The first provider target is implemented:
`http_webhook`, a sidecar-owned outbound HTTP webhook sink behind the existing
`library.scanned` event route.

Remaining provider breadth is split into named follow-ons in `DESIGN.md` and
`HANDOFF.md`.
