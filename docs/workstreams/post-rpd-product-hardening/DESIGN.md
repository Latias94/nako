# Post-RPD Product Hardening Design

Status: Active
Last updated: 2026-05-21

## Why This Lane Exists

`release-packaging-and-distribution` closed the gap between source checkout and
self-hosted operator artifact. Packaging is necessary but not sufficient: a
packaged Taru must also be safe and useful for a real media library.

The next danger is architectural sprawl. Metadata breadth, NFO/link management,
playback diagnostics, downloads, network traversal, AI, and addon distribution
are all valuable, but they have different data-loss, security, runtime, and UX
risks. Treating them as one implementation stream would blur authority and make
review impossible.

This umbrella freezes the product order and split rules. Concrete code changes
belong in dedicated execution workstreams.

## Relevant Authority

- ADRs:
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/deployment/SELF_HOSTED.md`
  - `docs/deployment/RELEASE_ARTIFACTS.md`
  - `docs/deployment/RELEASE_CHECKLIST.md`
- Related workstreams:
  - `docs/workstreams/release-packaging-and-distribution`
  - `docs/workstreams/metadata-catalog`
  - `docs/workstreams/transcode-runtime`
  - `docs/workstreams/addons-automation`
  - `docs/workstreams/access-boundary-auth`

## Problem

After packaging, Taru has several attractive next directions:

- richer built-in metadata providers;
- local interoperability through NFO and links;
- playback/transcode diagnostics;
- downloads/import;
- remote access;
- AI;
- addon runtime/distribution.

Each direction depends on the same core product truth: Taru must know what a
**Media Item** is, where a **Media Source** lives, which **Provider Mapping** is
trusted, what can write to the library, and which operations require an
**Acceptance Workflow**. Without an explicit order, the project risks adding
features that bypass local authority, leak paths/secrets, or create later
schema/API churn.

## Target State

- A durable post-RPD roadmap orders all major product-hardening lanes.
- The first execution lane, `metadata-provider-breadth`, is closed with fresh
  evidence.
- The next mainline execution lane is `nfo-link-authority`.
- Downloads are split as `managed-import-staging`, not implemented as generic
  acquisition inside core Taru.
- AI writes enter only as **Generated Artifacts** accepted through Taru-owned
  workflows.
- Addon distribution remains deferred until metadata/import/file-write
  side-effect boundaries are proven.
- Every execution lane has its own workstream, evidence gates, and closeout.

## In Scope

- Roadmap ordering after RPD.
- Split criteria and dependencies between product lanes.
- Opening the first execution workstream.
- Keeping later lanes visible without smuggling implementation into this
  umbrella.

## Out Of Scope

- Concrete schema migrations.
- Provider HTTP changes.
- NFO/link file mutations.
- Download/acquisition protocol implementation.
- Network tunnel runtime.
- AI model runtime or autonomous library writes.
- Addon manager, bundle signing, or native plugin ABI.

## Dependency Map

| Lane | Depends on | Unlocks |
| --- | --- | --- |
| Metadata Provider Breadth | RPD, metadata provider runtime, metadata merge policy | safer NFO conflict handling, import promotion, AI matching, addon metadata suggestions |
| NFO And Link Authority | metadata authority, NFO preservation, VFS write policy | safe library file writes, import promotion, interoperability |
| Playback / Transcode Ops Hardening | RPD preflight, transcode runtime, admin diagnostics | daily playback confidence, support bundles, future profiles |
| Managed Import Staging | metadata matching, NFO/link authority, artifact storage patterns | safe downloads/acquisition, watch-folder import, addon-proposed artifacts |
| Network Access Boundary | auth, packaging, release docs | remote clients, tunnel provider integration |
| AI Assisted Library Ops | metadata/import authority, acceptance workflows | explainable matching/title cleanup without autonomous writes |
| Addon Runtime / Distribution | addon permissions, side-effect APIs, managed artifact intake | broader ecosystem without core trust-boundary leaks |

## Roadmap Waves

### Wave 0 — Umbrella And First Execution Lane

Open this roadmap plus `metadata-provider-breadth`.

Status: completed on 2026-05-21.

### Wave 1 — Metadata Provider Breadth

Make TMDB, Douban, and Bangumi matching explainable and capability-aware. This
is the highest-leverage lane because imports, NFO conflicts, AI suggestions,
and future addon metadata all depend on trusted provider identity and conflict
semantics.

Status: completed on 2026-05-21. Shipped boundaries:

- diagnostics-safe provider capability reporting;
- deterministic candidate match decisions and reasons;
- ambiguous search refresh stops before fetch/cache/commit;
- cross-provider candidate review exposes conflicts without canonical writes.

### Wave 2 — Local Authority And Playback Confidence

Open `nfo-link-authority` and `playback-transcode-ops-hardening` after the
metadata provider boundaries are stable enough to avoid conflicting source of
truth decisions. These can proceed in parallel if their write scopes stay
separate.

### Wave 3 — Managed Import Staging

Implement Taru-owned quarantine/staging/import-plan behavior for operator URLs,
watch-folder candidates, or addon-proposed artifacts. Do not start with
torrent, Usenet, or protocol-specific acquisition in core Taru.

### Wave 4 — Network And AI

Harden remote access endpoints and add AI assistance only after the metadata
and import acceptance boundaries are trustworthy.

### Wave 5 — Addon Runtime / Distribution

Grow addon installation/distribution after core side-effect and artifact
intake policies are stable.

## Lane Contracts

### Metadata Provider Breadth

First safe slice:

- provider capability registry;
- matching policy classification;
- cross-provider conflict explanation;
- diagnostics that expose capabilities without secrets;
- no automatic canonical overwrite when confidence is ambiguous.

### NFO And Link Authority

First safe slice:

- local sidecar authority model;
- import/export conflict policy;
- link inventory diagnostics;
- no new symlink/hardlink mutation until dry-run and rollback rules exist.

### Playback / Transcode Ops Hardening

First safe slice:

- operator-facing FFmpeg/hardware diagnostics;
- preset validation;
- fallback reason reporting;
- no distributed queue or adaptive ladder in the first slice.

### Managed Import Staging

First safe slice:

- Taru-managed staging/quarantine records;
- content probe and duplicate hints;
- metadata inference;
- explicit promote plan;
- no direct library writes without confirmation.

### Network Access Boundary

First safe slice:

- remote access endpoint model;
- reverse-proxy/tunnel deployment contract;
- trusted proxy/header policy;
- no built-in NAT traversal runtime first.

### AI Assisted Library Ops

First safe slice:

- generated matching/title-cleanup suggestions;
- confidence and explanation;
- acceptance workflow;
- no autonomous writes.

### Addon Runtime / Distribution

First safe slice:

- addon package/manifest validation;
- sidecar trust and permission policy;
- no in-process native plugin ABI.

## Architecture Direction

Use a serial mainline and parallel sidecars:

- serial mainline: metadata authority -> local file authority -> managed
  import authority;
- parallel sidecars: playback diagnostics and network documentation can advance
  when they avoid schema/API collisions;
- deferred ecosystem work: AI and addon distribution should consume accepted
  core boundaries, not create their own mutation paths.

The umbrella intentionally avoids code changes. Execution lanes must own their
own TODO ledger, validation commands, closeout evidence, and follow-on splits.

## Post-Metadata Re-Score — 2026-05-21

`metadata-provider-breadth` removed the biggest provider-authority ambiguity:
Taru can now explain provider capabilities, reject or pause unsafe matches, and
surface cross-provider candidate conflicts without mutating canonical metadata.
That changes the next-lane ordering:

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| NFO And Link Authority | Highest | Metadata identity is now reviewable, but local sidecar/link writes still carry the highest data-loss risk. Taru needs dry-run, conflict, backup, and rollback semantics before import or addon file writes deepen. | Open next as the mainline execution lane. |
| Playback / Transcode Ops Hardening | High sidecar | It improves daily operator confidence and has little schema overlap with NFO/link if kept to diagnostics, preset validation, fallback reasons, and runtime evidence. | Safe parallel sidecar after the next mainline lane is opened. |
| Managed Import Staging | Blocked by local authority | Downloads/import promotion need NFO/link write boundaries and duplicate/link inventory rules to avoid unsafe library mutation. | Defer until NFO/link authority lands. |
| Network Access Boundary | Useful but not data-authority critical | Auth and packaging already exist; tunnel/proxy ergonomics matter, but they should not precede local library safety. | Defer after local authority or run as docs-only sidecar. |
| AI Assisted Library Ops | Blocked by acceptance durability | Candidate review exists, but durable suggestion queues and manual acceptance are not yet productized. | Defer until acceptance workflow and local authority are stronger. |
| Addon Runtime / Distribution | Last consumer | Addons should consume proven metadata, file-write, import, and artifact boundaries rather than inventing their own mutation paths. | Defer. |

The downloads idea remains valuable, but the correct shape is
`managed-import-staging`: Taru-owned quarantine, probe, duplicate/link hints,
metadata inference, and explicit promote plans. It should not start as a
generic downloader in core Taru.

## Closeout Condition

This umbrella can close when:

- the roadmap has been translated into concrete execution workstreams;
- each completed execution lane records fresh evidence and closeout;
- deferred lanes are either opened or explicitly re-scored;
- `docs/workstreams/README.md` points to the current active product lane.
