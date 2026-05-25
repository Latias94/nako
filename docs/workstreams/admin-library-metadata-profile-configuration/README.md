# Admin Library Metadata Profile Configuration

Status: Completed
Last updated: 2026-05-25

This workstream productizes Media Library metadata acquisition settings by
adding an Admin API path for reading and updating each library's effective
`MetadataProfile`.

The lower-level pipeline already exists: scans derive a
`MetadataScanAcquisitionPlan`, run configured NFO Import, can enqueue Addon Bulk
Metadata Scrape, and can request explicit Addon metadata writeback when enabled.
This lane makes that behavior operator-configurable without editing TOML.

Closed on 2026-05-25 after adding the Admin read/update API, generated Admin
Web contract entries, focused HTTP tests, and fresh verification evidence.

## Goals

- Expose a safe Admin read model for a Media Library's Metadata Profile.
- Let an administrator update scan-time metadata policy and related local/addon
  profile settings through Admin API.
- Persist changes through the existing library options repository path.
- Prove a scan uses the updated profile on the next run.

## Non-Goals

- Admin Web UI controls.
- New schema migrations.
- New metadata providers, embedded readers, or artwork acquisition behavior.
- Full NAS root validation.
- Addon capability negotiation or health policy changes.

## Related Work

- `docs/workstreams/library-metadata-scan-policy`
- `docs/workstreams/scan-addon-bulk-metadata-scrape`
- `docs/workstreams/metadata-acquisition-pipeline`
- `docs/workstreams/addon-protected-writes`
