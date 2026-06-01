# Generated Artifact Bulk Metadata Apply - Closeout

Status: Closed
Closed: 2026-06-01
Task: GABMA-070

## Shipped

Nako now has a guarded bulk Metadata Authority apply workflow for accepted
metadata Generated Artifacts:

- Admin bulk plan is
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-plan` with
  `AdminGeneratedArtifactMetadataBulkApplyPlanRequest { artifact_ids }` and a
  redacted `AdminGeneratedArtifactMetadataBulkApplyPlanResponse`.
- Bulk plan is read-only, bounded, duplicate-aware, and exposes aggregate plus
  per-artifact safe facts without mutating Canonical Metadata.
- Admin bulk confirm is
  `POST /admin/v1/automation/generated-artifacts/metadata-apply-batches` with
  `AdminGeneratedArtifactMetadataBulkApplyRequest { artifact_ids,
  idempotency_key }`.
- Confirm persists a stable batch identity and queues durable
  `generated_artifact_metadata_bulk_apply` work instead of applying an
  unbounded selection inside the HTTP request path.
- Batch status/result is
  `GET /admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}`
  with batch/job identity, redacted plan snapshots, execution counters, and
  per-item applied/noop/stale/failed/skipped result facts.
- Execution reuses the one-artifact Metadata Authority apply path so field
  locks, stale-target checks, `MetadataApplication`, catalog/search projection
  commits, and apply outcomes remain host-owned.
- Batch replay and per-item idempotency prevent duplicate mutations.
- Web Admin supports accepted-artifact selection, redacted bulk plan display,
  live-only confirmation, status refresh, and partial-result rendering.

## Final Evidence

Fresh backend verification on 2026-06-01:

```bash
cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast
cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast
cargo nextest run -p nako-db generated_artifact_bulk_metadata_apply --no-fail-fast
cargo fmt --all -- --check
```

Results:

- `nako-api generated_artifact_metadata_apply`: 4/4 passed.
- `nako-api admin_contract`: 5/5 passed.
- `nako-server generated_artifact_metadata_apply_plan`: 4/4 passed.
- `nako-server generated_artifact_bulk_metadata_apply`: 5/5 passed.
- `nako-db generated_artifact_bulk_metadata_apply`: 1/1 SQLite contract passed.
- `cargo fmt --all -- --check`: passed.

Fresh PostgreSQL verification on 2026-06-01:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
```

Results:

- 46/46 ignored PostgreSQL contracts passed through the local PostgreSQL 17
  harness.
- The run included
  `postgres_metadata_catalog_contract_generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic`.
- A direct ignored-test command without `NAKO_TEST_POSTGRES_URL` failed fast as
  designed before the harness run; that failure proves the gate does not report
  a false green without a real PostgreSQL URL.
- The harness warned that `pg_ctl stop` returned a failure, but `127.0.0.1:55432`
  was closed afterward and `target/postgres-contract` was removed.

Fresh Web verification on 2026-06-01:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Results:

- Data-source contracts: 37/37 passed.
- Route and route-state contracts: 55/55 passed.
- TypeScript check: passed.
- Bundle budget: passed; `admin-route-js` 203.77 raw KiB / 43.27 gzip KiB,
  `total-js` 1140.00 raw KiB / 336.96 gzip KiB under the explicit 340 KiB
  gzip limit.

Browser smoke from `GABMA-060`:

- local Vite target:
  `http://127.0.0.1:3000/admin/automation/generated-artifacts`;
- desktop `1440x1000` and mobile `390x844` mocked live Admin API flows covered
  selection, plan, confirm, completed-batch status, and redaction;
- screenshots: `.tmp/gabma060-desktop.png`, `.tmp/gabma060-mobile.png`.

Final docs verification on 2026-06-01:

```bash
python -m json.tool docs/workstreams/generated-artifact-bulk-metadata-apply/WORKSTREAM.json
git diff --check -- docs/workstreams/generated-artifact-bulk-metadata-apply docs/architecture docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md
git diff --check
```

Results:

- `WORKSTREAM.json` validation: passed.
- `TASKS.jsonl` and `CAMPAIGNS.jsonl` validation: passed.
- Targeted docs diff check: passed; Git emitted only LF/CRLF normalization
  warnings.
- Repository diff check: passed; Git emitted only LF/CRLF normalization
  warnings.

## Review Result

No blocking workstream compliance findings remain:

- the target state in `DESIGN.md` is satisfied;
- all tasks in `TODO.md` and `TASKS.jsonl` are complete;
- `CAMPAIGNS.jsonl` is closed;
- Admin routes are Admin-only and generated Admin TypeScript contracts are
  synchronized;
- Public Client route inventory does not expose the bulk Admin workflow;
- architecture maps and workstream indexes no longer route active work to this
  closed lane.

No blocking code-quality findings remain:

- bulk plan performs no Canonical Metadata mutation;
- confirmed mutation is durable and job-backed;
- item execution flows through the existing one-artifact apply service;
- per-item outcomes preserve partial failure rather than hiding it;
- read models drop raw artifact JSON, prompts, provider payloads, Source
  Locators, paths, idempotency keys, tokens, and secrets;
- Web fixture/fallback mode remains read-only;
- the `total-js` gzip budget increase to 340 KiB is accepted as an explicit
  tradeoff, while per-route budgets remain unchanged and passing.

## Follow-Ons

- `proposed:generated-artifact-provider-mapping-breadth`: provider-specific
  mapping beyond the current neutral metadata suggestion shape.
- `proposed:generated-artifact-apply-operations-repair`: batch/outcome search,
  failed/noop/stale repair, replay diagnostics, and operator recovery tooling.
- `proposed:admin-settings-api-backed-restoration`: restore placeholder Admin
  settings pages as API-backed panels without hiding growth in route budgets.

## Residual Risks

- The Web workflow supports explicit accepted-artifact selection, not broad
  saved queries or automation rules; those need a separate policy lane.
- Provider-specific field breadth remains intentionally outside this lane.
- The local PostgreSQL harness produced a cleanup warning even though the
  contracts passed and the port/data directory were gone afterward. Watch for
  recurring PostgreSQL harness stop warnings before relying on it in release
  automation.
- Operations repair tooling is still required for large-library operator
  recovery after partial failures.
