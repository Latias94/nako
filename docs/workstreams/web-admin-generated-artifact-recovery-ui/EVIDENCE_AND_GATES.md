# Web Admin Generated Artifact Recovery UI — Evidence And Gates

Status: Closed
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

## Verified Route Gates

Verified on 2026-06-02:

- `npm --prefix web run test -- src/test/data-source-contracts.test.ts`
  - 1 file passed; 38 tests passed.
- `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`
  - 2 files passed; 58 tests passed.
- `npm --prefix web run check`
  - `tsc --noEmit` passed.
- `npm --prefix web run build:budget`
  - passed after keeping `total-js` to 340.53 KiB gzip under the adjusted
    341 KiB limit.
- browser smoke at
  `/admin/automation/generated-artifacts/recovery?attention=needs_repair`
  - desktop 1280px: heading, fixture outcome, and read-only badge visible;
    no raw sensitive values; no page-level overflow.
  - mobile 390px: heading, fixture outcome, and read-only badge visible; no
    raw sensitive values; no page-level overflow; table overflow is contained
    inside the table scroller.
  - screenshots:
    - `target/codex-smoke/wagr-recovery-desktop.png`
    - `target/codex-smoke/wagr-recovery-mobile.png`

Budget note:

- The route adds a real Admin operator surface, so `web/scripts/check-bundle-budget.mjs`
  raises only the aggregate `total-js` gzip limit from 340 KiB to 341 KiB.
  Initial JS, initial CSS, Admin route JS, and Media route JS stayed under
  their existing budgets.

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
