# Evidence

## 2026-06-05

- Opened after `docs(architecture): reconcile source fingerprint scheduling
  maps` when a follow-on search found stale source fingerprint scheduling /
  operator diagnostics wording still present in `STORAGE_VFS.md`.
- Updated `docs/architecture/STORAGE_VFS.md` remote-storage follow-on wording
  from scheduling/operator diagnostics to queue/operator integration.

## Verification

- `rg -n "source fingerprint hash scheduling / operator diagnostics|fingerprint hash scheduling / operator diagnostics|source-fingerprint-hash-scheduling-and-diagnostics" docs/architecture/STORAGE_VFS.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md -S` — no matches
- `rg -n "fingerprint hash queue/operator integration|source-fingerprint-hash-queue-and-operator-integration" docs/architecture/STORAGE_VFS.md docs/architecture/LANES.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LIBRARY_PIPELINE.md -S`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-source-fingerprint-storage-map-follow-on-wording-cleanup`
