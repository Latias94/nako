# Generated Artifact Apply Operations Repair — Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/generated-artifact-apply-operations-repair/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/generated-artifact-apply-operations-repair/TASKS.jsonl",
    "docs/workstreams/generated-artifact-apply-operations-repair/CAMPAIGNS.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check -- docs/workstreams/generated-artifact-apply-operations-repair docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md
git diff --check
```

## Expected Iteration Gates

Choose the smallest focused gates that prove the read path or repair surface
being added. Candidate gates likely include:

- `cargo nextest run -p nako-api generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast`
- `cargo nextest run -p nako-server generated_artifact_bulk_metadata_apply --no-fail-fast`
- focused Web/Admin contract or route tests under `web/src/test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget` when Admin route code changes
- `cargo fmt --all -- --check`

Fresh verification should widen only when the chosen slice changes public/Admin
contract shape, persistence semantics, or route code.

## Evidence Anchors

- `docs/workstreams/generated-artifact-apply-operations-repair/DESIGN.md`
- `docs/workstreams/generated-artifact-apply-operations-repair/TODO.md`
- `docs/workstreams/generated-artifact-apply-operations-repair/TASKS.jsonl`
- `docs/workstreams/generated-artifact-apply-operations-repair/CAMPAIGNS.jsonl`
- `docs/workstreams/generated-artifact-apply-operations-repair/MILESTONES.md`
- `docs/workstreams/generated-artifact-metadata-authority-apply/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md`

## Notes

- `GAOR-010` proves the lane is grounded in existing architecture and closeout
  evidence instead of chat memory.
- `GAOR-020` should record what existing durable outcome and batch records can
  already support before any DTO or route is changed.
- Do not accept a repair mutation without explicit evidence that it reuses the
  current Metadata Authority apply semantics and preserves idempotency,
  freshness checks, and redaction.

## GAOR-020 Audit Evidence

Audit date: 2026-06-02

Code surfaces inspected:

- `crates/nako-core/src/automation.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/http/admin.rs`
- `web/src/api/admin/read-models-data-source.ts`
- `web/src/api/admin/mutations-data-source.ts`

Findings:

- durable one-artifact outcome persistence already exists with `Applied`,
  `Noop`, and `Failed` statuses plus plan snapshots and safe error fields;
- durable bulk batch persistence already exists with per-item `Pending`,
  `Skipped`, `Applied`, `Noop`, `Stale`, and `Failed` statuses plus optional
  `outcome_id` links;
- Admin/API/Web currently expose bulk batch create/get surfaces, but no direct
  Admin outcome list/detail route;
- the smallest missing product seam is therefore an outcome-oriented read path
  that can optionally incorporate batch provenance later.

Recommended next slice:

- `GAOR-030` should start with a redaction-safe Admin outcome list/detail read
  surface before considering any repair mutation.

## GAOR-030 First Read Surface Evidence

Implementation date: 2026-06-02

Delivered slice:

- repository seam for `get/list_generated_artifact_metadata_apply_outcome(s)`;
- Admin DTOs and generated TypeScript contract for outcome list/detail reads;
- Admin HTTP list/detail routes and app-service read paths;
- Web Admin client/read-model support for outcome list/detail surfaces.

Focused verification:

- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-server --tests`
- `npm --prefix web run check`

Result:

- pass; the first repair-oriented outcome read path is now wired through Rust,
  generated contract, and Web data-source boundaries without exposing raw
  prompt/payload secrets.

## GAOR-030 Recovery Queue Evidence

Implementation date: 2026-06-02

Delivered slice:

- domain recovery classification for apply outcomes and bulk batch terminal
  items: `needs_repair`, `needs_review`, `replay_only`, and `resolved`;
- repository seam and SQLite/Postgres read adapters for
  `list_generated_artifact_metadata_apply_recovery_entries`;
- Admin API response, generated TypeScript contract, and HTTP route
  `/admin/v1/automation/generated-artifact-apply-recovery`;
- Web Admin client/read-model mapper and fixture for the repair-oriented queue;
- recovery classification is owned by `nako-core`, while DB adapters only map
  rows and deserialize plans.

Focused verification:

- `cargo nextest run -p nako-api admin_contract generated_artifact_metadata_apply_recovery_response_classifies_repair_state --no-fail-fast`
- `cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic --no-fail-fast`
- `cargo nextest run -p nako-server admin_generated_artifact_metadata_apply_v1_commits_and_replays_redacted_result --no-fail-fast`
- `cargo check -p nako-server --tests`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`

Review result:

- no blocking findings after moving duplicated recovery classification out of
  the SQLite/Postgres adapters and into `nako-core`;
- `GAOR-030` is accepted as the first read-only repair surface. Mutation-based
  repair actions remain intentionally out of scope.
