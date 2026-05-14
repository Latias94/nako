# Local Development Setup

## Prerequisites

- Rust toolchain compatible with the workspace `rust-version`
- `cargo-nextest`
- FFmpeg tools available on `PATH` when running real probe smoke tests
- PowerShell on Windows

Useful checks:

```powershell
rustc --version
cargo --version
cargo nextest --version
ffprobe -version
ffmpeg -version
```

## Common Commands

Format:

```powershell
cargo fmt --all
cargo fmt --all -- --check
```

Check:

```powershell
cargo check --workspace
```

Test:

```powershell
cargo nextest run --workspace
```

Run the server:

```powershell
cargo run -p taru-server -- --config taru.toml serve
```

Print an example config:

```powershell
cargo run -p taru-server -- config-example
```

Run a synchronous local-library scan:

```powershell
cargo run -p taru-server -- --config taru.toml scan
```

Refresh TMDB metadata for one indexed item:

```powershell
$env:TMDB_READ_ACCESS_TOKEN = "<tmdb read access token>"
cargo run -p taru-server -- --config taru.toml refresh-metadata <item_id>
```

## Minimal Config

```toml
listen_addr = "127.0.0.1:3000"
database_url = "sqlite://taru.db"
ffprobe_path = "ffprobe"
ffmpeg_path = "ffmpeg"
scan_concurrency = 1
probe_concurrency = 2
metadata_concurrency = 2
remux_concurrency = 1
remux_timeout_ms = 1800000

[library]
id = "018f0000-0000-7000-8000-000000000001"
name = "Movies"
root = "F:/Media/Movies"

[metadata.tmdb]
enabled = false
access_token_env = "TMDB_READ_ACCESS_TOKEN"
api_base_url = "https://api.themoviedb.org/3"
image_base_url = "https://image.tmdb.org/t/p/original"
language = "en-US"
include_adult = false
```

## Logging

The server uses `tracing_subscriber` and honors `RUST_LOG`.

```powershell
$env:RUST_LOG = "taru_server=debug,taru_library=debug"
cargo run -p taru-server -- --config taru.toml serve
```

## Reference Repositories

Repositories under `repo-ref/` are for architecture and behavior research only.
Do not copy code, comments, SQL, tests, assets, or generated files from them
into Taru.
