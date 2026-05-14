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

## Minimal Config

```toml
listen_addr = "127.0.0.1:3000"
database_url = "sqlite://taru.db"
ffprobe_path = "ffprobe"
scan_concurrency = 1
probe_concurrency = 2

[library]
id = "018f0000-0000-7000-8000-000000000001"
name = "Movies"
root = "F:/Media/Movies"
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
