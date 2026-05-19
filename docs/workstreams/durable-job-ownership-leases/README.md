# Durable Job Ownership Leases

Status: Active
Last updated: 2026-05-19

## Purpose

Taru now has durable job rows, a runtime supervisor, Admin job read models, and
an opt-in Managed Artwork ingest worker. The remaining correctness gap is
durable ownership: a job can be marked `running`, but the database cannot say
which worker owns it, whether the ownership lease is still valid, or whether an
operator cancellation request has been observed.

This lane defines and implements the shared ownership, lease, heartbeat, and
cancel-request boundary before more job kinds move onto worker loops or expose
truthful cancellation controls.

## Goals

- Define a durable worker identity and fencing-token model for claimed jobs.
- Add lease and heartbeat semantics that make stale running jobs recoverable
  without failing queued work.
- Model cancellation as a durable request that a worker must observe at a
  checkpoint before Taru reports the job as cancelled.
- Keep per-job-kind execution typed; the shared layer owns lifecycle mechanics,
  not domain side effects.
- Keep Admin responses redacted: no raw job inputs, summaries, errors, source
  locators, storage handles, local paths, provider payloads, or secrets.

## Non-Goals

- Distributed scheduling across multiple Taru server processes in the first
  slice.
- Automatic retry or backoff policy.
- Migrating every job kind in one pass.
- Unsafe task killing or pretending process-local abort equals durable
  cancellation.
- Folding playback/transcode session cancellation into generic jobs.
- Exposing raw `input_json`, `summary_json`, error strings, Source Locators,
  storage URIs, cache paths, local paths, token material, or provider payloads
  through Admin or Public Client APIs.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

`DJOL-050` is complete: Admin can request job cancellation through a redacted
route that distinguishes queued, running, and terminal jobs. The next slice is
`DJOL-060`: close the lane if final gates pass, or split worker-side
cancellation checkpoints and broader worker migrations into follow-ons.

## Split Follow-Ons

- Generic retry and backoff policy.
- Metadata, webhook, NFO, automation, scan/probe, and cleanup worker migration.
- Multi-process distributed scheduling and lease stealing policy.
- Public or mobile client exposure of job controls.
