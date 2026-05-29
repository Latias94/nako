# Workstreams

Workstreams group related milestones, TODOs, phase notes, and design context.
They are not ownership silos; they are long-running areas of architectural
attention.

## Current Workstreams

- [hls-seek-restart-lifecycle](hls-seek-restart-lifecycle/README.md):
  active feature/refactor lane for making HLS playback seek-aware through
  generation identity, restart admission, FFmpeg seek command planning, and
  manifest-backed artifact serving without mixing playback lifecycle policy into
  transcode execution.
- [playback-planner-transcode-seam-deepening](playback-planner-transcode-seam-deepening/README.md):
  completed fearless refactor lane for moving remux/HLS playback
  `TranscodeProfile` construction out of `nako-playback` and into
  `nako-transcode` while preserving playback decisions, request identity, and
  runtime behavior before HLS seek, HDR, downmix, and scheduler work expands
  the surface area.
- [media-server-architecture-progress-map](media-server-architecture-progress-map/README.md):
  closed architecture documentation lane that added the top-level Nako
  architecture map, refreshed playback/transcode roadmap status, and recorded
  the HLS/media-engine boundary in ADR 0052.
- [hls-audio-sidecar-artifacts](hls-audio-sidecar-artifacts/README.md):
  closed fearless refactor lane for generating real HLS audio sidecar
  playlists/segments and publishing `TYPE=AUDIO` master playlist groups only
  for servable artifacts.
- [hls-alternate-audio-renditions](hls-alternate-audio-renditions/README.md):
  closed fearless refactor lane that made HLS selected audio stream mapping
  executable and deferred true `TYPE=AUDIO` groups until audio sidecar artifacts
  exist.
- [hls-master-renditions-authoring](hls-master-renditions-authoring/README.md):
  closed fearless refactor lane for making selected subtitle WebVTT sidecar
  artifacts discoverable through standard HLS master playlist media tags,
  preserving adaptive fMP4, single-variant HLS, artifact serving, and session
  reuse behavior.
- [hls-media-renditions-runtime](hls-media-renditions-runtime/README.md):
  closed fearless refactor lane that added a typed HLS media rendition boundary
  after source-aware adaptive video ladders, including selected subtitle WebVTT
  sidecar command planning, request-variant identity, server artifact
  reconstruction, and reuse coverage.
- [adaptive-hls-source-aware-ladder](adaptive-hls-source-aware-ladder/README.md):
  closed fearless refactor lane that made the adaptive HLS fMP4 runtime
  source-aware after the first fixed-ladder slice, including source/client
  constrained rendition planning, stable request-variant identity, artifact
  reconstruction, and no-audio adaptive FFmpeg stream-map support.
- [web-v0-copy-first-tanstack-refactor](web-v0-copy-first-tanstack-refactor/README.md):
  active copy-first frontend replacement lane that imports the complete
  `repo-ref/nako-admin-web` product shell into `web/`, then removes Next/Vercel
  runtime assumptions, restores Nako API boundaries, moves route ownership to
  TanStack, and records browser/Tauri performance gates before closeout.
- [web-deferred-product-reentry-plan](web-deferred-product-reentry-plan/README.md):
  active planning lane for routing the WBBP-deferred frontend surfaces back
  through real backend/API contracts before downloads, playlists, photos,
  music, podcasts, AI assistant, or automation can re-enter the live web
  runtime.
- [web-media-live-public-client-parity](web-media-live-public-client-parity/README.md):
  active implementation lane for moving the new `web/` Media surface from
  fixture-first browsing toward live Public Client browse/detail, browser-safe
  playback entry, playback state, browser/Tauri validation, and bundle-budget
  gates.
- [web-modern-frontend-and-tauri-foundation](web-modern-frontend-and-tauri-foundation/README.md):
  closed foundation lane that made `web/` the product frontend release line,
  re-authored the v0 UX direction into a Vite React/Tailwind/TanStack app with a
  Tauri path, and recorded why `apps/admin-web` remains validation-only until
  parity gates allow deletion.
- [playback-runtime-boundary-deepening](playback-runtime-boundary-deepening/README.md):
  completed fearless refactor lane for splitting HLS artifact serving, support
  evidence, diagnostics, and runtime/store boundaries out of the broad playback
  app service before adaptive HLS, fMP4, rsmpeg, or remote worker work adds more
  surface area.
- [source-aware-transcode-runtime](source-aware-transcode-runtime/README.md):
  completed execution lane for deepening Nako's HLS/remux transcode stack from
  fixed H.264/AAC command planning into source-aware media facts, structured
  transcode requirements, source-aware hardware pipeline planning, FFmpeg
  command boundaries, and runtime progress/segment supervision.
- [playback-media-maturity-first-slices](playback-media-maturity-first-slices/README.md):
  completed fearless refactor lane for the first post-source-aware playback
  media maturity slices: richer Public Client capability profile input, adaptive
  HLS/fMP4 planning vocabulary, and explicit subtitle/HDR/audio/bitrate/
  resolution compatibility reasons without overclaiming executable fMP4 or ABR
  runtime support.
- [executable-hls-fmp4-runtime-boundary](executable-hls-fmp4-runtime-boundary/README.md):
  completed fearless refactor lane for turning the HLS fMP4 planning vocabulary
  into the first executable runtime slice: fMP4 single-variant request identity,
  staging layout, FFmpeg muxer planning, and safe artifact serving while keeping
  adaptive ladders as a follow-on.
- [transcode-output-shape-hls-manifest-ladder](transcode-output-shape-hls-manifest-ladder/README.md):
  completed fearless refactor lane for deleting transitional transcode
  output-shape states, introducing an explicit HLS artifact manifest boundary,
  and implementing the first adaptive HLS fMP4 ladder runtime slice.
- [playback-capability-profile-and-rendition-planning](playback-capability-profile-and-rendition-planning/README.md):
  completed fearless refactor lane for deleting the shallow `PlaybackProfile`
  adapter, making selected playback output a typed Rendition Plan, and keeping
  browser, Nako renderer, and Chromecast-like playback flows on one
  capability-profile boundary before adaptive HLS, fMP4, DLNA profiles, or
  remote workers expand the surface area.
- [external-casting-adapter-boundary](external-casting-adapter-boundary/README.md):
  active execution lane for adding Chromecast, DLNA, and AirPlay through
  protocol-specific renderer adapters after Nako remote-client cast-safe
  transport. It keeps Nako host-owned policy, Renderer Sessions, Playback
  Sessions, and transport tickets while moving protocol discovery/control into
  adapter boundaries.
- [nako-renderer-cast-safe-transport](nako-renderer-cast-safe-transport/README.md):
  completed execution lane for adding renderer-scoped cast-safe media transport
  for Nako remote clients before protocol-specific Chromecast, DLNA, or AirPlay
  adapters. It kept renderer control on bearer-authenticated Public Client
  routes while making direct, remux, and HLS media URLs target-scoped and
  expiring.
- [playback-policy-and-renderer-targets](playback-policy-and-renderer-targets/README.md):
  completed execution lane for making playback decisions policy-aware and target
  aware after playback/transcode policy deepening. It introduced effective
  playback permissions and renderer target vocabulary for browser, desktop,
  mobile, and future casting without implementing casting protocols in that
  lane.
- [casting-renderer-runtime](casting-renderer-runtime/README.md):
  closed execution lane for implementing casting as Renderer Sessions plus
  protocol adapters after playback policy and renderer targets are in place. It
  shipped Nako-to-Nako direct play, renderer commands, and Admin diagnostics,
  then split non-direct Nako transport, Chromecast, DLNA, and AirPlay into
  follow-on lanes.
- [playback-transcode-policy-deepening](playback-transcode-policy-deepening/README.md):
  completed architecture-first execution lane for making Nako's playback planner,
  transcode policy, runtime inventory, and engine Adapter seams mature enough
  for Jellyfin-class device capability, transcode reason, hardware fallback,
  Admin diagnostics, and playback artifact lifecycle features without copying
  Jellyfin's DLNA model or embedding a new media engine in the server.
- [backend-media-product-deepening](backend-media-product-deepening/README.md):
  completed backend execution lane for hardening Nako's local-media product seams
  after Media Web foundation and credential/session auth. It covers clean
  database baselines, controlled invitation registration, Playback Session
  runtime deepening, and permission-gated Management Context Links between
  media browsing and admin operations without adding frontend UI,
  recommendations, open registration, or native desktop implementation.
- [metadata-application-policy-seam](metadata-application-policy-seam/README.md):
  completed backend execution lane for deepening Nako's host-owned Canonical
  Metadata application seam. It added a `MetadataApplication` Module so Addon
  writeback remains a fact-submission Adapter while Nako owns field locks,
  merge mode, catalog projection, persistence, and apply reporting. Official
  Addon adapter cleanup and scan Addon bulk continuation remain follow-ons.
- [credential-session-auth](credential-session-auth/README.md):
  completed backend execution lane for adding Nako's first local password
  credential and durable session authority after the completed User/Role/Library
  Access contract. It shipped Admin API password provisioning, Public Client
  login/current-account/logout, Bearer session resolution through
  `AuthenticatedPrincipal`, and refreshed generated client contracts. Admin Web,
  Media Web, cookies, invitations, account recovery, SSO, and Management
  Context Links remain focused follow-ons.
- [browser-playback-auth-transport](browser-playback-auth-transport/README.md):
  completed execution lane for choosing and implementing the secure browser
  playback transport that lets Media Web render a real player without exposing
  bearer tokens, raw Source Locators, local paths, or privileged permanent
  stream URLs. ADR 0036 accepts short-lived browser playback tickets; renderer
  transport now remains intentionally separate.
- [media-web-client-foundation](media-web-client-foundation/README.md):
  closed execution lane for the first browser-based Media Web surface inside
  the shared web frontend: local media browsing, search, Media Item detail,
  Source/Version Picker, safe watch shell, and User Playback State through the
  Public Client API without turning Admin routes into the playback client.
  Closeout recommends opening Browser Playback Auth Transport next.
- [identity-and-library-access-contract](identity-and-library-access-contract/README.md):
  completed execution lane for adding the first real post-Single-Admin identity
  model: local users, coarse roles, Library Access, bootstrap administrator
  behavior, principal resolution, Admin API access-management routes, and
  Public Client API effective-access enforcement. Follow-ons own Admin Web
  account UI, Media Web login/account switching, invitations, and Management
  Context Links.
- [client-surface-and-access-product-architecture](client-surface-and-access-product-architecture/README.md):
  active planning lane for deciding how Nako grows from Single-Admin Mode and
  Admin Web V2 into separate but connected Admin Web, Media Web, desktop, and
  mobile surfaces. It records account/access staging, Library Access,
  permission-gated media-to-admin management links, and Tauri/native desktop
  playback direction. Identity/access and Media Web have been split to narrower
  lanes; Management Context Links are next.
- [admin-web-v2-i18n-expansion](admin-web-v2-i18n-expansion/README.md):
  active execution lane for expanding Admin Web V2 localization beyond the
  shell and Media Library management routes. The first slice migrates
  `/overview` and `/access` route-visible copy while preserving API enum values
  and diagnostic facts.
- [admin-web-v2-users-access-readiness](admin-web-v2-users-access-readiness/README.md):
  completed execution lane for adding a truthful Users & Access surface to
  Admin Web V2. The first slice exposes Single-Admin Mode, the stable
  local-admin principal, role/account readiness, and effective Library Access
  without fake account or RBAC mutation controls.
- [admin-settings-configuration-authority](admin-settings-configuration-authority/README.md):
  active backend architecture lane split from Admin Web V2 settings mutation
  readiness. It defines the source of truth, startup merge, hot-apply or
  restart-required semantics, persistence, and Admin API shape needed before
  `/settings` can expose real save controls.
- [admin-web-v2-settings-mutation-authority](admin-web-v2-settings-mutation-authority/README.md):
  active execution lane for defining and implementing the first safe Admin Web
  V2 settings mutation path after the read-only `/settings` route. The first
  readiness task found no safe global settings mutation surface yet; Admin Web
  save controls are blocked until `admin-settings-configuration-authority`
  provides a real backend route or accepted runtime-only model.
- [admin-web-v2-catalog-repair-actions](admin-web-v2-catalog-repair-actions/README.md):
  completed execution lane for turning the read-only Catalog Governance queue into
  bounded, review-plan-driven repair workflows after Generated Artifact review
  and item artwork selection. The first executable task is route/API repair
  readiness, followed by Provider Mapping review-plan and confirmed mutation
  workflows.
- [admin-web-v2-item-artwork-selection](admin-web-v2-item-artwork-selection/README.md):
  completed execution lane for adding an item-scoped Managed Artwork gallery and
  guarded Selected Artwork select/unpublish workflows to Admin Web V2 after the
  Generated Artifact review/actions closeout. Closeout recommends opening
  `admin-web-v2-catalog-repair-actions` next unless settings mutation or
  users/permissions/Library Access is pulled forward.
- [admin-web-v2-generated-artifact-review-actions](admin-web-v2-generated-artifact-review-actions/README.md):
  completed execution lane for adding one-proposal Generated Artifact
  review-plan and accept/reject confirmation workflows to Admin Web V2 after
  the read-only `/automation/generated-artifacts` route and media
  browsing/detail governance lane closeout. Closeout recommends opening
  `admin-web-v2-item-artwork-selection` unless settings mutation or
  users/Library Access is pulled forward.
- [admin-web-v2-media-browsing-and-item-detail-governance](admin-web-v2-media-browsing-and-item-detail-governance/README.md):
  completed execution lane for adding governance-oriented `/catalog` and
  `/items/:itemId` routes after library management, using explicit public-read
  bridges and safe item/source/metadata/artwork/NFO readiness summaries without
  turning Admin Web into a playback client. Closeout recommends opening
  `admin-web-v2-generated-artifact-review-actions`.
- [admin-web-v2-library-management-and-localization](admin-web-v2-library-management-and-localization/README.md):
  completed execution lane for turning the read-only Admin Web V2 Media Libraries
  route into a library-management workflow with `/libraries/:libraryId`,
  metadata-profile visibility/editing, source/scan/NFO action policy, and the
  first Admin Web localization boundary. Closeout recommends opening a bounded
  media browsing/item detail governance lane.
- [admin-web-v2-product-architecture](admin-web-v2-product-architecture/README.md):
  completed product-architecture lane for turning the completed Admin Web V0
  scaffold and app-local Admin API TypeScript contract into a route-first V2
  admin console plan with root product/design context, stack decisions, shared
  UI vocabulary, and a first workflow proof before deeper implementation.
- [addon-notification-bridge](addon-notification-bridge/README.md):
  completed execution lane for the ACK-only official notification bridge proof
  after Addon Event Scheduler And Replay, including host registration,
  scheduler delivery, redaction-safe evidence, and a split provider-adapter
  follow-on.
- [addon-notification-provider-adapters](addon-notification-provider-adapters/README.md):
  completed execution lane for selecting, implementing, verifying, and closing
  the first sidecar-owned notification provider adapter after the ACK-only
  notification bridge proof, without adding provider credentials, templates, or
  provider-specific retry to Nako core.
- [addon-notification-platform-adapters](addon-notification-platform-adapters/README.md):
  completed execution lane for adding the first named notification platform
  adapter, `discord_webhook`, after the generic `http_webhook` proof while
  keeping platform credentials, payload shape, diagnostics, and retry behavior
  inside the sidecar boundary.
- [addon-notification-template-controls](addon-notification-template-controls/README.md):
  completed execution lane for adding safe notification template controls that
  use whitelisted event facts and payload keys without exposing raw event
  payload values, provider secrets, or Nako-managed provider concepts.
- [addon-notification-provider-attempt-history](addon-notification-provider-attempt-history/README.md):
  completed execution lane for adding redaction-safe sidecar-owned provider
  attempt history so operators can inspect notification provider outcomes
  without moving provider retry state into Nako core.
- [addon-notification-provider-live-smoke](addon-notification-provider-live-smoke/README.md):
  completed execution lane for opt-in live smoke coverage of notification
  provider delivery using explicit local secrets, skipped by default and never
  required by CI.
- [addon-event-scheduler-and-replay](addon-event-scheduler-and-replay/README.md):
  completed execution lane for due Addon Event work discovery, durable
  in-flight delivery claims, automatic retry, supervised disabled-by-default
  scheduler lifecycle integration, explicit forced replay, and simple
  redaction-safe event fact filters before notification bridge breadth.
- [metadata-profile-configuration-authority](metadata-profile-configuration-authority/README.md):
  completed source-of-truth lane for making Media Library Metadata Profile updates
  restart-proof by distinguishing preset defaults, explicit TOML overrides, and
  Admin API changes.
- [admin-library-metadata-profile-configuration](admin-library-metadata-profile-configuration/README.md):
  completed Admin API productization lane for reading and updating each Media
  Library's effective Metadata Profile so scan-time NFO Import, Addon Bulk
  Metadata Scrape, and explicit Addon metadata writeback can be configured
  without TOML edits.
- [metadata-acquisition-pipeline](metadata-acquisition-pipeline/README.md):
  completed architecture lane for turning scan-time metadata acquisition into a
  configurable Media Library pipeline, preserving NFO import, keeping Addon
  Bulk Metadata Scrape suggestion-only by default, and proving explicit Addon
  metadata writeback through Nako-owned Side Effects.
- [addon-ecosystem-foundation](addon-ecosystem-foundation/README.md):
  completed architecture lane for Addon Package / Addon Suite deployment, Addon
  Task fingerprinting, official catalog drift prevention, Addon Event
  Delivery, and the first event-driven official addon proof before broader
  notification, watch-state sync, MCP, Arr-stack, DLNA, WebDAV, UPnP, and
  network-tunnel breadth.
- [official-addon-e2e-alpha2](official-addon-e2e-alpha2/README.md):
  completed alpha2 lane for proving Nako `v0.1.0-alpha.1`, the public Addon
  Protocol crates, the GHCR server image, and
  `nako-metadata-scraper@0.1.0-alpha.1` as one repeatable host/addon smoke
  loop without expanding into Addon Manager, marketplace, package signing, or
  provider breadth.
- [fearless-future-architecture-refactor](fearless-future-architecture-refactor/README.md):
  completed execution lane for the next fearless refactor wave after M61-M63,
  covering server runtime control planes, PostgreSQL/backend module shape,
  API DTO boundaries, VFS/library-file-write authority, local inference
  boundaries, Docker-backed validation, and deletion of replaced paths without
  adding new product breadth.
- [nako-brand-identity](nako-brand-identity/README.md):
  design record for Nako's selected source icon, public tagline, one-line
  introduction, icon meaning, and final image-generation prompt.
- [admin-web-addon-credential-grant-onboarding](admin-web-addon-credential-grant-onboarding/README.md):
  completed Addon productization lane for one-time Addon Token issue/rotation,
  token revoke, accepted Addon Grant replacement, and enable readiness in
  Admin Web without expanding into sidecar lifecycle supervision.
- [admin-web-addon-onboarding](admin-web-addon-onboarding/README.md):
  completed Addon productization lane for paste-and-preview manifest JSON
  registration in Admin Web, defaulting registrations to disabled and handing
  off to Addon Operations, Install Guide, and Health Check without expanding
  into Addon Manager URL discovery or sidecar lifecycle supervision.
- [addon-install-guide-generation](addon-install-guide-generation/README.md):
  completed Addon productization lane for generating redaction-safe Docker
  Compose, systemd, Secret Reference, health-check, and registration
  verification guidance for externally run Addon Sidecars without expanding
  into Addon Manager install/update or process supervision.
- [post-rpd-product-hardening](post-rpd-product-hardening/README.md):
  completed roadmap umbrella for ordering post-packaging product lanes across
  metadata provider breadth, NFO/link authority, playback/transcode hardening,
  managed import staging, network access, AI assistance, and addon distribution.
- [addon-runtime-and-distribution](addon-runtime-and-distribution/README.md):
  completed post-RPD mainline lane for Addon Sidecar package/install descriptor,
  redacted install-guide, runtime readiness, task/event/artifact routing, and
  Admin-only diagnostics before any Addon Manager automation, marketplace,
  package signing, process supervision, or Native Plugin ABI.
- [addon-manager-lifecycle-automation](addon-manager-lifecycle-automation/README.md):
  completed Addon Manager control-plane lane for the first manager-owned
  registry/plan slot, covering Addon Health Check, token/grant summaries,
  Addon Install Guide output, and operator-confirmed install/update/remove
  intent without collapsing marketplace, package signing, provider breadth,
  rollback/update execution, or direct process/container supervision into the
  first slice.
- [addon-source-catalog-marketplace](addon-source-catalog-marketplace/README.md):
  completed implementation lane for the addon source catalog and marketplace
  discovery boundary, covering the built-in official source, browse metadata,
  and resolution of installable addon descriptors without collapsing package
  signing, provider breadth, rollback/update execution, authenticated outbound
  task dispatch credentials, official-addon task smoke, or process supervision
  into the core runtime mainline.
- [addon-task-runtime-contract](addon-task-runtime-contract/README.md):
  completed implementation lane for the host-owned Addon Task runtime
  boundary, covering sidecar-claimed execution, direct task-path dispatch,
  progress, results, cancellation, and retry without collapsing source catalog,
  package signing, provider breadth, or process supervision into the first
  slice.
- [addon-outbound-task-dispatch-credentials](addon-outbound-task-dispatch-credentials/README.md):
  completed follow-on lane for authenticated outbound task-dispatch credential
  storage and resolution for `AddonAuth::Bearer` and
  `AddonAuth::SharedSecret`, keeping direct task dispatch host-owned and
  redaction-safe.
- [ai-assisted-library-ops](ai-assisted-library-ops/README.md):
  completed post-RPD mainline lane for Generated Artifact proposal/readiness,
  Admin-only redacted diagnostics, and explicit accept/reject planning for
  AI-like outputs without autonomous canonical metadata, sidecar, library-file,
  Public Client API, or protocol churn.
- [network-access-boundary](network-access-boundary/README.md):
  completed post-RPD mainline lane for self-hosted remote access policy,
  trusted proxy/header handling, tunnel-provider readiness, origin constraints,
  and redacted Admin diagnostics without built-in NAT traversal runtime.
- [downloads-watch-folder-intake](downloads-watch-folder-intake/README.md):
  completed post-RPD mainline lane for acquisition intake and watch-folder
  candidate discovery that feeds Managed Import artifacts and accepted
  promotion/apply workflows without protocol-specific downloader runtime,
  direct library writes, network traversal, AI writes, or Addon runtime scope.
- [playback-transcode-ops-hardening](playback-transcode-ops-hardening/README.md):
  completed post-RPD mainline lane for hardening Playback Runtime readiness,
  typed hardware/fallback diagnostics, transcode validation, session failure
  taxonomy, and bounded Admin support evidence without changing metadata,
  NFO, import, downloader, network, AI, or addon mutation boundaries.
- [link-apply-and-import-promotion](link-apply-and-import-promotion/README.md):
  completed post-RPD mainline follow-on for turning Managed Import promotion
  previews into operator-confirmed, idempotent, rollback-aware Media Library
  mutations through VFS/storage and durable audit records. NFO sidecar mutation
  is split out.
- [nfo-sidecar-promotion-apply](nfo-sidecar-promotion-apply/README.md):
  completed follow-on split for accepted NFO import/export sidecar mutation as
  a Library File Write and metadata-authority workflow, with backup,
  round-trip, rollback/repair, field-lock, hierarchy-confirmation, and audit
  boundaries.
- [managed-import-staging](managed-import-staging/README.md):
  completed post-RPD mainline lane for Nako-owned quarantine, diagnostics, and
  explicit non-mutating promotion planning before downloads, watch-folder
  candidates, or Addon-proposed artifacts can write into a Media Library.
- [nfo-link-authority](nfo-link-authority/README.md):
  completed post-RPD mainline lane for making NFO/local metadata authority,
  non-destructive link planning, and Source Duplicate Relationship evidence
  explicit before managed import/download or addon file-write breadth.
- [metadata-provider-breadth](metadata-provider-breadth/README.md):
  completed Wave 1 execution lane for capability-aware and explainable TMDB,
  Douban, and Bangumi metadata matching, non-destructive ambiguous refresh, and
  cross-provider candidate review before NFO/link, import, AI, or addon
  metadata breadth depends on provider authority.
- [generated-sdk-runtime-ownership](generated-sdk-runtime-ownership/README.md):
  active planning lane for deciding whether Public Client API runtime
  responsibilities such as HTTP execution, public error parsing, API-version
  header checks, request preview redaction, and transport failure mapping remain
  Android-owned, move into a narrow generated SDK/runtime seam, or should be
  pulled forward into a shared Rust client core / UniFFI target state without
  moving Android UI, Media3, token storage, product diagnostics, SDK publishing,
  KMP, or full-platform Rust/UniFFI migration ownership.
- [generated-sdk-forward-compat-tolerance](generated-sdk-forward-compat-tolerance/README.md):
  completed follow-on for deciding and implementing generated Public Client SDK
  unknown string-enum and API-version tolerance without moving Android UI,
  Media3 playback, diagnostics, SDK publishing, KMP, or Rust/UniFFI ownership.
- [android-generated-public-client-sdk](android-generated-public-client-sdk/README.md):
  completed lane for replacing Android handwritten Public Client API DTO and route
  mirrors with an OpenAPI-backed generated Kotlin/JVM SDK before mobile
  Rust/UniFFI is introduced.
- [admin-addon-operations-mvp](admin-addon-operations-mvp/README.md):
  completed productization lane for Addon lifecycle mutation, unregister
  semantics, Addon Health Check, hosted Addon surface read models, and
  redaction-safe resource-call diagnostics without expanding into Addon Manager
  scope.
- [admin-web-addon-operations](admin-web-addon-operations/README.md):
  completed Addon productization lane for turning completed Admin Addon
  Operations API capabilities into a live-capable Admin Web Console Addons
  surface without expanding into Addon Manager install/update or sidecar
  process supervision.
- [addon-architecture-deepening](addon-architecture-deepening/README.md):
  completed architecture-first lane for deepening Addon Side Effect runtime,
  fingerprinted idempotency, Protected Write payload contracts, Addon Manifest
  declarations, Library File Write runtime, Admin Addon API DTO shielding, and
  Addon persistence parity before broader Addon breadth hardens shallow
  Interfaces.
- [self-hosted-release-readiness](self-hosted-release-readiness/README.md):
  completed release hardening lane for turning Nako's completed server/runtime
  capabilities into a repeatable self-hosted baseline with SQLite/PostgreSQL
  gates, API/SDK/redaction checks, deployment examples, backup/restore
  runbooks, and end-to-end smoke evidence.
- [release-packaging-and-distribution](release-packaging-and-distribution/README.md):
  completed release hardening lane for turning the verified self-hosted baseline
  into operator-facing artifacts, container/compose packaging, startup/config
  preflight, release scripts, checksums, and install/upgrade documentation.
- [fearless-architecture-deepening](fearless-architecture-deepening/README.md):
  completed M63 architecture-first lane for deepening Addon Side Effect
  Modules, Addon metadata commit atomicity, Library ingestion workflow seams,
  playback/transcode request identity, hardware diagnostics, search semantics,
  and test locality before new feature breadth hardens shallow Interfaces.
- [postgresql-production-readiness](postgresql-production-readiness/README.md):
  completed M62 execution lane that turned PostgreSQL from the M61 job-lease
  proof into a production-shaped backend through backend-neutral contracts,
  migration/schema parity, runtime backend selection, SQLite assumption
  cleanup, and repeatable verification.
- [managed-artwork-postgresql-parity](managed-artwork-postgresql-parity/README.md):
  completed follow-on split from M62 PGR-090 for PostgreSQL parity across
  Managed Artwork candidates, ingest jobs, artifacts, Selected Artwork,
  galleries, lifecycle cleanup, drift/remediation diagnostics, thumbnail
  variants, and redaction-safe runtime enablement.
- [future-ready-architecture-refactor](future-ready-architecture-refactor/README.md):
  completed M61 fearless architecture refactor lane for PostgreSQL-ready
  persistence, deeper runtime/domain/search/API seams, and deletion of
  redundant MVP paths before Nako's SQLite, metadata, Addon, AI automation,
  and client contracts harden.
- [android-client-foundation](android-client-foundation/README.md): completed
  Android-first client foundation work, covering native Android implementation
  order, playback-first mobile scope, Public Client API connection/browse/search
  loops, playback decision/request construction, Media3 playback smoke,
  playback session boundary, and follow-on API gaps under ADR 0026.
- [android-client-qa-harness](android-client-qa-harness/README.md): completed
  Android client testing lane for local emulator smoke checks, screenshot
  evidence, repeatable fixture/state assumptions, and developer-friendly QA
  commands for parallel Android work.
- [android-developer-validation-entrypoint](android-developer-validation-entrypoint/README.md):
  closed Android developer validation lane for one local handoff command that
  composes JVM tests, debug assemble, and smoke regression evidence.
- [android-smoke-regression-harness](android-smoke-regression-harness/DESIGN.md):
  closed local Android smoke regression lane for composing stable emulator
  fixture states and preserving failure handoff evidence.
- [android-material-expressive-ui](android-material-expressive-ui/README.md):
  completed Android UI rewrite lane for the V2 Material 3 Expressive direction,
  covering dark-first dynamic color, artwork-led media surfaces, restrained
  motion, adaptive phone/tablet chrome, and clean Compose UI boundaries.
- [android-api-contract-integration](android-api-contract-integration/DESIGN.md):
  closed Android Public Client API integration lane for productizing and
  smoke-proving Person Detail from Cast & Crew, with broad relationship indexes
  split to `android-relationship-indexes`.
- [android-relationship-indexes](android-relationship-indexes/DESIGN.md):
  closed follow-on that accepted Genres as the first relationship index, split
  Tags into `android-tags-index`, and deferred a top-level People index until
  role/search semantics are stronger.
- [architecture-review-followups](architecture-review-followups/README.md):
  completed planning and routing lane for the 2026-05-18 architecture review
  findings, covering metadata/catalog atomicity, metadata merge-policy
  unification, Media Library source-of-truth, Public Client Source Locator
  redaction, Addon side-effect seams, playback request identity, and transcode
  diagnostics follow-ups.
- [core-architecture-deepening](core-architecture-deepening/README.md):
  completed architecture-first execution lane for the fearless refactor across
  NFO import atomicity, Library scan source commits, workflow-port narrowing,
  playback/transcode profile identity, hardware capability diagnostics, Addon
  Sidecar alignment, and deletion of replaced shallow paths.
- [metadata-catalog-commit-atomicity](metadata-catalog-commit-atomicity/README.md):
  completed execution lane for deepening metadata/catalog commit consistency,
  starting with an atomic Catalog Item Graph and Search Projection commit before
  deciding whether to fold the broader metadata refresh unit of work into the
  same lane.
- [metadata-merge-policy-unification](metadata-merge-policy-unification/README.md):
  completed execution lane for unifying Canonical Metadata merge authority across
  NFO import, provider refresh, and hierarchy confirmation while keeping NFO XML
  preservation and provider breadth out of scope.
- [multi-library-hardening](multi-library-hardening/README.md): completed
  execution lane for hardening Media Library config/database source of truth,
  startup reconciliation, and removal of remaining one-library authority
  shortcuts after the M8 correctness baseline.
- [public-client-source-locator-redaction](public-client-source-locator-redaction/README.md):
  completed Public Client API follow-up for auditing and removing or redacting
  raw Source Locator exposure from protocol DTOs, OpenAPI, SDKs, and HTTP docs
  while preserving internal storage/playback locators.
- [addon-token-grants-side-effects](addon-token-grants-side-effects/README.md):
  completed ARF-006 follow-up for Addon Token issuance, rotation,
  Library-Scoped Addon Grants, and Nako-mediated Addon Side Effect intake before
  metadata, artwork, subtitle, or Library File Write behavior is enabled.
- [addon-protected-writes](addon-protected-writes/README.md):
  completed follow-on split from the Addon Token Grants Side Effects closeout,
  proving concrete Nako-owned Canonical Metadata `metadata_write` application
  with explicit apply outcome, Addon metadata source attribution, idempotency,
  redaction, and catalog/search refresh.
- [addon-managed-artwork-artifacts](addon-managed-artwork-artifacts/README.md):
  completed follow-on for the first safe `artwork_write` runtime path, covering
  MediaItem-targeted Addon Artwork Candidate proposals without exposing raw
  source URLs as public client artwork.
- [managed-artwork-ingest-selection](managed-artwork-ingest-selection/README.md):
  completed follow-on for accepting Artwork Candidates into internal
  Nako-managed ingest state through a redacted Admin API command and durable
  `managed_artwork_ingest` job, without public artwork publication.
- [managed-artwork-fetch-artifact-storage](managed-artwork-fetch-artifact-storage/README.md):
  completed follow-on for processing queued managed artwork ingest jobs through
  Nako-owned fetch/content validation and internal artifact byte storage before
  public image serving, thumbnails, or selected artwork publication.
- [managed-artwork-public-serving-selection](managed-artwork-public-serving-selection/README.md):
  completed follow-on for publishing stored Managed Artwork Artifacts as Selected
  Artwork and exposing first-party Public Client image references without
  leaking raw source URLs, cache URIs, storage URIs, or local paths.
- [managed-artwork-artifact-lifecycle-cleanup](managed-artwork-artifact-lifecycle-cleanup/README.md):
  completed follow-on for Managed Artwork Artifact lifecycle diagnostics,
  orphan cleanup dry-run, Selected Artwork retention protection, and protected
  cleanup without leaking storage URIs or local paths.
- [managed-artwork-artifact-store-drift-inventory](managed-artwork-artifact-store-drift-inventory/README.md):
  completed follow-on for bounded, redacted Admin diagnostics of drift between
  active Managed Artwork Artifact DB records and files under the local artifact
  root, without deletion or repair.
- [managed-artwork-remediation-policy](managed-artwork-remediation-policy/README.md):
  completed follow-on for redacted Managed Artwork remediation planning and
  confirmed cleanup of safe untracked artifact files, without missing-artifact
  repair or Selected Artwork management.
- [managed-artwork-thumbnail-variants](managed-artwork-thumbnail-variants/README.md):
  completed follow-on for bounded on-demand Selected Artwork image variants,
  redacted public/Admin variant contracts, and cache validators that do not
  expose artifact storage or content hashes.
- [managed-artwork-gallery-candidate-management](managed-artwork-gallery-candidate-management/README.md):
  completed follow-on for redacted Admin item-scoped artwork galleries, candidate
  comparison, and explicit Selected Artwork management without exposing raw
  candidate sources, storage handles, paths, cache URIs, or content hashes.
- [selected-artwork-unpublish-delete-policy](selected-artwork-unpublish-delete-policy/README.md):
  completed follow-on for explicit Selected Artwork unpublish behavior, Public
  Client image visibility after unpublish, and artifact retention/delete
  boundaries without exposing storage handles, paths, source URLs, cache URIs,
  or content hashes.
- [managed-artwork-ingest-runtime-controls](managed-artwork-ingest-runtime-controls/README.md):
  completed follow-on for redacted Admin retry/requeue controls around Managed
  Artwork ingest failures without conflating fetch execution, publication,
  cleanup, repair, or cancellation.
- [managed-artwork-module-deepening](managed-artwork-module-deepening/README.md):
  completed architecture follow-on for deepening Managed Artwork app/db/api
  Modules around candidates, artifacts, Selected Artwork, variants,
  lifecycle/remediation, and redaction-preserving seams without adding provider
  search, Public Client gallery, thumbnail eviction, repair/re-ingest, or new
  runtime retry/cancel semantics.
- [job-runtime-worker-control-plane](job-runtime-worker-control-plane/README.md):
  completed architecture follow-on for the first durable job worker/control-plane
  slice, covering opt-in supervised Managed Artwork ingest execution and typed
  startup recovery while splitting cancellation, generic leases, retry/backoff,
  and other job-kind migrations.
- [durable-job-ownership-leases](durable-job-ownership-leases/README.md):
  completed architecture follow-on for durable job worker identity, fenced
  ownership leases, heartbeats, cancel-request semantics, lease-aware startup
  recovery, shared leased runtime execution, and truthful redacted Admin
  cancellation requests.
- [worker-job-cancellation-checkpoints](worker-job-cancellation-checkpoints/README.md):
  completed follow-on for turning durable running-job cancel requests into
  cooperative worker-side cancellation checkpoints and fenced terminal
  acknowledgement across runtime, metadata maintenance, library scan/probe, and
  NFO app boundaries while splitting retry/backoff, lease stealing, child
  process cancellation, and per-sidecar NFO checkpoints.
- [nfo-sidecar-cancellation-checkpoints](nfo-sidecar-cancellation-checkpoints/README.md):
  completed follow-on for adding per-sidecar cooperative cancellation to NFO
  import/export service loops without making `nako-nfo` depend on server
  runtime types or mixing retry/backoff, lease policy, or child-process
  cancellation into the NFO boundary.
- [addon-library-file-write-policy](addon-library-file-write-policy/README.md):
  completed follow-on for the first addon-initiated Library File Write path,
  proving MediaSource-targeted Nako-owned NFO Export through Nako target
  derivation, storage/VFS, backup policy, redacted write reports, and
  idempotent replay.
- [admin-catalog-governance-read-model](admin-catalog-governance-read-model/README.md):
  completed M60 Admin API read-model work, covering a redacted catalog
  governance queue for unknown and low-confidence Media Items without changing
  the Public Client API.
- [admin-operations-read-models](admin-operations-read-models/README.md):
  completed M57-M59 Admin API read-model batch, covering redacted event outbox
  list/filter, storage staging/cache diagnostics, and sanitized server config
  diagnostics without changing the Public Client API.
- [admin-playback-runtime-diagnostics](admin-playback-runtime-diagnostics/README.md):
  completed M56 Admin API read-model work, covering safe playback runtime
  diagnostics for hardware acceleration policy/selection, FFmpeg capability
  evidence, transcode budgets, remote playback budgets, and staging cleanup
  configuration without changing the Public Client API.
- [admin-playback-session-read-model](admin-playback-session-read-model/README.md):
  completed M55 Admin API read-model work, covering safe playback session
  list/filter support for the web console without exposing transcode output
  paths or changing the Public Client API.
- [durable-job-runtime-admin-read-model](durable-job-runtime-admin-read-model/README.md):
  completed M54 server-side architecture work, covering durable job lifecycle
  centralization and the first Admin API v1 Jobs/Tasks read model.
- [nfo-backup-retention-diagnostics](nfo-backup-retention-diagnostics/README.md):
  completed M50 NFO backup retention and diagnostics work, covering bounded
  keep-latest pruning for local NFO sidecar backups, internal/admin backup
  diagnostics, and public client protocol boundary protection.
- [nfo-sidecar-backup-policy](nfo-sidecar-backup-policy/README.md): completed M49
  NFO sidecar backup policy work, covering same-directory local backup before
  forced sidecar overwrite, explicit VFS backup requests, internal backup
  diagnostics, and separation between XML preservation and storage persistence
  mechanics.
- [nfo-storage-write-policy](nfo-storage-write-policy/README.md): completed M48
  NFO storage write policy work, covering local atomic sidecar writes, explicit
  VFS write modes, internal NFO export diagnostics, and separation between XML
  preservation and storage persistence mechanics.
- [admin-web-console](admin-web-console/README.md): completed web admin console
  baseline, covering Nako's administration-first web surface, media governance
  page families, Admin API implications, brand direction, the `apps/admin-web`
  scaffold, and the live/mock Admin API data-source boundary.
- [admin-api-typescript-contract](admin-api-typescript-contract/README.md):
  completed follow-on for generating or mechanically synchronizing the
  `/admin/v1/*` TypeScript contract consumed by `apps/admin-web` while keeping
  it separate from the Public Client SDK and `nako-client-protocol`.
- [nfo-round-trip-preservation](nfo-round-trip-preservation/README.md):
  completed M47 NFO Round Trip preservation work, covering preservation-aware
  movie NFO update, unknown XML field retention, conflict reporting, forced
  export over existing sidecars, and import/export round trip preservation
  before VFS file write/link policy work.
- [catalog-hydration-lookup-deepening](catalog-hydration-lookup-deepening/README.md):
  completed M42 catalog hydration seam work, covering a workflow-level
  `CatalogHydrationPort`, hidden lookup internals, and narrower metadata/NFO
  test surfaces without public API or schema changes.
- [durable-job-recovery](durable-job-recovery/README.md): completed M41 durable
  job recovery work, covering startup recovery for unfinished queued/running
  jobs, server startup reporting, and removal of an unused old catalog search
  projection seam.
- [metadata-refresh-seam](metadata-refresh-seam/README.md): completed M40
  metadata refresh seam work, covering refresh workflow ports, provider runtime
  boundary review, fake-port behavior tests, and preservation of existing
  provider behavior.
- [repository-seam-deepening](repository-seam-deepening/README.md): completed M39
  repository seam work, covering `CatalogHydrationPort`, catalog hydration
  snapshot/lookup/commit behavior, and metadata/NFO caller-bound narrowing.
- [server-runtime-deepening](server-runtime-deepening/README.md): completed M38
  startup/runtime architecture work, covering `ServerStartupWorkflow`, startup
  reports, durable job runtime supervision, and first migration of library scan
  and metadata background jobs.
- [client-cli](client-cli/README.md): completed M37 client entrypoint work,
  covering the Apache-2.0 Rust client CLI, `nako-client` consumption, public
  API command scope, streaming request construction, token redaction, and
  dependency boundaries that keep AGPL server/internal crates out of clients.
- [client-sdk-contract](client-sdk-contract/README.md): completed M36 client
  SDK contract work, covering protocol-owned public route inventory,
  TypeScript/OpenAPI/Rust SDK inventory reuse, Apache-2.0 client boundary
  preservation, and Rust SDK streaming request builders.
- [rust-client-sdk](rust-client-sdk/README.md): completed M35 Rust client SDK
  foundation, covering the Apache-2.0 `nako-client` crate, protocol DTO reuse,
  clean dependency boundary, async JSON client methods, mock transport tests,
  route inventory checks, and SDK docs.
- [typescript-sdk-package](typescript-sdk-package/README.md): completed M34
  TypeScript SDK package hardening, covering the private `sdk/typescript`
  package, local TypeScript tooling, strict compile gate, repeatable generation
  command, and Rust generator/package sync test.
- [sdk-client-scaffold](sdk-client-scaffold/README.md): completed M33 SDK
  generation and client integration scaffold, covering a dependency-free
  TypeScript/Web/CLI SDK generator, auth/error/version handling, public route
  method inventory, and static leakage checks.
- [openapi-client-contract](openapi-client-contract/README.md): completed M32
  OpenAPI and Public Client SDK contract foundation, covering the first public
  OpenAPI v1 artifact, bearer-auth/error/version schema, route inventory, and
  leakage checks for future Flutter, web, CLI, and SDK work.
- [access-boundary-auth](access-boundary-auth/README.md): completed M31
  inbound HTTP access-boundary work, covering bearer-token auth, public/admin
  route protection, local development config, and separation from addon/
  webhook/provider outbound integration secrets.
- [public-api-contract](public-api-contract/README.md): completed M30 public
  API versioning and error envelope hardening, covering public v1
  compatibility, stable error code vocabulary, pagination/envelope rules, and
  public route evidence for future Flutter, web, CLI, and SDK clients.
- [public-client-api](public-client-api/README.md): completed M29 public
  client API contract work, covering the permissive protocol DTO expansion,
  browse/search/list/detail wire contracts, and playback decision response
  migration for future Flutter, web, and CLI clients.
- [crate-boundary-hardening](crate-boundary-hardening/README.md): completed
  M28 crate boundary and public protocol hardening, covering the permissive
  public client protocol boundary, core/module deepening, library/NFO workflow
  splits, and playback seam clarification.
- [metadata-catalog](metadata-catalog/README.md): M27 media-library domain
  expansion, covering the completed video-first domain baseline,
  schema/repository slice, local inference, provisional hierarchy,
  provider/NFO expansion, and metadata authority.
- [transcode-runtime](transcode-runtime/README.md): completed M25 playback and
  transcode runtime productization, covering playback service decomposition,
  FFmpeg-backed hardware capability probing, acceleration selection, resource
  budgets, session lifecycle, and client-facing playback contracts.
- [server-architecture-hardening](server-architecture-hardening/README.md):
  completed M24 server composition, application service, runtime supervisor,
  repository boundary, and obsolete-helper cleanup work.
- [runtime-foundation](runtime-foundation/README.md): completed M15-M19 database and
  runtime hardening, covering SQLite concurrency, migration execution, secret
  redaction, hardware capability selection, and cross-cutting operational
  boundaries.
- [playback-streaming](playback-streaming/README.md): completed M7 remote
  direct-body streaming, staging budget/cleanup, playback error mapping,
  remote playback resource budgets, and multi-library configuration work.
- [metadata-operations](metadata-operations/README.md): completed M13-M18 metadata
  maintenance jobs, diagnostics filtering, raw cache retention, and provider
  health visibility.
- [storage-vfs](storage-vfs/README.md): completed M6 remote storage, VFS cache,
  remote staging, playback policy, and WebDAV preview work.
- [addons-automation](addons-automation/README.md): completed M5 webhook,
  automation, addon manifest, provider, and trust-boundary work.
- [server-foundation](server-foundation/README.md): completed backend
  foundation, catalog, metadata, playback, transcode, VFS, and historical
  planning hub.

## When To Split A Workstream

Split a workstream when one of these becomes true:

- it has independent milestones that can progress without blocking the active
  backend foundation;
- it needs its own ADR cluster or validation matrix;
- its TODO file becomes too broad to guide the next implementation goal;
- the same docs are repeatedly edited for unrelated domains.

Expected future splits:

- SDK package publishing, client streaming/download helpers, Dart/Flutter SDK,
  Rust CLI, or concrete Flutter/web app work after the public protocol and
  first TypeScript/Rust SDK foundations stabilize.
- Admin Web V2 item artwork gallery/selection after generated Admin contract
  coverage for item artwork routes: `admin-web-v2-item-artwork-selection`.
- Admin Web V2 catalog repair/actions after safe catalog item detail, Provider
  Mapping, and Local Inference evidence semantics are defined:
  `admin-web-v2-catalog-repair-actions` (opened as the active lane).
- Admin Web V2 safe metadata diagnostics, item NFO status/actions, playback
  support detail, settings mutation, users/permissions/Library Access, and
  full-site i18n as separate lanes rather than extensions of the media
  browse/detail read slice.

Keep unsplit domains in `server-foundation` until a split reduces real
coordination cost. Avoid splitting merely because a domain exists conceptually.

## Standard Files

A substantial workstream should have:

- `README.md`: purpose, status, goals, non-goals, links to active phases.
- `MILESTONES.md`: ordered outcomes with deliverables and exit criteria.
- `TODO.md`: task-level checklist grouped by subsystem.
- `PHASE*.md`: phase-specific design and validation notes when needed.
