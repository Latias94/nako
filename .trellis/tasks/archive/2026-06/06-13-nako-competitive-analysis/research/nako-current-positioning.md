# Research: Nako Current Positioning

- Query: Nako's current product positioning and addon ecosystem using repo docs and the official addons repository; focus on current product promise, what works today, current boundaries, addon catalog capabilities, trust boundary, and likely differentiators vs Jellyfin/Plex explicitly supported by repo docs.
- Scope: mixed
- Date: 2026-06-13

## Findings

### Sources Inspected

- `README.md`: current release status, product promise, "what works today", current boundaries, SDK/package promise.
- `CONTEXT.md`: project glossary and durable product language for Addons, trust boundaries, media scope, and Jellyfin/Plex comparison guardrails.
- `docs/ARCHITECTURE.md`: architecture north star, principles, and maturity map.
- `docs/architecture/CONTROL_PLANE.md`: control-plane status for addon protocol, addon supervision, remote access, and relay boundaries.
- `docs/architecture/LANES.md`: ownership lane for addon protocol, addon client, official catalog, automation, and official addon repository coordination.
- `docs/addons/OFFICIAL_ADDON_CATALOG.md`: generated operator-visible official addon inventory.
- `docs/guides/ADDON_AUTHOR_GUIDE.md`: first HTTP addon contract, registration, health checks, diagnostics, grants, and token model.
- `docs/plans/ADDON_ECOSYSTEM_STRATEGY.md`: proposed addon ecosystem direction, trust tiers, current strengths/gaps.
- `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md`: competitor-oriented Build / Integrate / Defer / Do Not Chase matrix.
- `crates/nako-addon-protocol/README.md` and `crates/nako-addon-protocol/src/lib.rs`: addon protocol version, wire contracts, manifest/descriptor validation, health/task/event envelopes.
- `crates/nako-addon-client/src/lib.rs`: outbound addon call, health, task, event, auth, validation, and safe error helper patterns.
- `crates/nako-official-addon-catalog/README.md` and `crates/nako-official-addon-catalog/src/lib.rs`: generated official addon fact source.
- `crates/nako-server/src/http/addons.rs`, `crates/nako-server/src/app/addons/catalog.rs`, `surfaces.rs`, `routing.rs`, `task_runtime.rs`, `runtime.rs`: Admin Addon routes, built-in catalog, health/runtime readiness, routing plans, task lifecycle, and side-effect authority.
- `F:\SourceCodes\Rust\nako-official-addons\README.md`: official addon repository current release target, inventory, defaults, smoke commands, and boundaries.
- `F:\SourceCodes\Rust\nako-official-addons\crates\*/README.md`: current sidecar-specific docs for metadata scraper, resource search, subtitle provider, Chromecast renderer, DLNA renderer, notification bridge, and external acquisition runner.

### Current Product Promise

Nako's user-facing promise is currently "Your media home, gently kept" (`README.md:16`): an open-source media server backend for people who want their own films, shows, anime, and personal collection on hardware they control. The repo is explicit that Nako is `0.1.0-alpha.2`, a technical preview useful for development, self-hosted testing, and early addon work, not a stable Jellyfin or Plex replacement yet (`README.md:23`, `README.md:25-30`).

The architecture north star is broader and more ambitious than the alpha promise: a self-hosted media server backend aiming at a Jellyfin/Plex-class system that stays self-hostable, inspectable, and easy to extend without copying Jellyfin/Plex internals or adopting a native plugin ABI too early (`docs/ARCHITECTURE.md:5-8`). The glossary also makes the long-term media scope broader than the current video-first phase: video, audio, image, document, mixed, and online media are in the long-term **Media Server Scope**, while the current phase prioritizes movies, series, anime, home video, playback, metadata, and transcode (`CONTEXT.md:115-122`, `CONTEXT.md:494`).

### What Works Today

The current repo docs claim working foundations in these areas:

- Media library scanning with Local Inference, Source State, Media Source records, and Provisional Hierarchy (`README.md:35-36`).
- SQLite and PostgreSQL persistence paths with contract tests (`README.md:37`).
- Local filesystem VFS with WebDAV-oriented pieces (`README.md:38`).
- Metadata provider runtime for TMDB, Bangumi, and Douban (`README.md:39`).
- NFO import/export and local metadata authority (`README.md:40`).
- Managed Import, Nako-managed artwork, Library File Write, and operator-confirmed promotion/apply flows (`README.md:41-42`).
- Playback source selection, remux/transcode planning, hardware acceleration policy, runtime diagnostics, and bounded staging (`README.md:43-44`).
- Admin API and Admin Web Console pages for diagnostics, operations, Addon onboarding, credentials, grants, and runtime status (`README.md:45-46`).
- Addon Sidecar protocol support with scoped tokens, grants, health checks, install guides, and resource-call diagnostics (`README.md:47-48`).
- Docker/Compose examples, config preflight, release packaging, and operator docs (`README.md:49-50`).

The architecture map grades Addons/automation as "Good foundation" with next pressure around marketplace/install guidance, official provider breadth, and external resource actions (`docs/ARCHITECTURE.md:91`). Metadata and playback planning are also described as good first slices or good foundations (`docs/ARCHITECTURE.md:86-87`).

### Current Boundaries

Nako's alpha boundaries are intentionally conservative:

- Addons are externally run Addon Sidecars; Nako does not yet install, update, start, stop, remove, log, or supervise addon processes (`README.md:54-55`).
- The official addon catalog is discovery, compatibility, install references, and smoke status, not an Addon Manager (`docs/addons/OFFICIAL_ADDON_CATALOG.md:5`).
- Nako's built-in catalog source explicitly reports no package signing and no process supervision (`crates/nako-server/src/app/addons/catalog.rs:51-52`).
- Install guide output says Nako does not manage containers, processes, or packages; the operator owns sidecar installation, lifecycle, upgrades, logs, and removal (`crates/nako-server/src/app/addons/catalog.rs:183-187`).
- Network tunnel support is policy/readiness oriented; Nako does not currently run built-in NAT traversal or relay (`README.md:67-68`). The control-plane map likewise marks remote access cookbook as planned and built-in tunnel provider as deferred (`docs/architecture/CONTROL_PLANE.md:50-51`).
- AI-assisted workflows are not part of the `alpha.2` promise (`README.md:69-70`).
- Deployment guidance says keep Nako local-only, private-network, VPN, reverse-proxy, or tunnel-bounded with auth enabled (`README.md:71-72`).

### Addon Ecosystem And Catalog Capabilities

The Addon Protocol version is currently `0.1.0-alpha.1`; Nako accepts only explicitly supported protocol versions during alpha (`README.md:30-31`, `docs/guides/ADDON_AUTHOR_GUIDE.md:26-31`, `crates/nako-addon-protocol/src/lib.rs:8-9`). The protocol crate is permissively licensed for sidecar authors and does not depend on server internals (`crates/nako-addon-protocol/README.md:6-18`).

The generated official catalog currently lists seven official addons, all compatible with `>=0.1.0-alpha.2 <0.2.0`:

- `nako.official.metadata-scraper`: metadata resource, `bulk-metadata-scrape` task, `library.scanned` event, diagnostics page, scopes for metadata read/suggest, automation, and event read, trust tier "official side-effect", local smoke plus Nako-mediated alpha smoke (`docs/addons/OFFICIAL_ADDON_CATALOG.md:13`).
- `nako.official.resource-search`: `resource_search` and `resource_link_check`, read-only acquisition search/link-check scopes, trust tier "official read-only" (`docs/addons/OFFICIAL_ADDON_CATALOG.md:14`).
- `nako.official.subtitle-provider`: read-only `subtitle` resource and `subtitle_read` scope (`docs/addons/OFFICIAL_ADDON_CATALOG.md:15`).
- `nako.official.chromecast-renderer`: `renderer_adapter` resource with read/control scopes and renderer adapter trust tier (`docs/addons/OFFICIAL_ADDON_CATALOG.md:16`).
- `nako.official.dlna-renderer`: `renderer_adapter` resource with read/control scopes and renderer adapter trust tier (`docs/addons/OFFICIAL_ADDON_CATALOG.md:17`).
- `nako.official.notification-bridge`: webhook resource, `library.scanned` event, diagnostics, `webhook_event_read`, notification fan-out trust tier, local and optional live smoke (`docs/addons/OFFICIAL_ADDON_CATALOG.md:18`).
- `nako.official.external-acquisition-runner`: `external-acquisition-action` task, `acquisition_action_run` scope, official side-effect trust tier (`docs/addons/OFFICIAL_ADDON_CATALOG.md:19`).

The official catalog excludes `browser-worker` because it is a browser-render helper rather than an Addon catalog entry (`docs/addons/OFFICIAL_ADDON_CATALOG.md:9`, `crates/nako-official-addon-catalog/src/lib.rs:1443`). Catalog facts are generated from `crates/nako-official-addon-catalog`, which keeps manifest, install descriptors, compatible Nako version, trust tier, smoke status, and install references in one place (`.trellis/spec/nako-official-addon-catalog/backend/index.md:1-29`, `crates/nako-official-addon-catalog/src/lib.rs:1446-1604`).

The official addons repository is currently targeted at `v0.1.0-alpha.2` (`F:\SourceCodes\Rust\nako-official-addons\README.md:5`). Its README says the official sidecars are intentionally small and capability-focused, with the option to package related capabilities as one suite later (`F:\SourceCodes\Rust\nako-official-addons\README.md:10-12`). It documents metadata, notification, Chromecast, DLNA, resource search, external acquisition, and subtitle sidecars (`F:\SourceCodes\Rust\nako-official-addons\README.md:14-33`, `F:\SourceCodes\Rust\nako-official-addons\README.md:56-73`).

Default official addon behavior is smoke-friendly and opt-in for risky/live paths: fixture providers are enabled by default for local smoke, TMDB/Bangumi/Douban and AV metadata providers are disabled unless configured, Chromecast and DLNA are plan-only or manual/live gated by default, resource search fixture mode is deterministic, the external acquisition runner fixture is no-op, and subtitle provider fixture mode never writes subtitle files (`F:\SourceCodes\Rust\nako-official-addons\README.md:99-173`).

### Trust Boundary

The trust boundary is one of Nako's clearest positioning choices. An **Addon** is defined as a user-enabled extension outside the core server trust boundary that gives a Jellyfin-like extensibility experience without Jellyfin plugin compatibility (`CONTEXT.md:9-11`). An **Addon Sidecar** is an independently running process or service, explicitly not an in-process plugin (`CONTEXT.md:35-37`).

Nako preserves host-owned authority around addon behavior:

- An Addon Sidecar may call Nako APIs only with an Addon Token, which is scoped to accepted permissions and library grants (`CONTEXT.md:39-40`, `CONTEXT.md:474-475`).
- Addon event-triggered writes still use an Addon Token (`CONTEXT.md:476`).
- Addon side effects must pass through Nako-owned APIs, permissions, audit, and resource boundaries (`CONTEXT.md:473`).
- Addon external fetches may happen in the sidecar, but a Nako-managed artifact must enter through Nako APIs (`CONTEXT.md:478`).
- Library file writes initiated by addons must go through Nako storage/NFO/artwork/subtitle APIs (`CONTEXT.md:479`).
- Addon hosted pages are not trusted with Nako admin credentials (`CONTEXT.md:488`).

The code matches the docs:

- Admin routes exist for registration, catalog entries, health, runtime readiness, surfaces, routing plans, task runs, install guide, manager plan, diagnostics, resource search, subtitle flow, tokens, and grants (`crates/nako-server/src/http/addons.rs:42-169`).
- Runtime routes are separate, explicit Addon runtime endpoints for access checks, generated artifacts, acquisition candidates, external acquisition materialization, side effects, and task run claim/progress/complete/fail/cancel (`crates/nako-server/src/http/addons.rs:174-207`).
- Runtime readiness validates the manifest, checks grants, secret reference requirements, network policy, protocol, manifest, and safety before treating a sidecar as ready (`crates/nako-server/src/app/addons/surfaces.rs:78-163`).
- Task runs require an enabled addon, a valid manifest, granted task scopes, routing plan readiness, idempotency, and durable `JobKind::AddonTask` persistence (`crates/nako-server/src/app/addons/task_runtime.rs:44-123`).
- Side-effect submission resolves an addon principal from the raw token, authorizes the grant, validates the library/target, journals accepted or rejected validation status, and applies through Nako's router rather than letting the sidecar mutate storage directly (`crates/nako-server/src/app/addons/runtime.rs:58-78`, `crates/nako-server/src/app/addons/runtime.rs:103-192`).
- The addon client validates response envelopes and handles bearer/shared-secret auth headers, health checks, task/event/resource calls, and safe error codes (`crates/nako-addon-client/src/lib.rs:250-375`, `crates/nako-addon-client/src/lib.rs:662-989`, `crates/nako-addon-client/src/lib.rs:1331`).

### Likely Differentiators Vs Jellyfin/Plex Supported By Repo Docs

Only differences explicitly supported by repo docs are included here:

- **Jellyfin-like extensibility without Jellyfin plugin compatibility.** The glossary states addons should feel extensible like Jellyfin but are not Jellyfin Plugin Compatibility and do not use Jellyfin's plugin API/internal object model (`CONTEXT.md:437-438`, `CONTEXT.md:565`, `CONTEXT.md:725-726`).
- **Out-of-process addon sidecars instead of native in-process plugin ABI.** Architecture principles say addons are out-of-process HTTP sidecars with scoped APIs and tokens (`docs/ARCHITECTURE.md:72-73`), and the parity matrix explicitly marks in-process plugin ABI as "Do not chase" because it violates the Addon Sidecar boundary (`docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:67`).
- **Planner-first, policy-first media backend.** The architecture principles emphasize Direct Play first, planner-before-runtime, manifest-backed artifacts, explicit local authority, resource budgets, and control-plane boundaries (`docs/ARCHITECTURE.md:57-75`). This supports a positioning around explainable playback/transcode/addon decisions rather than ad hoc runtime behavior.
- **Local authority and portability as first-class behavior.** NFO, sidecars, user edits, and field locks are documented as first-class self-hosted behavior (`docs/ARCHITECTURE.md:70-71`), and the parity matrix calls NFO/sidecars a Nako differentiation area (`docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:46`).
- **Addon catalog as a strategic differentiator, but not yet mature.** The parity matrix says Plugin/Addons catalog is a build area and that Nako differentiation depends on it, while also noting the current entry is weak (`docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:66`). This is supported by the generated catalog and seven official addons, but the repo still frames marketplace/install UX as next pressure rather than complete product.
- **Self-hosted posture without Plex-style central account or first-party relay.** The parity matrix says Plex-style central account/device brokerage is "Do not chase" and first-party traffic relay is "Do not chase now" (`docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:68-69`). README similarly says no built-in NAT traversal/relay and recommends local/private/VPN/reverse-proxy/tunnel-bounded deployment (`README.md:67-72`).
- **Build + integrate rather than clone all competitor features.** The parity matrix classifies metadata providers, casting, subtitle automation, and acquisition-related workflows as build/integrate surfaces, while explicitly rejecting a raw feature-parity checklist approach (`docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:45`, `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:58`, `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:71`, `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md:117-137`).

### Related Specs

- `.trellis/spec/nako-addon-protocol/backend/index.md`: Addon Protocol is the permissive wire-contract crate for manifests, runtime resources, tasks, events, health, side effects, install guides, validation, and version compatibility.
- `.trellis/spec/nako-addon-client/backend/index.md`: Addon client owns outbound HTTP calls, protocol envelope validation, retry behavior, mockable transport, auth header handling, and redaction-safe errors.
- `.trellis/spec/nako-official-addon-catalog/backend/index.md`: official catalog is source of truth for official addon manifests, install descriptors, trust tier, smoke status, and generated operator artifact; it is not an Addon Manager.
- `.trellis/spec/nako-server/backend/index.md`: server owns Admin routes, auth/access checks, diagnostics, HTTP boundary behavior, and addon resource flow patterns.
- `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`: relevant for host-owned Addon resource search, subtitle import, external acquisition, planning/materialization, and redaction patterns.
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`: relevant to existing Admin Web addon onboarding, Addon task runs, and route/path generation conventions.

### Product Positioning Synthesis

The strongest evidence-backed positioning is:

> Nako is an alpha-stage, self-hosted, video-first media server backend that is building toward Jellyfin/Plex-class media ownership while differentiating on explicit local authority, planner-first playback/control-plane design, and an out-of-process Addon Sidecar ecosystem with scoped tokens, grants, health checks, install guides, diagnostics, and generated official catalog facts.

The ecosystem promise is credible but still alpha: Nako has a concrete Addon Protocol, Admin routes, runtime routes, catalog generation, official sidecars, local smoke paths, and host-owned side-effect gates. The user-facing marketplace/manager experience remains intentionally deferred.

## Caveats / Not Found

- No web browsing was used. Official repository evidence came from the local checkout at `F:\SourceCodes\Rust\nako-official-addons` and repository URLs already present in inspected docs.
- The README says "first alpha companion addon" is `nako-metadata-scraper@0.1.0-alpha.2` (`README.md:56-58`), while the generated official catalog and official addon repository list seven official addon entries. Interpret this as the first repeatable alpha host/addon smoke path being metadata-scraper, not as the full official catalog inventory.
- The research did not verify live behavior by running tests, smoke scripts, servers, or containers. Claims are documentation/code-inspection claims.
- The official addon ecosystem remains alpha. Docs explicitly say public API, Admin API, Addon Protocol, database schema, and generated SDKs may change before beta (`README.md:25-29`).
- Competitive conclusions are limited to repo-supported positioning. This file does not claim current Jellyfin or Plex product facts beyond what Nako's own docs use as comparison categories.
