# Quality Guidelines

Library workflow changes must preserve deterministic, bounded intake behavior.

## Required Patterns

- Keep scan traversal bounded by `LibraryScannerOptions::max_depth`.
- Keep probe concurrency bounded by `LibraryProbeOptions::max_concurrent_probes`.
- Sort discovered media sources and returned failures deterministically.
- Propagate stale-cache evidence from VFS metadata/listing into summaries.
- Use source fingerprint evidence as duplicate evidence, not source identity.
- Persist local inference evidence so provisional hierarchy decisions are
  explainable.

## Forbidden Patterns

- Do not collapse Media Sources across locators automatically.
- Do not let local inference overwrite confirmed canonical metadata during
  rescan.
- Do not turn recursive scan into unbounded filesystem or remote listing work.
- Do not hide storage/probe failures as generic skipped counts.
- Do not bypass VFS for local filesystem paths.

## Tests Required

- Scan tests for supported extension filtering, recursion, ordering, stale cache
  propagation, and source fingerprint evidence.
- Ingestion tests for insert/update/tombstone disposition.
- Probe tests for skip/force/failure persistence and bounded concurrency.
- Local inference tests for provisional hierarchy and evidence.

## Gate Selection

- Focused library:
  `cargo nextest run -p nako-library <filter> --no-fail-fast`
- Cross-crate intake:
  `cargo check -p nako-library -p nako-vfs -p nako-db --tests`

## Review Checklist

- Is the workflow bounded?
- Are source identity and duplicate evidence kept separate?
- Are failures durable and redaction-safe?
- Does confirmed canonical state remain protected?
