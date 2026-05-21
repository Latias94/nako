# 2026-05-21 — Metadata Breadth Closeout Re-Score

`metadata-provider-breadth` closed the first post-RPD execution lane. The
shipped boundary makes provider capabilities, candidate confidence, ambiguous
refresh, and cross-provider conflicts explicit without schema churn or
canonical metadata mutation on unsafe matches.

Re-score outcome:

- Next mainline lane: `nfo-link-authority`.
- Parallel sidecar candidate: `playback-transcode-ops-hardening`.
- Deferred until local authority: `managed-import-staging`.
- Deferred until stronger product surfaces: network access, AI-assisted library
  ops, and addon runtime/distribution.

The important architectural point is that downloads/import should not start as
a generic downloader. The correct next shape is Taru-owned managed import
staging, after NFO/link authority defines dry-run, backup, rollback, and local
file mutation rules.
