# Phase 18.0: Provider Runtime Productization

## Summary

M18 makes network metadata providers use one runtime boundary for
configuration, construction, secret resolution, diagnostics, and fallback
semantics.

The old `metadata.tmdb` compatibility path was removed. TMDB, Bangumi, Douban,
and future network providers are configured only through `[[metadata.providers]]`.
This keeps provider setup symmetric and prevents TMDB from having a separate
legacy path.

## Runtime Boundary

`crates/nako-server/src/app/metadata_runtime.rs` owns:

- provider registry construction
- provider-specific config mapping
- environment secret resolution
- configured header resolution
- runtime diagnostic DTO mapping
- duplicate provider config validation

`crates/nako-server/src/app/metadata.rs` remains responsible for metadata
refresh, maintenance jobs, raw cache cleanup, provider attempts, and event
recording.

## Configuration

Network provider configuration uses the shared array shape:

```toml
[[metadata.providers]]
provider = "tmdb"
enabled = true
token_env = "TMDB_READ_ACCESS_TOKEN"
language = "en-US"

[[metadata.providers]]
provider = "bangumi"
enabled = true
token_env = "BANGUMI_TOKEN"

[[metadata.providers]]
provider = "douban"
enabled = true
api_key_env = "DOUBAN_API_KEY"
```

Global runtime defaults still live under `[metadata.runtime]`. Individual
providers can override runtime settings with `runtime = { ... }`.

## Behavior

- Empty `metadata.providers` means no network provider is configured.
- A library profile can still request TMDB, Bangumi, or Douban. If that provider
  is not configured, the strategy records a `not_implemented` provider attempt.
- Disabled providers are represented explicitly through `enabled = false`.
- Missing or blank environment secrets make the configured provider
  `unavailable`, which allows profile fallback to the next provider.
- Duplicate provider entries are rejected during startup instead of silently
  overwriting registry state.

## Validation

M18 validation should include:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```
