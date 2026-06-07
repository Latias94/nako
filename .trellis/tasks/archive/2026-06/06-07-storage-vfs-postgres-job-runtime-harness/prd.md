# Storage VFS PostgreSQL Job Runtime Harness

## Goal

Add a focused PostgreSQL contract harness suite for durable job runtime parity
so M2 storage/control-plane reliability work can verify job lease, cancellation,
retry, queue-pressure, and priority policy contracts without running every
ignored PostgreSQL contract.

## What I Already Know

- `scripts/postgres-contract-harness.ps1` and
  `scripts/postgres-contract-harness.sh` already support focused suites:
  `managed-artwork`, `storage-runtime`, `source-identity`,
  `storage-source-parity`, and `all-contracts`.
- Existing `nako-db` contract tests already define PostgreSQL job lease and job
  retry contracts:
  - `postgres_job_lease_contract_claims_next_with_worker_token_and_filter`
  - `postgres_job_lease_contract_heartbeats_and_completes_with_run_token_fence`
  - `postgres_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner`
  - `postgres_job_lease_contract_recovers_only_expired_running_leases`
  - `postgres_job_retry_contract_persists_backoff_and_redacted_queue_pressure`
  - `postgres_job_retry_contract_priority_policy_orders_fairly_and_recovers`
- `docs/architecture/CONTROL_PLANE.md` lists durable job priority, retry,
  queue pressure, resource classes, and broader job-kind scheduler migration as
  reliability foundations/follow-ons.
- `docs/architecture/STORAGE_VFS.md` says broader PostgreSQL runtime harness
  evidence for future storage/control-plane query paths remains a follow-on.
- The `nako-db` quality spec already requires PowerShell and Bash harness suite
  parity, explicit filters for focused suites, and preserving safe skip /
  local cluster behavior.

## Requirements

- Add a new focused suite named `job-runtime` to both PostgreSQL harness
  scripts:
  - `scripts/postgres-contract-harness.ps1`
  - `scripts/postgres-contract-harness.sh`
- The suite must run explicit PostgreSQL filters for existing job lease and
  job retry contracts only.
- Do not change repository traits, SQL, migrations, runtime behavior, API
  shape, or product behavior.
- Update durable release/readiness docs that enumerate harness suites.
- Update `.trellis/spec/nako-db/backend/quality-guidelines.md` so future agents
  know `job-runtime` is a valid focused suite and when to use it.
- Record validation evidence in `evidence.md`.

## Acceptance Criteria

- [x] PowerShell harness accepts `-Suite job-runtime` and maps it to explicit
      job lease/retry PostgreSQL filters.
- [x] Bash harness accepts `--suite job-runtime` and maps it to the same filter
      set.
- [x] `storage-source-parity` remains storage/source-specific and does not
      silently absorb job runtime contracts.
- [x] `all-contracts` remains the only broad `postgres_` filter suite.
- [x] Documentation lists `job-runtime` as a focused PostgreSQL contract suite.
- [x] Trellis task context validation passes.
- [x] Focused SQLite job lease/retry tests pass.
- [x] PowerShell PostgreSQL harness `-Suite job-runtime` passes when local
      PostgreSQL tooling is available, or records a safe skip if unavailable.
- [x] Bash syntax check passes.
- [x] `git diff --check` passes.

## Definition Of Done

- Code, docs, spec, and task evidence are committed together.
- No product/runtime behavior changes are included.
- If local PostgreSQL tooling is unavailable, the evidence must say so and
  demonstrate safe skip behavior rather than claiming parity passed.

## Technical Approach

- Introduce a shared explicit filter list in each harness script for:
  - job lease claim/filter;
  - job lease heartbeat/completion with run-token fencing;
  - job cancellation acknowledgement;
  - expired lease recovery;
  - retry backoff and redacted queue pressure;
  - priority/fairness recovery.
- Add `job-runtime` to each command's valid suite list and usage text.
- Update release and architecture docs that describe harness suite selection.
- Validate with narrow contract filters before running the new PostgreSQL
  suite.

## Decision (ADR-lite)

**Context**: M2 storage/VFS reliability now has focused PostgreSQL harness
suites for storage runtime and source identity. Durable job runtime parity is
also load-bearing for VFS repair, source hashing, scan scheduling, and future
job-kind scheduler migration, but currently requires either ad hoc filters or
the broad `all-contracts` suite.

**Decision**: Add `job-runtime` as a focused harness suite that names only the
existing job lease/retry PostgreSQL contracts.

**Consequences**: Future storage/control-plane work can verify durable job
runtime parity cheaply and explicitly. The change does not alter persistence
semantics; it only makes existing contracts easier to run consistently across
PowerShell and Bash.

## Out Of Scope

- No new job repository method.
- No new job contract test.
- No SQL or migration change.
- No scheduler/runtime behavior change.
- No Admin API or Web UI change.
- No CI workflow wiring unless a later task promotes the suite.

## Technical Notes

- Relevant docs:
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/workstreams/self-hosted-release-readiness/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/self-hosted-release-readiness/DESIGN.md`
- Relevant specs:
  - `.trellis/spec/nako-db/backend/index.md`
  - `.trellis/spec/nako-db/backend/quality-guidelines.md`
  - `.trellis/spec/nako-db/backend/database-guidelines.md`
- Predecessor evidence:
  - `.trellis/tasks/archive/2026-06/06-03-05c-storage-runtime-postgres-parity-harness/`
  - `.trellis/tasks/archive/2026-06/06-06-06-06-postgres-source-identity-contract-harness/`
