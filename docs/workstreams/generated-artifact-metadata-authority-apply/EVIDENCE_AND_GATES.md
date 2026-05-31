# Generated Artifact Metadata Authority Apply - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Always Run For Docs Changes

- `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`
- `git diff --check -- docs/workstreams/generated-artifact-metadata-authority-apply docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`

## Backend Gates

Use focused gates for each task, then broaden if public contracts or schema
change:

- `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-db metadata_application --no-fail-fast`
- `cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

If PostgreSQL schema or repository behavior changes, run the matching
PostgreSQL contract gate used by the current `nako-db` workflow.

## Web Gates

Only run after `GAMA-050` exposes a real Admin route:

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- Browser/Playwright smoke for desktop and mobile apply-plan/result states.

## Evidence Log

- `GAMA-010`: Opened the lane and recorded the audit. Current review acceptance
  stages Metadata Authority apply but does not mutate Canonical Metadata.
- `GAMA-020` (verified 2026-05-29): Added the read-only Generated Artifact metadata apply-plan
  contract. Evidence:
  - `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
  - `cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check -- crates/nako-core/src/automation.rs crates/nako-api/src/admin/automation.rs crates/nako-server/src/app/automation.rs crates/nako-server/src/app/tests/automation.rs crates/nako-server/src/http/admin.rs crates/nako-server/src/http/tests/mod.rs crates/nako-server/src/http/tests/system.rs`
  Broader workspace/package gates were not run because this slice changes only
  the Generated Artifact/Admin apply-plan contract and is covered by focused
  API, app, HTTP, formatting, JSON, and diff gates.
- `GAMA-030` (verified 2026-05-30): Added host-owned app-layer apply execution
  for accepted metadata Generated Artifacts. Evidence:
  - `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
  - `cargo nextest run -p nako-db metadata_application --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`
  - `git diff --check -- crates/nako-core/src/automation.rs crates/nako-core/src/media/metadata.rs crates/nako-core/src/repository/metadata.rs crates/nako-db/src/contract_tests.rs crates/nako-db/src/facade.rs crates/nako-db/src/postgres/metadata_catalog.rs crates/nako-db/src/sqlite/metadata.rs crates/nako-server/src/app/addons/metadata_write.rs crates/nako-server/src/app/automation.rs crates/nako-server/src/app/metadata_application.rs crates/nako-server/src/app/tests/automation.rs docs/workstreams/generated-artifact-metadata-authority-apply`
  Behavior proven:
  - executable plans apply unlocked fields through `MetadataApplication`;
  - all field locks protect Generated Artifact apply;
  - stale source/item/library targets reject before mutation;
  - repeated apply returns a no-op replay result when no applicable fields remain;
  - Canonical Metadata and catalog/search projection commit in one metadata
    application transaction with SQLite rollback coverage.
  PostgreSQL runtime contract coverage was not run because
  `NAKO_TEST_POSTGRES_URL` is unset in this session; the PostgreSQL
  implementation compiled through the focused gates and has a matching ignored
  contract entry.
- `GAMA-040` (verified 2026-05-30): Added durable Generated Artifact metadata
  apply outcomes and explicit idempotency-key replay. Evidence:
  - `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
  - `cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome --no-fail-fast`
  - `cargo nextest run -p nako-db postgres_metadata_catalog_contract_generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic --run-ignored ignored-only --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`
  - `git diff --check`
  Behavior proven:
  - successful apply persists an `applied` outcome with the redacted plan and
    atomically commits Canonical Metadata plus catalog/search projection;
  - repeated use of the same idempotency key replays the stored outcome;
  - a later distinct key after all applicable fields are already applied
    persists a durable `noop` outcome;
  - failed non-executable plans persist a `failed` outcome without mutating the
    original item;
  - SQLite and PostgreSQL contracts cover duplicate-key rejection and rollback
    when metadata application persistence fails.
  The PostgreSQL contract ran against a temporary local PostgreSQL cluster under
  `target/postgres-contract-gama040`, which was stopped and cleaned after the
  run.
- `GAMA-050` (verified 2026-05-30): Exposed the final Admin Generated Artifact
  metadata apply route and synchronized generated Admin TypeScript contracts.
  Evidence:
  - `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server admin_generated_artifact_metadata_apply --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`
  - `git diff --check -- docs/workstreams/generated-artifact-metadata-authority-apply docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`
  - `git diff --check`
  Behavior proven:
  - final apply route is
    `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply`;
  - request body is `AdminGeneratedArtifactMetadataApplyRequest` with explicit
    `idempotency_key`;
  - response is redacted `AdminGeneratedArtifactMetadataApplyResponse` with
    outcome id, apply status, replay flag, safe applied source, and field-level
    redacted plan summaries;
  - Admin auth rejects missing credentials before route execution;
  - successful apply mutates Canonical Metadata through the existing
    MetadataApplication path and returns no raw prompt, payload, Source Locator,
    path, fingerprint, explanation, or secret values;
  - repeating the same idempotency key replays the same durable outcome;
  - invalid idempotency keys map to `400 invalid_input` and missing artifacts
    map to `404 not_found` without leaking sensitive body content;
  - generated Admin TypeScript contracts in `apps/admin-web` and `web` match
    the Rust generator and keep Admin routes out of the Public Client SDK.
  Web workflow gates were not run because `GAMA-060` is out of scope for this
  task.

  Planner result intake on 2026-05-31 replayed this slice onto the current
  `main` baseline and fixed the generated Admin route table length after the
  final apply route was added. Fresh intake evidence:
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
  - `cargo nextest run -p nako-server admin_generated_artifact_metadata_apply --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json`
  - `git diff --cached --check`
  - `git diff --check`

## Required Final Evidence

Before closeout, record:

- exact Admin route method/path/body/response for apply plan and apply;
- no-mutation evidence for read-only apply plan;
- lock-respecting mutation evidence for apply;
- stale-target rejection before mutation;
- idempotent replay behavior;
- redaction assertions for payload, prompt, locators, paths, and secrets;
- SQLite/PostgreSQL parity if persistence changes;
- Web screenshots and tests if `GAMA-060` ships in this lane.
