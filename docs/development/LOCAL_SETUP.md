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
webhook_concurrency = 2
remux_timeout_ms = 1800000
remux_staging_root = "F:/Taru/cache/remux"

[transcode]
hardware_acceleration = "none"
hardware_fallback = "cpu"
cpu_concurrency = 1
gpu_concurrency = 1

[staging]
max_bytes = 107374182400
retention_ms = 604800000
cleanup_on_startup = true

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

## WebDAV Preview Config

M6 includes a read-only WebDAV preview for one configured library. Add
`[library.webdav]` to switch the configured library root from `local:///` to a
WebDAV storage URI. The WebDAV password is a secret environment reference; it
must not be placed in `base_url`, source locators, jobs, logs, or scan state.

```toml
[library]
id = "018f0000-0000-7000-8000-000000000001"
name = "Remote Movies"
root = "F:/Media/Movies"

[library.webdav]
root = "webdav:///Movies"
base_url = "https://nas.example.test/dav"
username = "media"
password_env = "TARU_WEBDAV_PASSWORD"
timeout_ms = 30000
max_attempts = 2
```

```powershell
$env:TARU_WEBDAV_PASSWORD = "<webdav password>"
cargo run -p taru-server -- --config taru.toml scan
```

Remote probe inputs are staged under `remux_staging_root/probe-inputs`. Remote
remux and HLS inputs are staged under `remux_staging_root/inputs` before FFmpeg
is invoked. Direct play streams WebDAV ranges through `taru-vfs` into the HTTP
response body. `[staging].max_bytes` limits the total manifest-tracked remote
input staging bytes before new probe or FFmpeg input staging starts.
`[staging].retention_ms` controls when staged inputs become startup cleanup
candidates.

Runtime notes:

- `scan_concurrency` bounds directory scan work.
- `probe_concurrency` bounds ffprobe work.
- `metadata_concurrency` bounds provider/NFO metadata jobs.
- `remux_concurrency` bounds copy-remux FFmpeg sessions.
- `webhook_concurrency` bounds explicit webhook dispatch attempts.
- `remux_timeout_ms` bounds one remux or HLS FFmpeg process.
- `remux_staging_root` stores staged remux outputs; HLS outputs are staged
  below `remux_staging_root/hls`. Remote probe and FFmpeg input staging also
  uses children below this root.
- `[staging].max_bytes` bounds manifest-tracked remote probe and FFmpeg input
  staging. The default is 100 GiB.
- `[staging].retention_ms` defaults to 7 days, and
  `[staging].cleanup_on_startup` removes expired staged input files and manifest
  records during startup.
- `[transcode].cpu_concurrency` and `[transcode].gpu_concurrency` bound HLS
  transcode sessions by selected acceleration class.

Hardware acceleration values:

```text
hardware_acceleration = "none"       # CPU-only x264 command planning
hardware_acceleration = "vaapi"      # VAAPI command planning
hardware_acceleration = "nvenc"      # NVIDIA NVENC command planning
hardware_acceleration = "quick_sync" # Intel Quick Sync command planning

hardware_fallback = "cpu"  # fall back to CPU when requested hardware is unavailable
hardware_fallback = "fail" # fail planning when requested hardware is unavailable
```

Hardware acceleration is currently modeled as capability, policy, resource
budget, and FFmpeg command planning. The workspace tests use mocked/static
capabilities and do not require a real GPU.

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
