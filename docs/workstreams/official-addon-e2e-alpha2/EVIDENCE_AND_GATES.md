# Official Addon E2E Alpha2 - Evidence And Gates

Status: Completed
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

Hosted official Addon E2E smoke:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

The script defaults to the published `nako-metadata-scraper@0.1.0-alpha.1`
binary from crates.io. Use `-AddonBinarySource workspace` only when validating
a clean candidate in `F:/SourceCodes/Rust/nako-official-addons`.

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
| 2026-05-23 | OAE2E-030 | `cargo install nako-metadata-scraper --version 0.1.0-alpha.1 --root target/oae2e-alpha2-addon-install/0.1.0-alpha.1 --force --locked` installed the published Addon binary from crates.io. | Pass |
| 2026-05-23 | OAE2E-030 | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1` started `ghcr.io/latias94/nako-server:0.1.0-alpha.1`, started the published metadata scraper binary with container-visible `base_url=http://host.docker.internal:19100`, registered the Addon through Nako Admin API, ran hosted health, enabled the Addon, and completed one hosted metadata resource diagnostic with `status=succeeded`, `attempts=1`, and `http_status=200`. | Pass |
| 2026-05-23 | OAE2E-040 | `cargo nextest run -p nako-server register_addon_routes_disabled_by_default_and_validate_contract --no-fail-fast` passed after adding coverage for unsupported `manifest.protocol_version` registration rejection. | Pass |
| 2026-05-23 | OAE2E-040 | `cargo nextest run -p nako-server admin_addon_resource_call_diagnostic_classifies_safe_failures --no-fail-fast` passed after adding coverage for hosted resource responses with unsupported protocol versions. | Pass |
| 2026-05-23 | OAE2E-040 | `cargo nextest run -p nako-addon-protocol protocol_version --no-fail-fast` passed. | Pass |
| 2026-05-23 | OAE2E-040 | `cargo nextest run -p nako-server addon --no-fail-fast` passed: 53 passed, 217 skipped. | Pass |
| 2026-05-23 | OAE2E-050 | `README.md`, `docs/deployment/RELEASE_CHECKLIST.md`, and `docs/guides/ADDON_AUTHOR_GUIDE.md` now point to `scripts/official-addon-e2e-smoke.ps1` as the published alpha host/addon smoke entrypoint. | Pass |
| 2026-05-23 | OAE2E-060 | `cargo fmt --all -- --check`; `cargo nextest run -p nako-server addon --no-fail-fast`; `git diff --check` | Pass. Fresh closeout gates passed after docs and code updates. |

Note: the sibling `nako-official-addons` worktree is currently ahead of the
published alpha.1 binary and failed a workspace source build because provider
facts changed locally. The OAE2E release smoke therefore intentionally validates
the published crates.io binary by default.

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

Closeout status: complete. Remaining work belongs in separate follow-on lanes.
