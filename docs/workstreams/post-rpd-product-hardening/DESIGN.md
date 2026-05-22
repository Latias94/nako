# Post-RPD Product Hardening Design

Status: Active
Last updated: 2026-05-22

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
- Completed execution lanes record evidence and closeout before the next lane
  becomes current.
- Downloads are split first as staged artifact intake, not implemented as
  generic acquisition inside core Taru.
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
| Downloads / Watch Folder Intake | managed import staging, accepted promotion apply, NFO sidecar apply, playback supportability | safe acquisition candidates without direct library mutation |
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

Status: staging and non-mutating promotion preview completed on 2026-05-21.
Actual promotion apply is split to `link-apply-and-import-promotion`.

Status: promotion apply safety progressed on 2026-05-21.
`link-apply-and-import-promotion` implemented durable acceptance/audit,
app-service replay,
VFS-mediated copy/hardlink/symlink target apply, catalog commit ordering,
duplicate evidence, and cleanup-complete/cleanup-pending audit. LAIP-070 split
NFO sidecar import/export mutation to `nfo-sidecar-promotion-apply` because it
is a separate **Library File Write** and metadata-authority workflow. LAIP-080
closed the promotion apply lane after fresh closeout gates.

### Wave 4 — Acquisition Intake, Network, And AI

Open downloads/watch-folder intake after metadata, local file writes, staged
import, NFO sidecar apply, and playback supportability are trustworthy. Harden
remote access endpoints as a sidecar when it does not weaken auth or library
mutation boundaries. Add AI assistance only after generated artifact intake and
acceptance workflows are explicit.

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

## Post-Managed-Import Re-Score — 2026-05-21

`managed-import-staging` completed the non-mutating side of import promotion:
durable artifact records, redacted diagnostics, promotion preview, VFS link
dry-run summary, duplicate hints, NFO authority hints, provider identity review,
and explicit blockers. That removes the need to keep staging open, but it also
makes the next highest-risk boundary sharper: Taru still cannot safely mutate a
Media Library root from a staged artifact.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Link Apply And Import Promotion | Highest | This is the first mutating step after staging: copy/link target creation, Media Source commit, duplicate evidence, rollback/cleanup, and audit must be proven before downloads/watch-folder/addon-proposed artifacts can become library files. | Open next as the mainline execution lane. |
| Playback / Transcode Ops Hardening | High sidecar | Still useful for daily operations and mostly disjoint from import apply if limited to diagnostics, preset validation, fallback reasons, and runtime evidence. | Safe parallel sidecar candidate after apply domain work starts. |
| Network Access Boundary | Useful but not data-authority critical | Remote access matters, but it should not precede local library mutation safety now that import staging is ready. | Defer or run as docs/runtime sidecar. |
| AI Assisted Library Ops | Blocked by acceptance durability | AI suggestions must reuse the same acceptance/audit model rather than invent autonomous writes. | Defer until promotion acceptance/apply is durable. |
| Addon Runtime / Distribution | Downstream consumer | Addons should propose artifacts and side effects into proven Taru-owned apply paths. | Defer until apply boundaries are proven. |

Downloads remain downstream of this decision. The next shape is not a generic
downloader; it is first a safe promotion apply boundary for already-staged
artifacts.

## Post-LAIP-070 Split Decision — 2026-05-21

The apply boundary is now split into two separate mutation classes:

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Link Apply And Import Promotion | Complete | It owns accepted staged artifact promotion, VFS-mediated target creation, catalog commit ordering, duplicate evidence, and cleanup audit. NFO sidecar mutation has been split out. | Closed at LAIP-080; use as the downstream promotion boundary for acquisition/import lanes. |
| NFO Sidecar Promotion Apply | High | NFO export/import is the next local data-loss boundary: sidecar backup, round-trip preservation, local authority, field locks, hierarchy confirmation, rollback/repair, idempotency, and audit need a dedicated lane. | Opened as `nfo-sidecar-promotion-apply`; candidate next mainline lane for PRPH-080. |
| Playback / Transcode Ops Hardening | High sidecar | Daily playback confidence still matters and can stay mostly disjoint if limited to diagnostics, preset validation, fallback reasons, and runtime evidence. | Re-score against NFO sidecar apply during PRPH-080. |
| Network Access Boundary | Useful but not data-authority critical | Remote access should consume existing auth/deployment boundaries and avoid weakening library mutation policy. | Defer or run as docs/runtime sidecar. |
| AI Assisted Library Ops | Blocked by acceptance durability | AI suggestions should enter as generated artifacts and reuse accepted metadata/import/sidecar apply boundaries. | Defer until accepted side-effect lanes are stable. |
| Addon Runtime / Distribution | Downstream consumer | Addons should call Taru-owned apply/file-write APIs instead of inventing direct filesystem mutation. | Defer until core side-effect APIs are proven. |
| Downloads / Watch Folder | Downstream of promotion apply | Acquisition should produce staged artifacts and use existing promotion apply; it should not bypass sidecar/file-write policy. | Re-score after LAIP closeout and NFO sidecar apply decision. |

## Post-LAIP Closeout Re-Score — 2026-05-21

`link-apply-and-import-promotion` is now complete. Taru can accept a staged
artifact promotion, revalidate plan facts, create the target through VFS,
commit catalog/duplicate state after target durability, and record
cleanup-complete or cleanup-pending outcomes after partial failure. That closes
the first mutating import boundary, but not every library-file side effect.

The remaining highest-risk data-loss boundary is accepted NFO sidecar
import/export. It is the last local authority write still sitting between
today's safe promotion path and later download/watch-folder/addon breadth.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| NFO Sidecar Promotion Apply | Highest | It owns the remaining local **Library File Write** and metadata-authority mutation: NFO Round Trip, backup, retention, field locks, hierarchy confirmation, rollback/repair, idempotency, and redacted audit. It directly protects local library data before downloads, AI, or Addons can safely propose file/metadata changes. | Select as next mainline lane. Execute `nfo-sidecar-promotion-apply` NSPA-020. |
| Playback / Transcode Ops Hardening | High sidecar | Playback confidence matters, but existing `transcode-runtime` and `admin-playback-runtime-diagnostics` already cover a baseline. Additional ops hardening is valuable and mostly disjoint if limited to diagnostics, preset validation, fallback reasons, and support evidence. | Keep as the safest parallel sidecar after NSPA acceptance/audit starts. |
| Downloads / Watch Folder | High but downstream | Promotion apply is now proven, but acquisition still must not bypass NFO sidecar policy, staged artifact intake, or accepted apply. Watch-folder imports often rely on sidecar metadata next to media files. | Defer until NFO sidecar apply acceptance/audit exists, then open a downloader/watch-folder acquisition lane. |
| Network Access Boundary | Useful but not data-authority critical | Remote access should harden endpoint, trusted proxy, and tunnel-provider contracts without weakening auth or library writes. | Defer or run as docs/runtime sidecar after current local authority write is underway. |
| AI Assisted Library Ops | Blocked by accepted side effects | AI should produce **Generated Artifacts** and consume acceptance workflows, not directly mutate canonical metadata or sidecars. | Defer until NFO sidecar apply and import acceptance surfaces are proven. |
| Addon Runtime / Distribution | Downstream consumer | Addons should call Taru-owned metadata/import/file-write APIs and scoped tokens rather than inventing direct filesystem mutation. | Defer until core side-effect APIs, including NFO sidecar apply, are stable. |

PRPH-080 therefore does not open a new lane. It selects the existing
`nfo-sidecar-promotion-apply` workstream as the next execution lane and returns
implementation to NSPA-020.

## Post-NSPA Closeout Re-Score — 2026-05-21

`nfo-sidecar-promotion-apply` is now complete. Taru can explicitly accept NFO
sidecar import/export, revalidate preview facts, export through NFO Round Trip
and VFS write/backup/retention behavior, import local authority through
canonical metadata and field locks, confirm hierarchy, restore from backups
after audit failure, and record `failed_before_mutation`, `rollback_complete`,
or `repair_pending` outcomes without raw path/XML leakage.

This changes the product risk ordering: the highest local data-loss boundaries
for metadata, sidecars, staged import, and library-file mutation are now
represented by accepted apply workflows. The next product-hardening pressure is
operator playback confidence and transcode supportability.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Playback / Transcode Ops Hardening | Highest | Playback/transcode is the next daily operator pain point. Existing `transcode-runtime` and Admin playback diagnostics provide a baseline, but productization still needs clear preset validation, fallback explanations, hardware evidence, support bundles, and failure taxonomy without changing metadata/import authority. | Open next as the mainline execution lane. |
| Downloads / Watch Folder | High but still downstream | Promotion apply and NFO sidecar apply are now safe enough for acquisition to consume, but protocol/download breadth must still start as staged artifacts and explicit promote/apply operations. | Re-score after playback ops, or open only if scoped to Managed Import intake without bypassing apply boundaries. |
| Network Access Boundary | Useful sidecar | Remote access can harden trusted proxy, tunnel documentation, and endpoint policy without touching library mutation. | Safe parallel docs/runtime sidecar after playback lane opens. |
| AI Assisted Library Ops | Downstream consumer | AI can propose generated artifacts and metadata/sidecar changes, but must reuse accepted apply boundaries and never write autonomously. | Defer until generated artifact intake is explicitly designed. |
| Addon Runtime / Distribution | Downstream consumer | Addons now have more Taru-owned side-effect APIs to consume, but distribution/runtime should not expand before scoped API exposure and side-effect permission UX are decided. | Defer until Admin/API exposure of sidecar apply is stable. |

PRPH-090 therefore selects Playback/Transcode Ops Hardening as the next
mainline lane. PRPH-100 opens `playback-transcode-ops-hardening` with PTOH-020
as the first executable task. It must stay runtime/diagnostic-focused: no
metadata schema churn, no downloader protocol work, and no library file
mutation.

## Playback/Transcode Ops Lane Open — 2026-05-22

`playback-transcode-ops-hardening` opened as the active mainline execution lane.
It builds on completed `playback-streaming`, `transcode-runtime`, and
`admin-playback-runtime-diagnostics` work rather than reopening those baselines.

The first executable slice was PTOH-020: define a stable Admin-only playback
runtime readiness contract that classifies FFmpeg probe, hardware capability,
selected fallback, transcode budget, remote playback budget, and staging
prerequisites without exposing raw paths, Source Locators, FFmpeg command
lines, output paths, stderr payloads, secrets, or credentials.

Downloads/watch-folder, network, AI, and addon runtime remain downstream or
parallel only when they consume existing accepted Taru-owned boundaries.

## Post-PTOH Closeout Re-Score — 2026-05-22

`playback-transcode-ops-hardening` is now complete. Taru can report playback
runtime readiness, explain hardware fallback and validation failures, categorize
transcode session failures, and provide bounded Admin-only support evidence
without raw Source Locators, local paths, FFmpeg command lines, output paths,
raw stderr, fingerprints, secrets, credentials, or Public Client API churn.

That shifts the next product risk back to acquisition breadth. Metadata
authority, local NFO/link semantics, managed staging, accepted promotion apply,
NFO sidecar apply, rollback/repair, and playback supportability are now proven
well enough to accept new candidates, but only if acquisition produces staged
artifacts and explicit apply plans rather than direct library mutations.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Downloads / Watch Folder Intake | Highest | Operators need a way to bring files into Taru from watched folders or future download outputs. The safe shape is now clear: discover or accept candidates into Taru-owned staging, produce redacted diagnostics and promotion evidence, and hand off to existing Link Apply and NFO Sidecar Apply workflows. | Open next as the mainline execution lane. Start with intake/domain/watch-folder candidate records, not torrent/Usenet/protocol-specific downloader runtime. |
| Network Access Boundary | High sidecar | Remote clients matter, and playback supportability is now stronger. This lane can harden endpoint, reverse-proxy, trusted-header, and tunnel-provider policy without touching library mutation. | Safe parallel sidecar after downloads/watch-folder intake is opened, or next if acquisition needs to wait. |
| AI Assisted Library Ops | Medium downstream | AI can propose match/title/sidecar cleanup as Generated Artifacts, but autonomous writes would bypass accepted metadata/import/sidecar boundaries. | Defer until generated artifact intake and acceptance queues are explicit. |
| Addon Runtime / Distribution | Medium downstream | Addons have more Taru-owned side-effect APIs to consume, but runtime/distribution still needs scoped permission UX and side-effect routing into proven apply paths. | Defer until acquisition intake and generated artifact/side-effect queues are stable. |
| Support Bundles / Playback UI | Useful follow-on | PTOH-050 exposes a read model, but not downloadable bundles, retention, or Admin UI workflow. | Split as a playback support follow-on only if operator workflow becomes urgent; do not block acquisition intake. |

PRPH-110 therefore selects downloads/watch-folder intake as the next mainline
lane. PRPH-120 should open `downloads-watch-folder-intake` with a narrow first
slice: staged artifact acquisition intake and watch-folder candidate discovery.
It must not add direct library writes, protocol-specific downloader behavior,
NFO mutation shortcuts, network traversal, AI writes, or Addon runtime changes.

## Post-DWI Closeout Re-Score — 2026-05-22

`downloads-watch-folder-intake` is now complete. Taru can durably represent
Acquisition Intake Candidates, discover watch-folder entries through
storage/VFS list/stat, classify ready/incomplete/unsupported candidates, expose
Admin-only redacted diagnostics, and explicitly hand accepted candidates into
Managed Import artifacts without Media Source creation, promotion apply, NFO
sidecar mutation, Public Client API changes, or `taru-client-protocol` churn.

That closes the first safe acquisition boundary. The remaining product risk is
no longer local library mutation; it is exposing Taru safely to real clients and
deployment topologies without weakening auth, path redaction, proxy trust, or
side-effect boundaries.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Network Access Boundary | Highest | Remote clients and self-hosted deployment need explicit external endpoint, trusted proxy/header, origin, and tunnel-provider policy. This can advance product readiness without touching metadata authority, Managed Import, NFO apply, acquisition intake, AI, Addons, or downloader protocols. | Open next as the mainline execution lane. First slice should be policy/readiness and Admin diagnostics, not built-in NAT traversal runtime. |
| Protocol Downloader Integrations | High but downstream | Intake now exists, so torrent/Usenet/RSS/download-client adapters have a safe target. They still need separate credential, retry, sandbox, and adapter-failure policies. | Split after network policy or when a concrete adapter is selected; adapters must submit candidates/artifacts into Acquisition Intake. |
| Background Watch Scheduling | Useful follow-on | DWI proved polling/discovery through storage/VFS but did not add scheduler or OS watcher runtime. Scheduling needs job/runtime ownership, debounce, leases, and backpressure decisions. | Split as an intake operations follow-on; do not reopen DWI. |
| Admin Intake Workflow Polish | Useful follow-on | Typed diagnostics exist, but full operator workflow for accept/reject/bulk actions needs UX and command semantics. | Split only after acceptance commands and promotion preview UX are explicitly scoped. |
| AI Assisted Library Ops | Medium downstream | AI can now propose Generated Artifacts or candidate evidence into Taru-owned queues, but autonomous metadata, sidecar, or file writes would still bypass acceptance. | Defer until generated artifact proposal/acceptance queues are opened. |
| Addon Runtime / Distribution | Medium downstream | Addons can consume more stable Taru-owned side-effect APIs, but runtime/distribution still needs manifest validation, permission UX, and side-effect routing into proven apply/intake paths. | Defer until network policy and generated artifact/side-effect queue semantics are explicit. |

PRPH-130 therefore selects Network Access Boundary as the next recommended
mainline lane. PRPH-140 should open `network-access-boundary` with a narrow
first slice: endpoint/proxy/tunnel policy, trusted external URL handling,
CORS/origin constraints, and Admin-only readiness diagnostics. It must not add
built-in NAT traversal runtime, downloader protocols, AI writes, Addon runtime,
or library mutation behavior.

## Network Access Boundary Lane Open — 2026-05-22

`network-access-boundary` opened as the active mainline execution lane. It
builds on completed inbound bearer auth, self-hosted deployment docs, playback
supportability, and acquisition intake rather than reopening those baselines.

The first executable slice is NAB-020: define and validate a network access
policy/config model for local-only, reverse-proxy, private-network, and
tunnel-provider modes. This slice should produce safe defaults and redacted
config-check diagnostics before any concrete tunnel runtime or Public Client
endpoint-discovery behavior is added.

NAB-020 and NAB-030 are now complete. Taru has a validated network access
policy and an HTTP boundary that preserves bearer-auth precedence, keeps health
public, enforces configured browser origins and preflight behavior, and trusts
forwarded scheme/host only when trusted proxy headers are enabled and the
remote source matches exact-IP/CIDR policy.

The next executable slice is NAB-040: expose Admin-only network readiness
diagnostics and typed Admin web support. This must summarize mode, endpoint,
trusted proxy, origin, and tunnel-provider readiness without exposing bearer
tokens, raw forwarded headers, credential-bearing URLs, local paths, tunnel
secrets, Public Client API shape, or `taru-client-protocol` changes.

## Post-NAB Closeout Re-Score — 2026-05-22

`network-access-boundary` is now complete. Taru has explicit network exposure
policy for local-only, reverse-proxy, private-network, and tunnel-provider
modes; config-check readiness; request-time trusted forwarded header handling;
origin/CORS behavior that preserves auth order; and Admin-only redacted network
diagnostics. It did not ship built-in NAT traversal runtime, relay services,
endpoint discovery, identity/RBAC, downloader protocols, Addon runtime, Public
Client API changes, or `taru-client-protocol` churn.

That closes the remote access boundary needed before AI and Addon surfaces
become more useful to operators. The highest remaining product risk is now
AI-like output entering library operations without a Generated Artifact review
and acceptance contract.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| AI Assisted Library Ops | Highest | Metadata, NFO/file-write, import/promotion, playback, acquisition intake, and network boundaries are now proven. AI suggestions can help matching/title cleanup, but only if they enter Taru as Generated Artifacts with redacted diagnostics and explicit acceptance rather than autonomous writes. | Open next as the mainline execution lane. First slice should be proposal/readiness and Admin diagnostics, not local model runtime. |
| Addon Runtime / Distribution | High downstream | Addons can now consume stable side-effect APIs and network policy, but distribution/runtime should not expand before generated artifact/side-effect queue semantics are explicit. | Defer until AI proposal/acceptance semantics are proven. |
| Protocol Downloader Integrations | High but separate | Acquisition intake exists and network policy is stronger, but torrent/Usenet/download-client adapters still need credential, retry, sandbox, and adapter-failure policies. | Split as a downloader lane; do not mix with AI or Addon distribution. |
| Concrete Tunnel Runtime / Endpoint Discovery | Useful follow-on | Network policy/readiness exists, but starting cloudflared/ngrok/Tailscale or exposing remote endpoint discovery to clients is a separate runtime/security problem. | Split after current policy lane; do not reopen NAB. |

PRPH-150 therefore closes Network Access Boundary and selects AI Assisted
Library Ops as the next mainline lane. PRPH-160 should open
`ai-assisted-library-ops` with a narrow first slice: Generated Artifact
proposal/readiness, redacted Admin diagnostics, and explicit accept/reject
planning. It must not add local model runtime, embeddings/vector DB,
provider-specific AI adapters, autonomous writes, Addon distribution, Public
Client API changes, or `taru-client-protocol` changes.

## AI Assisted Library Ops Lane Open — 2026-05-22

`ai-assisted-library-ops` opened as the active mainline execution lane after
Network Access Boundary closeout. It builds on completed metadata provider,
NFO/link, Managed Import, promotion apply, NFO sidecar apply, playback,
acquisition intake, network, and external automation foundations.

The first executable slice was AILO-020: deepen existing Automation Artifacts
into a Generated Artifact proposal/readiness queue. AILO-030 then exposed
Admin-only proposal diagnostics and typed Admin Web support. AILO-040 added
explicit accept/reject planning for metadata-cleanup proposals without
autonomous canonical metadata, sidecar, Media Source, Managed Import, or
library-file writes. AILO-050 closed the lane and split provider-specific
adapters, local model runtime, embeddings/vector search, Public Client display,
protocol downloaders, Addon distribution, and deeper metadata-authority apply
follow-ons.

## Post-AILO Closeout Re-Score — 2026-05-22

`ai-assisted-library-ops` is now complete. Taru can represent AI-like outputs
as Generated Artifact proposals with stable target/provenance/payload/readiness
summaries, expose them through Admin-only redacted diagnostics, and record
explicit accept/reject planning for metadata-cleanup suggestions without
autonomous writes. Public Client API and `taru-client-protocol` remain
unchanged.

That removes the last major prerequisite before returning to the Addon roadmap:
Addons can now consume side-effect APIs, acquisition intake, network readiness,
and Generated Artifact proposal queues instead of inventing direct mutation or
AI-specific paths.

| Lane | Current score | Why | Decision |
| --- | --- | --- | --- |
| Addon Runtime / Distribution | Highest | Addon protocol, side-effect APIs, Admin Addon operations, network policy, acquisition intake, and Generated Artifact acceptance semantics are now explicit. The next risk is making sidecar package/install/runtime readiness safe before broader distribution. | Open next as the mainline execution lane. First slice should be package/install descriptor and redacted install-guide readiness, not Addon Manager automation. |
| Protocol Downloader Integrations | High but separate | Downloader adapters have safe intake and network policy targets, but they need credential, retry, sandbox, and adapter-specific failure policies. | Split as a downloader lane after Addon runtime/distribution starts or when a concrete adapter is selected. |
| Concrete Tunnel Runtime / Endpoint Discovery | Useful follow-on | Network policy/readiness exists, but launching tunnel providers or exposing endpoint discovery to clients has separate runtime/security implications. | Split as a network runtime/client-discovery lane. |
| Local AI Runtime / Vector Search | Useful follow-on | Generated Artifact semantics exist, but model execution, embeddings, storage, GPU scheduling, and provider adapters are separate operational concerns. | Split after Addon runtime/distribution or when a concrete provider/runtime is selected. |

PRPH-170 therefore opens `addon-runtime-and-distribution` as the next mainline
lane. The first executable task is ARD-020 package/install descriptor and
redacted install-guide boundary. It must not add Addon Manager discovery,
automatic install/update, package signing trust root, process supervision,
Native Plugin ABI, direct library writes, Public Client API changes, or
`taru-client-protocol` changes.

## Closeout Condition

This umbrella can close when:

- the roadmap has been translated into concrete execution workstreams;
- each completed execution lane records fresh evidence and closeout;
- deferred lanes are either opened or explicitly re-scored;
- `docs/workstreams/README.md` points to the current active product lane.
