# Generated Artifact Apply Repair Actions — Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/generated-artifact-apply-repair-actions/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/generated-artifact-apply-repair-actions/TASKS.jsonl",
    "docs/workstreams/generated-artifact-apply-repair-actions/CAMPAIGNS.jsonl",
    "docs/workstreams/generated-artifact-apply-repair-actions/CONTEXT.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check -- docs/workstreams/generated-artifact-apply-repair-actions docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md
git diff --check
```

## Expected Repair Gates

- `cargo nextest run -p nako-server generated_artifact_metadata_apply_replays_same_idempotency_key_from_durable_outcome generated_artifact_metadata_apply_rejects_stale_target_before_mutation --no-fail-fast`
- focused `cargo nextest run` for any new Admin API/server/db repair contract
- `cargo check -p nako-server --tests`
- Admin TypeScript contract generation/checks if DTOs change
- `npm --prefix web run test -- src/test/data-source-contracts.test.ts` when Web data sources change
- route/state tests when Web recovery repair UX changes
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke when Web mutation UX changes
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/generated-artifact-metadata-authority-apply/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-apply-operations-repair/CLOSEOUT.md`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-apply-repair-actions/DESIGN.md`
- `docs/workstreams/generated-artifact-apply-repair-actions/TODO.md`
- `docs/workstreams/generated-artifact-apply-repair-actions/TASKS.jsonl`
- `docs/workstreams/generated-artifact-apply-repair-actions/CAMPAIGNS.jsonl`

## Recon Evidence

Read-only explorer audit on 2026-06-02:

- Existing `AutomationService::apply_generated_artifact_metadata` is the
  correct single-artifact execution kernel because it replans, checks target
  freshness, persists idempotent outcomes, delegates to MetadataApplication,
  and records durable audit facts.
- Existing bulk execution already delegates per item to the same single-apply
  path.
- A repair action does not need a new metadata mutation core.
- If product UX stays "recovery row -> current apply plan -> operator
  confirmation", no backend mutation is needed.
- If product UX needs one-click row repair, add only a narrow Admin wrapper
  that validates recovery context and then delegates to existing apply or bulk
  apply behavior.

GAARA-020 should therefore prove one of two paths:

1. Web-only repair preparation over existing apply routes.
2. A narrow recovery-context wrapper with no duplicated executor.

## GAARA-020 Seam Decision

Decision on 2026-06-02: use Web-only repair preparation over the existing
Metadata Authority apply routes. Do not add a backend recovery mutation wrapper
for the current product shape.

Why:

- `AutomationService::apply_generated_artifact_metadata` already owns the
  mutation boundary: it normalizes a caller-provided idempotency key, replays
  durable outcomes for the same key, replans from the current artifact state,
  rejects stale targets before mutation, commits failures as outcomes, and
  persists successful `MetadataApplication` and Provider Mapping results.
- Bulk apply already delegates each item to the same single-artifact apply path,
  so adding a second recovery executor would duplicate behavior and increase
  audit risk.
- The Admin recovery queue is read-only and redaction-safe. Its row action
  passes only the artifact id into the existing Metadata Authority apply route.
- The Web apply route fetches the current apply plan before confirmation,
  disables fixture-mode mutation, and generates a fresh Web idempotency key for
  the live apply request. It does not reuse recovery-row plan snapshots or
  recovery-row idempotency data.

Fresh gates:

```bash
cargo nextest run -p nako-server generated_artifact_metadata_apply_replays_same_idempotency_key_from_durable_outcome generated_artifact_metadata_apply_rejects_stale_target_before_mutation --no-fail-fast
```

Result: passed, 2 tests run.

```bash
npm --prefix web run test -- src/test/route-state-contracts.test.tsx
```

Result: passed, 33 tests run. The new route-state contract covers recovery row
navigation into the current apply plan, no mutation before confirmation, a new
Web-generated idempotency key on confirmation, and no display or reuse of the
unsafe recovery idempotency field embedded in the fixture response.

```bash
npm --prefix web run check
```

Result: passed.

```bash
python -m json.tool docs/workstreams/generated-artifact-apply-repair-actions/WORKSTREAM.json
```

Result: passed.

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/generated-artifact-apply-repair-actions/TASKS.jsonl",
    "docs/workstreams/generated-artifact-apply-repair-actions/CAMPAIGNS.jsonl",
    "docs/workstreams/generated-artifact-apply-repair-actions/CONTEXT.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

Result: passed.

```bash
git diff --check
```

Result: passed with Windows line-ending warnings only.

Follow-on decision:

- `GAARA-030` is deferred. Run it only if product chooses a one-click
  recovery-context wrapper that adds real guards beyond the current apply route.
- `GAARA-040` is deferred. Run it only for explicit UX copy or confirmation
  polish beyond the current recovery-row-to-apply-plan flow.
- `GAARA-050` should close the lane or split those deferred choices as
  separate follow-ons.

## Open Decisions For GAARA-020

- Resolved: repair does not need a backend mutation wrapper for the current
  recovery-row-to-apply-plan product shape.
- Resolved: the existing single/bulk apply mutation is sufficient when launched
  from recovery context because Web re-plans before confirmation and uses a new
  idempotency key.
- Resolved: the first Web action remains preparation, not one-click execution.

## Notes

- The repair action must not replay a stale plan.
- Existing apply semantics are the preferred execution kernel.
- Treat raw artifact payloads, prompts, Source Locators, paths, tokens,
  secrets, provider responses, and idempotency keys as forbidden UI/API data.
