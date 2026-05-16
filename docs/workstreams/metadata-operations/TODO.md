# Metadata Operations TODO

## Implemented In M13

- [x] Add `metadata_maintenance` job kind.
- [x] Add HTTP enqueue route for metadata maintenance jobs.
- [x] Support library and explicit item-set scopes.
- [x] Support provider, profile, language, refresh mode, kind, and force overrides.
- [x] Summarize attempted, succeeded, failed, no-match, rate-limited, and
  skipped items.
- [x] Filter metadata attempts by provider and status.
- [x] Filter raw cache diagnostics by provider.
- [x] Add raw cache retention config and cleanup route.
- [x] Expose process-local provider runtime health.
- [x] Reuse the server metadata provider registry across refreshes.

## Follow-Up

- [x] Add a dry-run mode that lists the items and effective profile without
  touching providers.
- [x] Add scheduled maintenance policies.
- [x] Add startup/background raw cache cleanup.
- [x] Add provider circuit breaker backoff.
- [x] Remove the legacy `metadata.tmdb` configuration path.
- [x] Move provider construction and secret resolution behind a focused
      metadata runtime module.
- [x] Reject duplicate configured network providers at startup.
- [ ] Add multi-process provider health and raw cleanup coordination if Taru
  supports shared-database multi-instance deployment.
- [ ] Add total counts when diagnostics pagination needs them.
