# Official Addon E2E Alpha2 - Handoff

Status: Completed
Last updated: 2026-05-23

## Current State

The workstream has been opened and scoped. Nako `v0.1.0-alpha.1` is published
with release artifacts, public crates, and GHCR images. The official companion
repository has `nako-metadata-scraper@0.1.0-alpha.1`.

OAE2E-020, OAE2E-030, OAE2E-040, and OAE2E-050 are complete. The published
Nako GHCR image can run `nako-server --help` and `ffmpeg -version`. The
official metadata scraper can run locally in fixture/default mode and passes
its direct sidecar smoke without provider secrets.

`scripts/official-addon-e2e-smoke.ps1` now proves the hosted loop with released
surfaces: it runs `ghcr.io/latias94/nako-server:0.1.0-alpha.1`, installs or
uses the published `nako-metadata-scraper@0.1.0-alpha.1` binary from crates.io,
registers the Addon through Nako Admin API, runs hosted health, enables it, and
executes one redaction-safe hosted metadata resource diagnostic.

Protocol compatibility diagnostics now have explicit server coverage:
unsupported `manifest.protocol_version` is rejected during registration, and a
hosted resource response using an unsupported protocol version returns
`protocol_mismatch` without echoing request payloads or sidecar secrets.

The user-facing docs now point to `scripts/official-addon-e2e-smoke.ps1` as the
official alpha host/addon smoke entrypoint. That script defaults to the
published crates.io binary and only uses a sibling workspace build when a clean
candidate addon worktree is being validated.

## Next Task

Start OAE2E-060.

Goal: close the lane or split remaining alpha2 follow-ons.

Suggested first steps:

1. Capture final evidence and any remaining risks in `EVIDENCE_AND_GATES.md`.
2. Run `verify-rust-workstream` against the current lane state.
3. Split Addon Manager, marketplace, package signing, or provider breadth
   follow-ons if they became the next real work.
4. Summarize the completed alpha loop in the closeout note.

## Known Risks

- CI may need fixture-only mode if real provider APIs require secrets.
- Running both services locally may need stable port assignment and token
  redaction.
- Addon registration may expose a missing host-side admin/smoke seam; split it
  only if it is not a small fix.
- `F:/SourceCodes/Rust/nako-official-addons` may be ahead of the published
  alpha.1 binary. Use the smoke script's default crates.io binary mode when
  proving released alpha.1; use `-AddonBinarySource workspace` only for a clean
  candidate Addon worktree.
- `ghcr.io/latias94/nako-metadata-scraper:0.1.0-alpha.1` was not readable from
  this environment during OAE2E-030, so the current release proof uses the
  crates.io binary instead of an Addon container image.
