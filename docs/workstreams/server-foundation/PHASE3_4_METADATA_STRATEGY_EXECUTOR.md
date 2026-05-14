# Phase 3.4: Metadata Strategy Executor

## Goal

Move metadata refresh execution from server-side provider branching into a
provider registry and strategy executor owned by `taru-metadata`.

The executor receives an item, an effective `MetadataProfile`, and a provider
registry. It tries providers in profile order and records each attempt so a
refresh job can explain whether a provider succeeded, was disabled, was
unavailable, was not implemented, returned no match, or failed.

## Architecture

- `taru-server` resolves runtime configuration, credentials, library profile,
  and job state.
- `taru-metadata` owns provider execution policy, fallback, merge behavior, and
  attempt summaries.
- Provider credentials never enter job inputs or summaries.
- Missing provider implementations are normal strategy outcomes, not hard
  server errors.
- Repository failures remain fatal and stop fallback because continuing could
  hide persistence corruption.

## Provider Attempt Semantics

Provider attempts are recorded as:

- `succeeded`: provider matched, fetched, merged, and cached raw response.
- `skipped_disabled`: provider exists in profile but is disabled in runtime
  config.
- `skipped_unavailable`: provider is configured but cannot be built, for
  example missing credentials.
- `not_implemented`: provider appears in the profile but has no registered
  implementation.
- `no_match`: provider executed but returned no candidate.
- `failed`: provider returned a recoverable provider-level error.

The first successful provider short-circuits the strategy. If no provider
succeeds, the job fails with an exhaustion error summarizing all attempts.

## Merge Rules

The executor keeps the M3.1-M3.3 merge contract:

- Locked fields are preserved.
- `missing_only` fills empty fields only.
- `full_refresh` replaces unlocked fields.
- Raw provider responses are cached for successful fetches.

## Current Scope

Implemented in this phase:

- Provider registry for available, disabled, and unavailable providers.
- Strategy executor for profile-ordered fallback.
- Server integration that registers TMDB when available and records TMDB config
  problems as strategy attempts.
- Refresh summaries with attempted providers and selected provider.

Out of scope:

- Douban and Bangumi provider implementations.
- Series, season, and episode fetch support.
- Local NFO import/export jobs.
- Item-level profile overrides.

## Validation

Required test coverage:

- Bangumi not implemented falls back to TMDB.
- Disabled provider is skipped.
- All providers failing produces a failed refresh.
- First provider success does not call later providers.
- Locked fields remain preserved during strategy refresh.

Required commands:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```
