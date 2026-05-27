# Changelog

All notable changes to Nako are recorded here.

## 0.1.0-alpha.2 - 2026-05-27

Nako alpha.2 is still a technical-preview release, but it is a much more
complete self-hosted media server baseline than alpha.1. This release focuses
on usable browser playback, local users and access controls, Admin/media
navigation, Addon-driven metadata, casting foundations, and more reliable
FFmpeg/transcode diagnostics.

### Added

- Media Web foundation inside the existing web app, including media browsing,
  media detail routes, a playback shell, and browser player progress writes.
- Ticketed browser playback for direct media streams, remux output, HLS
  playlists, and HLS segments.
- Durable playback and transcode session tracking so active, finished, failed,
  cancelled, and reused playback work can be inspected.
- Local credential sessions, invitation-based local registration, user status,
  roles, and library access policies.
- Library access enforcement on public routes and playback policy resolution
  for direct play, remux, transcode, and policy target decisions.
- Management context links so operators can move from media/library context to
  relevant Admin actions.
- Renderer session persistence, renderer Admin diagnostics, policy-checked
  play commands, renderer transport tickets, cast-safe media transport, and
  the first Chromecast official adapter contract.
- Library metadata profiles, scan-time NFO import, scan acquisition pipeline
  writeback, Addon bulk scrape scheduling, Addon metadata graph writeback, and
  host-owned metadata application policy for authority, lock, merge, and
  catalog projection.
- Addon event subscriptions, event delivery, scheduler claims/work queues,
  replay filters, source catalog discovery, manager lifecycle plan intents,
  outbound task dispatch credentials, host-owned task runtime dispatch, and
  official addon catalog facts.
- Expanded Admin diagnostics for playback runtime, renderer runtime, storage,
  jobs, Addons, metadata, managed artwork, system config, runtime settings, and
  policy readiness.
- Transcode execution policy, profile-driven playback capability planning,
  runtime FFmpeg inventory diagnostics, FFmpeg probe inventory, and a
  stage-aware hardware pipeline planner.
- Release packaging, Docker image publishing, public crate dry-run/publish
  workflows, release gates, self-host smoke coverage, and PostgreSQL managed
  artwork contract coverage.

### Changed

- Browser playback now uses host-issued tickets instead of exposing raw media
  access through generic Admin assumptions.
- Addon and provider metadata writes now go through a host-owned metadata
  application seam instead of letting adapter code decide merge/catalog policy.
- Pre-production database migrations were flattened into a simpler baseline for
  the next alpha while there are no production users to preserve.
- Admin Web moved toward route-first v2 surfaces and shared Media/Admin
  coexistence instead of treating media browsing as a separate product.
- CPU HLS fallback readiness now requires the software encoders it actually
  needs (`libx264` and `aac`) instead of assuming CPU transcode always works.
- HLS transcode unavailability no longer prevents the server from starting;
  Admin diagnostics report unavailable HLS readiness and selected HLS slots
  accurately.
- Redaction and SDK/Admin contract checks were broadened so sensitive paths,
  locators, raw provider payloads, tokens, and secrets stay out of generated
  responses.

### Upgrade Notes

- This is still an alpha release. Back up the database, artwork artifacts,
  config, secrets, and NFO sidecars before upgrading.
- Do not downgrade an alpha.2 database unless restoring a complete alpha.1
  backup. The migration baseline was intentionally flattened before alpha.2.
- The Addon Protocol remains `0.1.0-alpha.1` for this release. Addons targeting
  the alpha.1 protocol should continue to register against alpha.2 unless they
  depend on host features added after alpha.1.
- If FFmpeg lacks `libx264` or `aac`, HLS transcode is reported unavailable
  instead of silently falling back and failing later. Direct play, remux,
  browsing, and Admin startup should still work.

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
