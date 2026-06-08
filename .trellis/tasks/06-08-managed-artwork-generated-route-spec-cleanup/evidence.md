# Evidence

## Implementation

- Updated `.trellis/spec/nako-api/backend/quality-guidelines.md` so the
  Managed Artwork ingest requeue scenario no longer says `process-next` must
  remain excluded.
- Reworded the Bad case to forbid using requeue as a hidden process-next
  executor instead of forbidding the already-generated process-next contract.

## Verification

- `rg -n "remain explicitly excluded|process-next.*excluded|generating the `process-next`" .trellis/spec/nako-api/backend/quality-guidelines.md`
  returned no stale process-next exclusion language.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-managed-artwork-generated-route-spec-cleanup`
  passed.
- `git diff --check` passed.
