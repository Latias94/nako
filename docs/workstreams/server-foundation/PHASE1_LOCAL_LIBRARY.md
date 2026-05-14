# Phase 1: Local Library Foundation

## Status

In progress. The first CLI-based local library loop is implemented and verified
with tests plus a local smoke run.

## Scope

Phase 1 proves that Taru can:

- read a local server configuration
- connect to SQLite and run migrations
- scan a configured local media root
- parse basic movie and episode names
- persist `Library`, `MediaItem`, and `MediaSource`
- call ffprobe with bounded concurrency
- persist media probe results and stream details
- query indexed content through a minimal CLI

## Current CLI

Print an example config:

```powershell
cargo run -p taru-server -- config-example
```

Scan and probe the configured local library:

```powershell
cargo run -p taru-server -- --config taru.toml scan
```

List indexed sources, items, and probe results:

```powershell
cargo run -p taru-server -- --config taru.toml list
```

## Configuration

```toml
database_url = "sqlite://taru.db"
ffprobe_path = "ffprobe"
probe_concurrency = 2

[library]
id = "018f0000-0000-7000-8000-000000000001"
name = "Movies"
root = "F:/Media/Movies"
```

The configured root is mounted internally as `local:///` for the local VFS
backend. Scanned media sources are stored as stable `local:///...` locators.

## Verification

Automated gates:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

Manual smoke verification performed:

- generated a 1-second sample video with ffmpeg
- ran `taru-server --config <temp>/taru.toml scan`
- ran `taru-server --config <temp>/taru.toml list`
- confirmed one media source, title `Sample Movie`, and two probe streams
- ran scan twice and confirmed idempotency:
  - first scan inserted 1 source and probed 1 source
  - second scan inserted 0 sources, updated 1 source, and skipped 1 probe

## Remaining Phase 1 Work

- Add a stable sample config file or documented config path convention.
- Improve CLI help text and JSON output shape if needed.
- Add persisted job state instead of summary-only execution.
- Add structured logging and tracing initialization.
- Decide whether Phase 1 should include a minimal HTTP API in addition to CLI.
