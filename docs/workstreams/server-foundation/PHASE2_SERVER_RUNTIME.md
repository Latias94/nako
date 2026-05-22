# Phase 2: Server Runtime and API Foundation

## Status

Implemented in the current workspace. Phase 2 turns the Phase 1 CLI loop into
a long-running server runtime with persisted background jobs, structured logs,
and a minimal HTTP API.

## Scope

Phase 2 proves that Nako can:

- start `nako-server` as a long-running HTTP service
- read `listen_addr`, SQLite, ffprobe, scan, probe, and local library settings
- connect to SQLite and run migrations during startup
- persist background job lifecycle state
- enqueue library scan work through HTTP without blocking the request
- keep scan and probe work behind explicit concurrency limits
- expose indexed library, source, item, probe, and job state through HTTP
- initialize structured logging through `tracing_subscriber`
- map internal `NakoError` values to safe JSON HTTP error responses
- keep CLI scan/list commands on the same application service used by HTTP

## Configuration

```toml
listen_addr = "127.0.0.1:3000"
database_url = "sqlite://nako.db"
ffprobe_path = "ffprobe"
scan_concurrency = 1
probe_concurrency = 2

[library]
id = "018f0000-0000-7000-8000-000000000001"
name = "Movies"
root = "F:/Media/Movies"
```

The configured local root is mounted internally as `local:///`. Phase 2 still
supports one configured local library; multi-library and remote-storage
management are intentionally left for later phases.

## Current CLI

Print an example config:

```powershell
cargo run -p nako-server -- config-example
```

Start the HTTP server:

```powershell
cargo run -p nako-server -- --config nako.toml serve
```

`serve` is the default command:

```powershell
cargo run -p nako-server -- --config nako.toml
```

Run the configured scan synchronously from the CLI:

```powershell
cargo run -p nako-server -- --config nako.toml scan
```

List indexed sources, items, and probe results:

```powershell
cargo run -p nako-server -- --config nako.toml list
```

## HTTP API

Phase 2 exposes the first minimal API surface:

```text
GET  /health
GET  /libraries
POST /libraries/{library_id}/scan
GET  /libraries/{library_id}/sources
GET  /items
GET  /sources/{source_id}/probe
GET  /jobs/{job_id}
```

`POST /libraries/{library_id}/scan` returns `202 Accepted` with a queued job.
The scan job then runs in the background. Job state can be read through
`GET /jobs/{job_id}`.

## Job State

Jobs are persisted in SQLite with:

- `queued`
- `running`
- `succeeded`
- `failed`

Each job records:

- `kind`
- `resource_class`
- `library_id`
- `source_id`
- `input_json`
- `summary_json`
- `error`
- `queued_at`
- `started_at`
- `completed_at`

The first implemented resource class is `disk.scan`. The scan pipeline calls
the probe pipeline, which remains bounded by `probe_concurrency`.

## Observability

The server initializes `tracing_subscriber` and honors `RUST_LOG`. Background
scan jobs use structured spans and fields including:

- `job_id`
- `library_id`
- `resource_class`
- `probe_concurrency`

HTTP errors are converted to JSON responses with stable error codes. Database,
storage, and provider errors return safe public messages while detailed context
is kept in structured logs.

## Verification

Automated gates:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

Key test coverage:

- SQLite job lifecycle persistence
- application-level scan job success path
- HTTP health and library routes
- HTTP scan job enqueue response
- HTTP 404 error mapping for missing jobs

## Out of Scope

- TMDB, Douban, Bangumi, and NFO enrichment
- direct play, remux, transcode, or HLS serving
- authentication and authorization
- addon runtime and webhook delivery
- remote-storage providers and byte-range cache
