# Directory Structure

`nako-library` owns the library intake workflow after files are visible through
VFS. It does not own storage backend implementation, media probe extraction, DB
adapters, or canonical metadata provider refresh.

## Current Layout

```text
crates/nako-library/src/
├── lib.rs                     # public exports and workflow tests
├── scan.rs                    # VFS library scanner and discovered sources
├── ingestion.rs               # repository-backed scan/source commit workflow
├── ingestion/source_commit.rs # source observation planning
├── probe.rs                   # media probe orchestration
├── local_inference/           # parser-backed provisional hierarchy planning
├── failure.rs                 # ingestion failure classification helpers
├── index.rs                   # index service orchestration
└── summary.rs                 # scan/probe/index summaries
```

## Module Rules

- Keep VFS traversal in `scan.rs` through `StorageBackend`.
- Keep database-facing writes in `ingestion.rs` through `nako-core` repository
  traits.
- Keep media technical fact extraction orchestration in `probe.rs` through the
  `nako-media-probe` trait.
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
- `ingestion.rs`: scan snapshot, source observation, failure, and tombstone
  persistence through traits.
- `probe.rs`: bounded concurrent probe workflow.
- `local_inference/plan.rs`: local name parsing to provisional item/source
  records.
