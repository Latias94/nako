# Official Addon E2E Alpha2

Status: Active
Last updated: 2026-05-23

## Why This Lane Exists

Nako `v0.1.0-alpha.1` now ships a server release, public Addon Protocol crates,
a GHCR server image, and a published official companion Addon in
`nako-official-addons`. Those pieces prove separately, but the alpha user path
is still not proven as one loop: start Nako, start `nako-metadata-scraper`,
register the Addon, verify health, and make a resource call through Nako's
hosted Addon runtime.

## Relevant Authority

- ADRs:
  - `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- Existing docs:
  - `README.md`
  - `CHANGELOG.md`
  - `docs/guides/ADDON_AUTHOR_GUIDE.md`
  - `docs/api/HTTP_API.md`
  - `docs/deployment/RELEASE_CHECKLIST.md`
- Related repositories:
  - `F:/SourceCodes/Rust/nako-official-addons`
  - `https://github.com/Latias94/nako-official-addons`

## Problem

The released server, protocol crates, and official Addon can each be consumed,
but there is no repeatable cross-repository smoke that proves they work together
from a fresh operator or Addon author perspective.

## Target State

When this lane closes:

- a documented alpha smoke path starts Nako and `nako-metadata-scraper`,
  registers the Addon, checks hosted health, and performs a hosted resource
  call;
- the smoke uses released or release-shaped surfaces first, not path-only
  workspace assumptions;
- protocol version compatibility and mismatch failures are diagnosable;
- docs describe the exact responsibility split between Nako server and the
  external Addon Sidecar;
- follow-on provider breadth, Addon Manager automation, and marketplace work are
  explicitly split.

## In Scope

- Main-repo smoke script or documented command sequence for Nako plus
  `nako-metadata-scraper`.
- Host-side Addon registration, hosted health check, and resource-call evidence.
- Negative compatibility proof for unsupported `protocol_version` behavior when
  feasible without large fixtures.
- README/deployment/addon-guide updates that teach the alpha loop.
- Optional CI workflow entry if the smoke can be made deterministic on GitHub
  runners without leaking provider secrets.

## Out Of Scope

- Addon Manager install/update/remove automation.
- Addon process supervision by Nako.
- Plugin marketplace, package signing, or trust distribution.
- Broad TMDB/Douban/Bangumi provider correctness.
- AI workflows, NAT traversal, mobile UI work, or playback/transcode changes.
- Publishing new Nako server or Addon versions unless the smoke exposes a
  release-blocking bug.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `nako-metadata-scraper@0.1.0-alpha.1` targets Addon Protocol `0.1.0-alpha.1`. | High | `nako-official-addons` README and Cargo metadata. | The smoke must pin a different Addon release or split a compatibility follow-on. |
| Nako can register externally run Addon Sidecars through existing Admin/Addons surfaces. | High | Completed Addon onboarding and operations lanes. | This lane must first close the missing host-side seam. |
| A fixture/default provider mode can smoke without real provider secrets. | Medium | `nako-official-addons` docs mention fixture/TMDB/Bangumi runtime defaults. | CI must stay documentation-only or require opt-in secrets. |
| The released server image is sufficient for host-side smoke. | Medium | `ghcr.io/latias94/nako-server:0.1.0-alpha.1` manifest and smoke passed. | Use source-built Nako for the first proof, then split image parity. |

## Architecture Direction

Treat Addon Sidecars as external processes and the Addon Protocol as the
compatibility contract. The main repo owns host behavior: registration,
capability validation, hosted health, token/grant boundaries, and resource-call
diagnostics. The official Addons repo owns sidecar runtime behavior, provider
mapping, and its own release cadence.

This lane should not move Addon lifecycle supervision into Nako. The proof is
that a self-hosted operator can wire the pieces together explicitly.

## Closeout Condition

This lane can close when:

- the alpha E2E smoke is documented and repeatable;
- host-side and sidecar-side evidence are recorded;
- protocol compatibility behavior is proven or the missing seam is split;
- docs reflect the shipped behavior;
- and Addon Manager/provider breadth follow-ons are explicitly deferred.

