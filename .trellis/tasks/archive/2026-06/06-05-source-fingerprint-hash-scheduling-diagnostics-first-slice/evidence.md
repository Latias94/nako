# Evidence

## 2026-06-05

- Task opened from `docs/architecture/STORAGE_VFS.md`
  `proposed:source-fingerprint-hash-scheduling-and-diagnostics` after the
  source fingerprint hash execution kernel shipped.
- Scope selected as a `nako-library` planning seam: make advisory hash
  escalation decisions actionable and diagnostic without adding DB, API,
  queueing, execution, or source merge behavior.
- Implemented `nako-library::source_hash` scheduling diagnostics:
  advisory `SourceFingerprintEscalationDecision` + opt-in policy now map to a
  redaction-safe diagnostic and optional in-process `SourceFingerprintHashRequest`.
- Updated library spec and storage/library architecture maps to distinguish the
  shipped planning seam from future durable queues, Admin/Public API exposure,
  persistence, execution integration, or automatic source reconciliation.

## Verification

- `cargo fmt --all`
- `cargo nextest run -p nako-library source_hash --no-fail-fast` — 10 passed
- `cargo check -p nako-library --tests`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-05-source-fingerprint-hash-scheduling-diagnostics-first-slice`
