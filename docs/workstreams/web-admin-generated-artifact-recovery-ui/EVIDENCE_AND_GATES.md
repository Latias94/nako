# Web Admin Generated Artifact Recovery UI — Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Lane Opening Gates

```bash
python -m json.tool docs/workstreams/web-admin-generated-artifact-recovery-ui/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/web-admin-generated-artifact-recovery-ui/TASKS.jsonl",
    "docs/workstreams/web-admin-generated-artifact-recovery-ui/CAMPAIGNS.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check -- docs/workstreams/web-admin-generated-artifact-recovery-ui docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md docs/architecture/CONTROL_PLANE.md docs/workstreams/README.md docs/GOALS.md docs/ROADMAP.md
git diff --check
```

## Expected Route Gates

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
- `npm --prefix web run check`
- browser smoke on desktop and mobile widths when route code changes
- `npm --prefix web run build:budget` if the route materially changes Admin
  bundle size

## Evidence Anchors

- `docs/workstreams/generated-artifact-apply-operations-repair/CLOSEOUT.md`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/DESIGN.md`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/TODO.md`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/TASKS.jsonl`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/CAMPAIGNS.jsonl`

## Notes

- The first route must be read-only.
- Do not add mutation controls until `proposed:generated-artifact-apply-repair-actions`
  proves idempotent reuse of Metadata Authority apply semantics.
- Treat raw artifact payloads, prompts, Source Locators, paths, tokens,
  secrets, and idempotency keys as forbidden UI data.
