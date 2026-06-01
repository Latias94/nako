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

## Open Decisions For GAARA-020

- Does repair need a backend mutation wrapper, or is the existing single/bulk
  apply mutation sufficient when launched from recovery context?
- If a wrapper is needed, which recovery-context guards does it add that the
  existing apply route cannot provide?
- Should the first Web action remain "prepare repair" instead of "execute
  repair" until backend guard evidence exists?

## Notes

- The repair action must not replay a stale plan.
- Existing apply semantics are the preferred execution kernel.
- Treat raw artifact payloads, prompts, Source Locators, paths, tokens,
  secrets, provider responses, and idempotency keys as forbidden UI/API data.
