# Generated Artifact Provider Mapping Breadth - Closeout

Status: Closed
Closed: 2026-06-02
Task: GAPM-060

## Shipped

Nako now supports Provider Subject and Provider Mapping breadth inside the
guarded Generated Artifact Metadata Authority workflow:

- review acceptance remains a staging action and does not mutate Canonical
  Metadata or Provider Mappings;
- one-artifact metadata apply-plan exposes redacted Provider Mapping proposal
  entries, reasons, and apply/skip/noop counters next to Canonical Metadata
  field plans;
- final one-artifact metadata apply can upsert Provider Subjects and accepted
  Provider Mappings idempotently through host-owned repositories and the same
  durable outcome transaction used by Canonical Metadata apply;
- bulk metadata apply reuses the one-artifact execution path, surfaces mapping
  counters through bulk summaries and batch snapshots, and does not add a
  second Provider Mapping executor;
- generated Admin API contracts and `web/` Admin read models preserve mapping
  proposal/result facts without exposing raw payloads, prompts, provider raw
  responses, Source Locators, host paths, tokens, secrets, or idempotency
  keys;
- Web Admin renders Provider Mapping plan/result details in the single-artifact
  Metadata Authority apply route and aggregate/per-item mapping facts in the
  bulk apply route.

## Final Evidence

Fresh backend evidence already recorded on 2026-06-01:

```bash
cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server generated_artifact_metadata_apply_plan --no-fail-fast
cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast
cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast
cargo nextest run -p nako-db provider_mapping generated_artifact_metadata_apply --no-fail-fast
cargo fmt --all -- --check
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
```

Results recorded in `EVIDENCE_AND_GATES.md`:

- read-only Provider Mapping plan support passed focused API/server gates;
- durable one-artifact Provider Subject / Provider Mapping apply passed
  server/db/PostgreSQL parity gates;
- bulk summary and batch reconciliation passed API/server/db gates;
- PostgreSQL parity was proven while repository transaction behavior changed and
  does not need a second rerun for the Web-only closeout slice.

Fresh Web verification on 2026-06-02:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Results:

- data-source contracts: 37/37 passed;
- route and route-state contracts: 55/55 passed;
- TypeScript check: passed;
- bundle budget: passed with `admin-route-js` 207.38 raw KiB / 43.76 gzip KiB
  under the 260/65 KiB budget and `total-js` 1146.68 raw KiB / 338.17 gzip
  KiB under the 1250/340 KiB budget.

Browser smoke on 2026-06-02 against `http://127.0.0.1:3000`:

- desktop `1440x900`: Provider Mapping plan/result facts rendered in the
  single-artifact Metadata Authority apply route and the bulk apply workflow
  with no secret leakage and no horizontal overflow;
- mobile `390x844`: Provider Mapping plan facts rendered in the
  single-artifact Metadata Authority apply route with no secret leakage and no
  horizontal overflow.

Console output contained only the existing Vite Fast Refresh / React DevTools
hook shim warning.

Fresh docs verification on 2026-06-02:

```bash
python -m json.tool docs/workstreams/generated-artifact-provider-mapping-breadth/WORKSTREAM.json
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/generated-artifact-provider-mapping-breadth/TASKS.jsonl",
    "docs/workstreams/generated-artifact-provider-mapping-breadth/CAMPAIGNS.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
git diff --check -- docs/workstreams/generated-artifact-provider-mapping-breadth docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md
git diff --check
```

Results:

- `WORKSTREAM.json` validation: passed;
- `TASKS.jsonl` and `CAMPAIGNS.jsonl` validation: passed;
- targeted and repository-wide diff checks: passed; Git emitted only LF/CRLF
  normalization warnings.

## Review Result

No blocking workstream compliance findings remain:

- the target state in `DESIGN.md` is satisfied across read-only planning,
  durable one-artifact apply, bulk/Admin reconciliation, and Web Admin
  rendering;
- `TODO.md`, `TASKS.jsonl`, `WORKSTREAM.json`, and architecture routing can now
  move from active execution to closed evidence;
- Provider Mapping breadth remains explicitly separate from provider search,
  provider depth, hierarchy repair, and operations repair tooling.

No blocking code-quality findings remain:

- review acceptance is still staging-only;
- one-artifact and bulk apply preserve target freshness checks and idempotent
  outcomes;
- Web Admin remains honest about fixture/fallback mode and does not claim live
  mutation when the plan falls back to fixture data;
- sensitive provider/addon/source data remains redacted through DTOs, read
  models, tests, and browser smoke.

## Follow-Ons

- `proposed:generated-artifact-apply-operations-repair`: operator search,
  failed/noop/stale repair, replay diagnostics, and recovery tooling after
  partial or stale apply outcomes.
- `proposed:provider-identity-mapping-breadth`: deeper provider identity
  precision such as broader subject-kind coverage, conflict diagnostics,
  provider-depth ergonomics, and stronger operator review semantics beyond the
  current guarded breadth slice.
- `proposed:admin-settings-api-backed-restoration`: restore placeholder Admin
  settings panels as API-backed surfaces without hiding route-budget growth
  inside Generated Artifact lanes.

## Residual Risks

- Web Admin now shows the first Provider Mapping breadth slice, but there is
  still no dedicated repair/audit workflow for stale, skipped, noop, or failed
  apply outcomes across large queues.
- Provider Mapping precision remains intentionally shallow: the lane proves
  host-owned breadth, not the full provider-depth and conflict-diagnostics
  product.
- The browser smoke relied on the local Vite server and fixture/live test data;
  future release-oriented smoke should keep verifying this route as broader
  Admin surfaces evolve.
