# nako-library Backend Development Guidelines

These specs describe library scan, source ingestion, probe orchestration, local
inference, and ingestion failure handling in `crates/nako-library`.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding scan,
  probe, ingestion, local inference, or summary modules.
- Read [Database Guidelines](./database-guidelines.md) before changing
  repository interactions or persisted scan/source/probe behavior.
- Read [Error Handling](./error-handling.md) before changing scan/probe failure
  classification or ingestion failure persistence.
- Read [Quality Guidelines](./quality-guidelines.md) before changing source
  identity, tombstone, stale-cache, local inference, or bounded probe behavior.
- Read [Logging Guidelines](./logging-guidelines.md) before adding library
  diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Scan, ingestion, probe, local inference, failure, summary modules | Filled from code and architecture docs |
| [Database Guidelines](./database-guidelines.md) | Repository-trait workflow and bounded persistence loops | Filled from code |
| [Error Handling](./error-handling.md) | Ingestion failure classes, retryability, scan/probe failure records | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Source identity, tombstones, stale cache, local inference, probe gates | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Redaction-safe scan/probe diagnostics | Filled from code |

## Authority / Evidence

- `docs/architecture/LIBRARY_PIPELINE.md`
- ADR 0012: durable scan state and source tombstones.
- ADR 0016: remote storage and VFS cache boundary.
- `crates/nako-library/src/scan.rs`
- `crates/nako-library/src/ingestion.rs`
- `crates/nako-library/src/probe.rs`
- `crates/nako-library/src/local_inference/*`
