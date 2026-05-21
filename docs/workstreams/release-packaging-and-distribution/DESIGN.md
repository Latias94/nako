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
