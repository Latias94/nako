# Phase 3.2: TMDB Provider MVP and Metadata Refresh Job

## Status

Implemented in the current workspace.

## Scope

Phase 3.2 adds the first real external metadata path:

- TMDB movie search and movie-detail fetch behind `taru-metadata` traits
- metadata refresh service that reuses field locks and merge policy
- raw TMDB detail response cache in `provider_raw_responses`
- persisted `metadata_refresh` jobs with durable input and summary JSON
- bounded metadata refresh concurrency in the server runtime
- HTTP and CLI triggers for refreshing one indexed item
- config placeholders that resolve provider secrets from environment variables

The implementation is intentionally movie-first. Series, seasons, episodes,
Douban, Bangumi, and addon metadata providers should reuse the same provider
trait and refresh service shape.

## Configuration

TMDB credentials are not stored in SQLite and are not serialized into job
inputs. The server config stores an environment variable name:

```toml
metadata_concurrency = 2

[metadata.tmdb]
enabled = true
access_token_env = "TMDB_READ_ACCESS_TOKEN"
api_base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "en-US"
include_adult = false
```

The environment variable should contain a TMDB API read access token. Job
inputs store only the item ID, provider name, force flag, and language.

## Refresh Flow

For one item:

1. Load the indexed `MediaItem`.
2. If a TMDB external ID already exists, fetch details directly.
3. Otherwise search TMDB by title and release year, then fetch the best
   candidate details.
4. Convert TMDB data into `CanonicalMetadata`.
5. Load field locks and merge through `MetadataMergePolicy`.
6. Persist the updated item and the raw TMDB detail response.
7. Complete the job with a `MetadataRefreshSummary`.

## API and CLI

HTTP trigger:

```text
POST /items/{item_id}/metadata/refresh
```

The route returns `202 Accepted` with a queued job. The background job updates
the job status and summary.

CLI trigger:

```powershell
cargo run -p taru-server -- --config taru.toml refresh-metadata <item_id>
```

The CLI command runs synchronously and prints the job plus refresh summary as
JSON.

## Verification

Automated coverage uses mocked provider responses and does not call the real
TMDB network API.

Key coverage:

- locked fields survive refresh
- existing TMDB external IDs skip search and fetch directly
- provider raw responses are cached
- job input does not include secrets
- HTTP route queues a metadata refresh job
- TMDB detail JSON maps into canonical metadata

Expected gates:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

## Out of Scope

- TMDB series, season, and episode metadata
- Douban and Bangumi providers
- secret storage or key rotation UI
- rate-limit backoff and retry scheduling
- provider result review UI before applying changes

## Reference Links

- TMDB API authentication: https://developer.themoviedb.org/docs/authentication-application
- TMDB movie search: https://developer.themoviedb.org/reference/search-movie
- TMDB movie details: https://developer.themoviedb.org/reference/movie-details
