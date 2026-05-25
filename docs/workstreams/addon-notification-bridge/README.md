# Addon Notification Bridge

Status: Complete
Last updated: 2026-05-25

## Purpose

Build the first real event-driven official Addon after Addon Event Scheduler And
Replay. The lane proves that Nako can emit a durable event, schedule an Addon
Event Subscription, call an external sidecar, and record redaction-safe delivery
evidence without moving notification-provider credentials or provider-specific
message formatting into Nako core.

## Authoritative Files

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Closeout

This lane is closed. The ACK-only notification bridge proof is implemented,
registered in Nako's official Addon catalog, verified through scheduled
`library.scanned` delivery, and split from real provider adapters, which now
live in `docs/workstreams/addon-notification-provider-adapters/`.
