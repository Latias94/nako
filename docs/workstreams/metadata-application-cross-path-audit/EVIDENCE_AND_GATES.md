# Metadata Application Cross-Path Audit - Evidence and Gates

Status: Complete
Last updated: 2026-05-26

## Files Reviewed

- `crates/nako-metadata/src/provider_attempt.rs`
- `crates/nako-metadata/src/confirmation.rs`
- `crates/nako-core/src/media/merge.rs`
- `crates/nako-server/src/app/metadata_application.rs`

## Evidence

Provider refresh and hierarchy confirmation already share the pure core merge
policy. Their surrounding responsibilities differ:

- provider refresh resolves provider keys, fetches provider payloads, stores raw
  responses, and returns provider subjects;
- hierarchy confirmation validates item structure and provider mappings;
- server metadata application writes item metadata, field locks, apply reports,
  and catalog projection.

Those responsibilities are not the same Module boundary.

## Gates

No code changed for this audit lane. The scan continuation lane ran the focused
server gates documented in
`docs/workstreams/scan-addon-bulk-continuation/EVIDENCE_AND_GATES.md`.
