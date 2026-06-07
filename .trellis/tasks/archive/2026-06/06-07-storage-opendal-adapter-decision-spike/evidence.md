# Evidence

## 2026-06-07 OpenDAL Adapter Decision Spike

Decision:

- Defer `opendal` as a production dependency for now.
- Keep Nako's `StorageBackend` boundary and domain-owned storage semantics.
- Revisit only as a narrow adapter spike behind Nako policy if future backend
  breadth becomes a committed product target.

Why:

- Nako already owns `StorageUri`, Source Locator redaction, Source Fingerprint
  evidence, storage health, cache repair authority, deterministic staging, and
  Admin-safe diagnostics.
- OpenDAL 0.57.0 is credible, but its broader capability surface would still
  need a Nako-owned adapter layer to preserve capability narrowing, redaction,
  range/stream behavior, and runtime policy.
- The current M1/M2 wave does not justify production dependency churn before a
  narrower backend target is proven.

Boundaries:

- No production dependency was added.
- No schema, API, config shape, or runtime behavior changed.
- No storage semantics were widened.

Validation:

- `git diff --check`
  - Result: passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-storage-opendal-adapter-decision-spike`
  - Result: passed.
