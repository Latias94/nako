# Release Packaging And Distribution Design

Status: Active
Last updated: 2026-05-21

## Problem

Taru now has a repeatable self-hosted release readiness baseline, but it still
assumes a developer-style source checkout. Operators need a productized path for
installing, configuring, running, upgrading, and diagnosing Taru without reading
chat history or reverse-engineering build commands.

The current gap is not another media feature. The gap is distribution trust:
release artifacts must be buildable, identifiable, configuration-checked,
container-friendly, and covered by smoke evidence.

## Target State

- A release package contract defines what Taru ships for self-hosted operators.
- The server binary has operator-safe startup/config validation behavior.
- Docker/compose examples are grounded in a real build path rather than only a
  PostgreSQL service example.
- GitHub Actions or local scripts can produce release artifacts with version,
  checksum, and validation evidence.
- Release notes/checklist docs explain install, upgrade, rollback, backup, and
  known caveats.
- Downloads are evaluated as a first-class Taru-Managed Artifact candidate, but
  not mixed into packaging implementation unless explicitly split.

## Artifact Contract V0

The packaging lane treats the release artifact as an operator contract. A Taru
self-hosted release may be produced from source, a container image, or a binary
archive, but each distribution form must expose the same contract:

### Ships In The Release

- `taru-server` executable for the selected target platform.
- Example `taru.toml` files for SQLite and PostgreSQL deployments.
- Container/compose definitions once RPD-030 lands.
- Release manifest with Taru version, git revision, target triple, build time,
  and included file list once RPD-040 lands.
- Checksums for generated release artifacts once RPD-040 lands.
- Operator install, first-start, upgrade, rollback, backup, diagnostics, and
  support-bundle docs once RPD-050 lands.

### Does Not Ship In The Release

- Media Library contents.
- Database files, generated artifact bytes, transcode/remux cache, or NFO
  sidecar backups.
- Real secrets, tokens, provider keys, database passwords, or private proxy
  credentials.
- GPL/reference-project source from `repo-ref/`.
- Addon sidecar binaries unless a later addon distribution policy defines a
  separate artifact boundary.

### Runtime State Layout

Packaged examples should converge on this durable layout:

| Concern | Host path example | Container path target | Durability |
| --- | --- | --- | --- |
| Config | `/etc/taru/taru.toml` | `/config/taru.toml` | Operator-owned, backup recommended |
| SQLite DB | `/var/lib/taru/taru.db` | `/data/taru.db` | Durable, backup required before upgrade |
| Managed Artwork Artifacts | `/var/lib/taru/artwork` | `/data/artwork` | Durable Taru-managed artifacts |
| Remux/transcode staging | `/var/cache/taru/remux` | `/cache/remux` | Rebuildable cache |
| Media Library | `/media/movies` | `/media/movies:ro` by default | Operator-owned source of truth |

PostgreSQL deployments replace the SQLite DB file with a PostgreSQL service and
volume. Compose examples must keep database volume state outside the Taru server
container and must not place database credentials in committed config files.

### Operator Safety Contract

- Startup/config preflight must be runnable without connecting to production
  databases or external providers by default.
- Preflight output must avoid full database URLs, raw local media paths, tokens,
  proxy URLs, or provider secrets.
- Unsafe public binds without auth are a hard failure.
- Auth enabled without a usable token environment variable is a hard failure.
- Artifact/staging path checks must distinguish static validation from an
  explicit create/write probe.
- Container examples must use durable external volumes and placeholder
  environment variables for secrets.

## Baseline Inventory

Current repository state at RPD-010:

| Area | Current state | RPD implication |
| --- | --- | --- |
| Workspace version | `Cargo.toml` version `0.1.0`, Rust `1.85`, AGPL server license | Artifact manifest can reuse workspace package metadata. |
| Server binary | `crates/taru-server` has a `taru-server` CLI with `config-example`, `serve`, scan, list, metadata, NFO commands | RPD-020 should add `config-check` rather than hiding preflight inside `serve`. |
| Config examples | `deploy/sqlite/taru.toml` and `deploy/postgres/taru.toml` exist | Packaging should reuse these examples and make them preflight-friendly. |
| Compose examples | `deploy/compose/postgres.yml` currently provides only PostgreSQL | RPD-030 must add a Taru server container/compose shape. |
| Dockerfile | No Taru server `Dockerfile` or `.dockerignore` exists | RPD-030 owns the initial build context and image contract. |
| Release scripts | `scripts/release-gate.*`, `self-host-smoke.*`, and `postgres-contract-harness.*` exist | RPD-040 should add artifact scripts and call existing gates rather than duplicating them. |
| CI | `.github/workflows/release-gate.yml` runs fast, PostgreSQL, self-host smoke, API/SDK redaction gates | RPD-040 should add artifact job shape that invokes repo scripts. |
| Deployment docs | `docs/deployment/SELF_HOSTED.md` and `BACKUP_RESTORE_UPGRADE.md` exist | RPD-050 should distinguish source-built dev flows from packaged operation. |

## In Scope

- Packaging/distribution workstream planning and closeout gates.
- Server binary release contract and version/diagnostic surface review.
- Config validation / startup preflight behavior for operator mistakes.
- Dockerfile and compose shape for Taru server plus PostgreSQL.
- Local packaging scripts and CI release artifact job shape.
- Release checklist, checksums/SBOM placeholders, and operator docs.
- Follow-on evaluation for Metadata, NFO/link management, Playback/transcode,
  Network traversal, AI, and Downloads.

## Out Of Scope

- Implementing new Metadata provider breadth.
- Implementing NFO soft/hard link management.
- Implementing download managers, torrent/Usenet integrations, or offline sync.
- Implementing network traversal/tunneling runtime.
- Implementing AI product features.
- Publishing to real registries or app stores without a separate release policy.

## Architecture Direction

- Treat packaging as an operator contract, not a build-script afterthought.
- Keep the source-built developer workflow working while adding artifact-based
  flows.
- Make config validation explicit and redaction-safe. Startup failures should
  name missing settings, invalid directories, unsupported database modes, or
  unsafe binds without printing secrets.
- Prefer reproducible local scripts first; CI should call the same scripts where
  practical.
- Keep durable state outside containers by default: database, artifact root,
  media libraries, NFO sidecars, config, and secrets must survive image changes.
- Downloads, if accepted later, should use Taru-Managed Artifact language and
  bounded safety policies: no raw path leaks, no direct Addon writes, explicit
  staging/quarantine, and a clear distinction between Addon External Fetch and
  Taru-managed download state.

## Candidate Follow-On Product Lanes

### Metadata Provider Breadth

High value after packaging because operators will immediately ask for TMDB,
Douban, and Bangumi quality. Scope should include provider capability registry,
matching policy, raw response retention, manual confirmation, rate-limit/backoff,
and conflict UX.

### NFO And Link Management

High value for interoperability with existing libraries. Scope should include
sidecar import/export policy, soft/hard link authority, backup/restore behavior,
and redacted diagnostics. It should build on existing NFO preservation and VFS
write policies.

### Playback / Transcode Product Hardening

High value for daily use. Scope should include runtime presets, hardware
capability diagnostics, failure explanations, transcode queue policy, and
operator-facing FFmpeg checks.

### Downloads

Potentially high value but high risk. Downloads can mean many different things:
remote file import, offline transcode/download for clients, Addon-managed
fetches, torrent/Usenet acquisition, or library ingestion from watch folders.
The first Taru-safe slice should likely be **managed import/download staging**:
Taru tracks a remote URL or Addon-proposed artifact, stores it under quarantine
or staging, validates it, then promotes it to a Media Library through explicit
operator action. Acquisition protocols should be separate Addon Sidecars, not
core server logic.

### Network Traversal

High value for self-hosted access but security-sensitive. It should be split
from packaging after the local deployment story is stable.

### AI

Useful after metadata/import boundaries are strong. Best first slice is assisted
matching/title cleanup or local inference explanation, not a broad AI agent.
