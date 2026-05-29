# Playback Runtime Resource Scheduler — Milestones

Status: Active
Last updated: 2026-05-29

## M0 — Scope And Evidence Freeze

Status: Completed

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs/docs/workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/playback-runtime-resource-scheduler/DESIGN.md`
- `docs/workstreams/playback-runtime-resource-scheduler/TODO.md`

## M1 — Admission Vocabulary Proof

Status: Completed

Exit criteria:

- Playback resource demand is represented by typed server-owned values.
- Admission decisions can explain accepted and rejected resource classes.
- The first tests prove the model without changing public route behavior.

Primary gates:

- `cargo nextest run -p nako-server playback_resource --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M2 — HLS And Remux Permit Enforcement

Status: Completed

Exit criteria:

- HLS and remux start paths acquire admission permits before process-backed
  runtime work starts.
- Existing session reuse does not double-acquire permits.
- Cancellation and failure release permits deterministically.

Primary gates:

- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M3 — Diagnostics And Operations Surface

Status: Completed

Exit criteria:

- Admin diagnostics report configured capacity and current pressure.
- Runtime pressure reporting remains redaction-safe.
- Operators can distinguish busy, unavailable, and unsupported playback
  conditions.

Primary gates:

- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M4 — Closeout

Status: Pending

Exit criteria:

- Fresh focused gate evidence is recorded.
- Architecture docs reflect the shipped runtime scheduler behavior.
- Queueing, remote workers, OS isolation, and per-device tuning are completed,
  deferred, or split into named follow-ons.
- `WORKSTREAM.json` status is updated.
