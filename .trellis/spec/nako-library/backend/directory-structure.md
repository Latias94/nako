# Directory Structure

`nako-library` owns the library intake workflow after files are visible through
VFS. It does not own storage backend implementation, media probe extraction, DB
adapters, or canonical metadata provider refresh.

## Current Layout

```text
crates/nako-library/src/
├── lib.rs                     # public exports and workflow tests
├── intake.rs                  # stable candidate intake evidence primitives
├── scan.rs                    # VFS library scanner and discovered sources
├── ingestion.rs               # repository-backed scan/source commit workflow
├── ingestion/source_commit.rs # source observation planning
├── probe.rs                   # media probe orchestration
├── source_hash.rs             # source fingerprint hash job input, scheduling diagnostics, job summary, and execution kernel
├── local_inference/           # parser-backed provisional hierarchy planning
├── failure.rs                 # ingestion failure classification helpers
├── index.rs                   # index service orchestration
└── summary.rs                 # scan/probe/index summaries
```

## Module Rules

- Keep VFS traversal in `scan.rs` through `StorageBackend`.
- Keep stable-candidate intake evidence helpers in `intake.rs`; this module may
  decide whether repeated watch observations are stable enough for intake but
  must not own runtime scheduling, filesystem watcher daemons, or storage
  admission.
- Keep database-facing writes in `ingestion.rs` through `nako-core` repository
  traits.
- Keep media technical fact extraction orchestration in `probe.rs` through the
  `nako-media-probe` trait.
- Keep source fingerprint hash durable input, scheduling diagnostics, job
  summary, and execution in `source_hash.rs`. The durable job input may carry
  only Media Library ID, Media Source ID, source scheme, and hash mode. The
  planner may turn advisory escalation decisions into optional in-process hash
  requests with redaction-safe diagnostics. The job summary may project
  execution reports into mode, evidence kind, confidence, stale state, and bytes
  hashed, but must not carry raw fingerprint/hash material. The execution
  kernel may compute evidence through `StorageBackend` `read_range` /
  `stream_range`. None of these paths may schedule scans, write repositories,
  add API fields, or merge Media Sources.
- Keep path/name-derived provisional hierarchy in `local_inference/`.
- Keep user/operator summaries in `summary.rs`; do not leak raw backend errors
  into summaries without classification.
- Re-export public workflow records from `lib.rs`.

## Forbidden Placement

- Do not import `sqlx` or `nako-db` outside tests. Production code uses
  repository traits.
- Do not implement storage backend behavior here. Use `nako-vfs`.
- Do not turn local inference into canonical provider metadata. It produces
  provisional hierarchy and evidence.
- Do not merge two Media Sources just because they share a fingerprint. Use
  Source Duplicate Relationship evidence.

## Examples

- `scan.rs`: recursive VFS scan with media extension filtering and stale-cache
  propagation.
- `intake.rs`: repeated watch observations reduced to stable-candidate evidence
  before any future scan/probe integration.
- `ingestion.rs`: scan snapshot, source observation, failure, and tombstone
  persistence through traits.
- `probe.rs`: bounded concurrent probe workflow.
- `source_hash.rs`: source fingerprint hash job input contract, advisory
  escalation decision to optional hash request planning, redaction-safe durable
  job summary projection, plus bounded partial hash and streaming full hash
  evidence execution for future source fingerprint escalation workflows.
- `local_inference/plan.rs`: local name parsing to provisional item/source
  records.
