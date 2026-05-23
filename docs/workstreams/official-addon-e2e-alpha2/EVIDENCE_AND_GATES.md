# Official Addon E2E Alpha2 - Evidence And Gates

Status: Active
Last updated: 2026-05-23

## Required Gates

Prefer narrow gates while iterating. Broaden only when the host/addon loop
touches shared runtime behavior.

Baseline docs gate:

```bash
git diff --check
```

Protocol/package gates:

```bash
cargo nextest run -p nako-addon-protocol protocol_version --no-fail-fast
cargo nextest run -p nako-addon-client --no-fail-fast
```

Server/addon host gates:

```bash
cargo check -p nako-server --tests
cargo nextest run -p nako-server addon --no-fail-fast
```

Official Addon sidecar gate, run in `F:/SourceCodes/Rust/nako-official-addons`:

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Docker/image smoke, if using the published server image:

```bash
docker pull ghcr.io/latias94/nako-server:0.1.0-alpha.1
docker run --rm --entrypoint nako-server ghcr.io/latias94/nako-server:0.1.0-alpha.1 --help
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | OAE2E-010 | Workstream opened after Nako `v0.1.0-alpha.1`, crates.io public SDK crates, GHCR server image, and `nako-metadata-scraper@0.1.0-alpha.1` were published. | Pass |
| 2026-05-23 | OAE2E-020 | `docker run --rm --entrypoint nako-server ghcr.io/latias94/nako-server:0.1.0-alpha.1 --help` pulled the public GHCR image and printed the server CLI. | Pass |
| 2026-05-23 | OAE2E-020 | `docker run --rm --entrypoint ffmpeg ghcr.io/latias94/nako-server:0.1.0-alpha.1 -version` confirmed FFmpeg is present in the published image. | Pass |
| 2026-05-23 | OAE2E-020 | In `F:/SourceCodes/Rust/nako-official-addons`, `cargo build -p nako-metadata-scraper` passed, then `addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9100` passed against a locally started sidecar. Output confirmed manifest `nako.official.metadata-scraper@0.1.0-alpha.1`, protocol `0.1.0-alpha.1`, enabled provider `fixture`, one metadata candidate, and one generated artifact. | Pass |

## Redaction Rules

- Do not record raw Addon Tokens, Admin Tokens, provider API keys, or full local
  media paths.
- Resource-call evidence may include Addon IDs, protocol versions, resource
  names, status codes, and redacted diagnostic summaries.
- Provider-specific payloads should use fixture data unless a private support
  process is explicitly agreed.

## Closeout Evidence

Closeout requires:

- final command evidence for the host/addon loop;
- docs updated to match commands;
- explicit split/defer notes for Addon Manager, marketplace, provider breadth,
  and package signing.
