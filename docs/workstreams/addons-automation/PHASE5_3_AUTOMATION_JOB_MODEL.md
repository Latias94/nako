# Phase 5.3: Automation Job Model

Status: completed.

## Goal

Model external API-key backed automation jobs for recommendation, metadata
cleanup, summary generation, and title matching without adding local model,
embedding, or vector infrastructure.

## Completed Shape

- Added automation provider configuration records with secret references,
  declared capabilities, timeout, retry budget, and enabled/disabled status.
- Added automation job input and summary envelopes that reuse Taru's persisted
  `jobs` table with `JobKind::Automation` and resource class
  `automation.external_api`.
- Added automation artifacts for generated outputs. Artifacts start as
  `proposed` and can later be accepted or rejected explicitly.
- Added SQLite migration `0011_automation.sql` for provider configuration and
  generated artifacts.
- Added `AutomationRepository` and SQLite implementation.
- Expanded `taru-automation` into a bounded provider runner with timeout,
  cancellation token, safe error mapping, mockable provider trait, job enqueue,
  and artifact persistence.
- Added HTTP routes to configure providers, enqueue automation jobs, and inspect
  generated artifacts.

## Job Kinds

The first automation capabilities are:

- `recommendation`
- `metadata_cleanup`
- `summary`
- `title_match`

All are represented as `JobKind::Automation` with a capability field in the
job input. This keeps the public job lifecycle uniform while allowing new
automation capabilities to be added without new job tables.

## Artifact Policy

Generated outputs are not canonical metadata. M5.3 stores them as artifacts
with status:

- `proposed`
- `accepted`
- `rejected`

The runner rejects provider outcomes that claim they have already mutated
canonical metadata. Acceptance policy and canonical writeback are future work.

## HTTP Surface

Initial routes:

- `POST /automation/providers`
- `GET /automation/providers`
- `GET /automation/providers/{provider_id}`
- `POST /automation/jobs`
- `GET /automation/jobs/{job_id}/artifacts`
- `GET /items/{item_id}/automation/artifacts`

HTTP routes enqueue and inspect work. They do not call external automation
providers inline.

## Non-Goals

- No concrete OpenAI-compatible provider implementation yet.
- No local model runtime, embedding pipeline, vector database, or GPU model
  scheduler.
- No canonical metadata writeback from generated outputs.
- No automatic outbox-triggered automation scheduler yet.

## Validation

Coverage:

- `taru-db` tests verify automation provider and artifact persistence.
- `taru-automation` tests verify mocked provider execution, proposed artifact
  persistence, job summary creation, secret omission from job input, and
  rejection of canonical metadata mutation.
- `taru-server` HTTP tests verify provider configuration, job enqueue, secret
  omission, and artifact inspection.
- Workspace gates pass: `cargo fmt --all -- --check`, `cargo check
  --workspace`, `cargo nextest run --workspace`, and `git diff --check`.
