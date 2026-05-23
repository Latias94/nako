# Changelog

All notable changes to Nako are recorded here.

## 0.1.0-alpha.1 - 2026-05-23

### Added

- Self-hosted Nako server technical preview for local and private-network use.
- Rust modular-monolith backend with explicit crates for core domain records,
  persistence, metadata, NFO, catalog/search, library ingestion, VFS, playback,
  transcode policy, automation, events, public/admin API shapes, and Addon
  protocol surfaces.
- SQLite and PostgreSQL persistence paths with backend contract coverage.
- Media Library scan, Local Inference, Source State, Media Source, and
  Provisional Hierarchy support.
- Metadata provider surfaces for TMDB, Bangumi, and Douban with provider runtime
  configuration and diagnostics.
- NFO import/export and local metadata authority groundwork.
- Managed Import, Nako-managed artwork, Library File Write, and promotion/apply
  workflows for operator-confirmed library mutation.
- Playback source selection, remux/transcode planning, hardware acceleration
  policy, runtime diagnostics, and bounded staging paths.
- Addon Sidecar protocol, scoped Addon Tokens and Grants, Addon Health Check,
  Addon Install Guide generation, Admin Addon operations, and experimental
  reference addon support.
- Public `nako` Rust SDK facade crate for Addon Protocol and Addon Client
  integrations.
- Official companion addon alignment with
  `nako-metadata-scraper@0.1.0-alpha.1` in `nako-official-addons`.
- Admin API and Admin Web Console surfaces for operations, diagnostics, addon
  onboarding, credential/grant management, and runtime status.
- Deployment examples for SQLite and PostgreSQL, Docker Compose examples,
  config preflight, release packaging script, backup/restore guidance, and
  redaction-safe support bundle expectations.

### Changed

- Docker builds now use Rust 1.95 and a cargo-chef planner/builder flow so
  dependency builds can be cached separately from application source changes.
- Docker builds now accept `DEBIAN_MIRROR` and `DEBIAN_SECURITY_MIRROR` build
  arguments for networks where `deb.debian.org` is unstable.
- Workspace release metadata now declares the alpha version, author, repository,
  homepage, and project description.
- Addon/client protocol and SDK-facing crates now use Apache-2.0 OR MIT, while
  server-side crates remain AGPL-3.0-or-later.
- The Addon Protocol Version is now the SemVer-shaped runtime compatibility
  contract `0.1.0-alpha.1`, separate from Addon Version and Rust crate package
  versions.
- Release CI now includes crates.io dry-run/publish workflow support for the
  public permissive crates and creates GitHub Release drafts for tagged server
  packages.
- Docker publish CI can build, smoke-test, and push tagged alpha images to
  GitHub Container Registry without adding extra JavaScript actions.

### Known Limitations

- This is an alpha technical preview, not a stable Jellyfin/Plex replacement.
- Public API, Admin API, Addon Protocol, database schema, and generated SDK
  shapes may change without compatibility guarantees before beta.
- Addons are externally run Addon Sidecars; Nako does not yet provide an Addon
  Manager, marketplace, package signing, or sidecar process supervision.
- Network tunnel support is currently policy/readiness oriented; Nako does not
  run a built-in NAT traversal or relay service.
- AI-assisted workflows are not part of this release promise.
- Direct public-internet exposure is not recommended; run behind local-only,
  private-network, VPN, reverse-proxy, or tunnel boundaries with auth enabled.
