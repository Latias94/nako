# Research: Jellyfin and Plex competitive landscape

- Query: Research Jellyfin and Plex as official-doc competitors for Nako's planned self-hosted, video-first media server. Focus on product positioning, core media server features, plugin/extension model, metadata model, playback/transcode/remote access, and distinctions relevant to Nako.
- Scope: mixed
- Date: 2026-06-13

## Findings

### Executive Summary

Jellyfin is the open, community-run reference point: a free software media server with broad server features, a plugin catalog, local metadata/NFO support, and direct self-hosted networking expectations. Its competitive lesson for Nako is capability completeness plus operator control.

Plex is the polished commercial reference point: personal media organization and streaming wrapped in first-party clients, account-mediated remote access, sharing, premium features, and a strong "it just works across devices" story. Its competitive lesson for Nako is product smoothness, client coverage, and remote playback diagnostics.

Nako should not clone either internal model. Existing project direction already favors a Rust modular monolith, provider-neutral domain language, HTTP Addon Sidecars instead of in-process plugins, and explicit Remote Access Endpoints rather than a first-party relay. The opportunity is to combine Jellyfin's self-hosted transparency with Plex-like operator ergonomics while staying video-first for M1.

### Files Found

- `CONTEXT.md` - Nako project glossary for Addons, Media Libraries, Canonical Metadata, Playback Runtime, Playback Transcode, and Remote Access Endpoints.
- `docs/architecture/PLAYBACK.md` - Nako playback capability map and target chain for Direct Play, Remux, HLS Transcode, artifacts, tickets, and player follow-ons.
- `docs/architecture/LANES.md` - Current M1 lane routing, including the self-hosted, video-first, single-admin/operator journey.
- `docs/architecture/CONTROL_PLANE.md` - Control-plane map for remote access, endpoint discovery, durable jobs, diagnostics, and relay/tunnel deferral.
- `docs/deployment/REMOTE_ACCESS.md` - Nako remote access cookbook for reverse proxy, VPN/private network, and operator-managed tunnels.
- `docs/adr/0001-modular-monolith-rust-workspace.md` - Rust modular monolith decision for the self-hosted MVP.
- `docs/adr/0003-http-addons-before-in-process-plugins.md` - Nako decision to prefer HTTP Addons before native/in-process plugins.
- `crates/nako-addon-protocol/src/lib.rs` - Addon Protocol wire contract, manifest declarations, resources, hosted pages, events, tasks, and permissions.
- `crates/nako-playback/src/lib.rs` - Playback planner and decision model for Direct Play, Remux, Transcode, and denied playback.
- `crates/nako-playback/src/capability.rs` - Playback capability profiles and compatibility evaluation.
- `crates/nako-transcode/src/*` - HLS artifacts, hardware acceleration inventory, FFmpeg encoder planning, transcode profiles, and runtime limits.
- `crates/nako-core/src/media/*` - Media item/source, canonical metadata, provider mapping, library profile, fingerprint, and local inference records.
- `crates/nako-nfo/src/*` - NFO import/export, document codec, preservation, and authority workflows.

### Product Positioning

Jellyfin official positioning:

- Jellyfin presents itself as a free software media system and "Server Backend & API" in the official GitHub repository.
- The public site emphasizes volunteer-built, no fees, no tracking, and control over personal media.
- This makes Jellyfin the strongest open/self-hosted comparison for Nako's operator trust, privacy, and ownership story.

Plex official positioning:

- Plex's personal media server page positions Plex Media Server as a way to organize, beautify, and stream movies and music.
- Plex combines personal media with a broader commercial ecosystem: Plex account, many client apps, Plex Pass upsells, support articles, and official Docker deployment.
- This makes Plex the strongest comparison for polish, client reach, sharing, and remote access convenience rather than for open server internals.

Nako distinction:

- Nako should present itself as video-first now but not hard-code a video-only scope. The glossary explicitly separates `Media Server Scope` from `Video-First Phase` (`CONTEXT.md:115`, `CONTEXT.md:119`, `CONTEXT.md:494`, `CONTEXT.md:754`).
- M1 should compete on the self-hosted, video-first, single-admin journey: library configuration, scan, browse, playback, diagnostics, and repair (`docs/architecture/LANES.md:23`).

### Core Media Server Features

Common baseline expected by both competitors:

- Library setup and scan/indexing.
- Movie, TV/series, music/photo or broader media libraries.
- Metadata fetching and artwork.
- Local file naming conventions and local metadata escape hatches.
- User/admin settings.
- Browser and device playback.
- Direct Play/Direct Stream or Remux/Transcode decision path.
- Remote access guidance.
- Deployment documentation.

Jellyfin feature implications:

- Jellyfin is broader than video and has a server/admin model with many settings, libraries, users, plugins, and networking controls.
- Nako should treat Jellyfin as the parity expectation for self-hosted server capability breadth, not as an API compatibility target.

Plex feature implications:

- Plex validates that product-level expectations include remote streaming, device/client breadth, sharing, clear server status, and simple setup.
- Nako's early product surface should emphasize operator readiness and playback readiness, not only backend capability existence.

Nako current alignment:

- The M1 queue already centers a single self-hosted operator path through scan, catalog browse, playback, and diagnostics (`docs/architecture/LANES.md:23`).
- The modular monolith ADR keeps initial deployment simple while preserving crate boundaries for scan, metadata, search, playback, transcode, storage, webhooks, automation, and addons (`docs/adr/0001-modular-monolith-rust-workspace.md:9`, `docs/adr/0001-modular-monolith-rust-workspace.md:23`).

### Plugin / Extension Model

Jellyfin:

- Official Jellyfin docs expose a plugin catalog and manual plugin installation path, with plugin categories including authentication, channels, and general plugins.
- The official plugin template documents in-process/server-integrated extension points such as dashboard configuration pages and item comparers.
- Competitive implication: Jellyfin sets the expectation that self-hosted users can extend metadata, auth, channels, notifications, and server behavior.

Plex:

- Official Plex support still documents manual `.bundle` plugin installation.
- The official source found for plugins is legacy/manual-install oriented; no current first-party extension platform comparable to Nako Addon Protocol or Jellyfin's plugin template was found in this pass.
- Competitive implication: Plex is not the right model for Nako extension architecture. It is more useful as a product and client ecosystem comparator.

Nako:

- Nako already rejects Jellyfin Plugin Compatibility as the goal: Addons may offer a Jellyfin-like extensibility experience without implementing Jellyfin's plugin API (`CONTEXT.md:437`, `CONTEXT.md:484`, `CONTEXT.md:565`).
- ADR 0003 explicitly chooses HTTP Addons before native plugins because in-process plugins raise ABI, sandboxing, crash isolation, versioning, and trust concerns (`docs/adr/0003-http-addons-before-in-process-plugins.md:16`, `docs/adr/0003-http-addons-before-in-process-plugins.md:27`).
- The wire contract is already explicit: protocol version constants (`crates/nako-addon-protocol/src/lib.rs:8`), `AddonManifest` (`crates/nako-addon-protocol/src/lib.rs:134`), resources (`crates/nako-addon-protocol/src/lib.rs:257`), hosted pages (`crates/nako-addon-protocol/src/lib.rs:384`), events (`crates/nako-addon-protocol/src/lib.rs:453`), tasks (`crates/nako-addon-protocol/src/lib.rs:483`), and permissions (`crates/nako-addon-protocol/src/lib.rs:1880`).
- Product distinction: Nako can promise extension capability while keeping addon code outside the server process and routing side effects through Nako-owned APIs, grants, audit, and resource limits.

### Metadata Model

Jellyfin:

- Official metadata docs cover provider access and local metadata/NFO behavior.
- Jellyfin reinforces the self-hosted expectation that local sidecar metadata and provider metadata can coexist.

Plex:

- Official Plex scanner docs say scanners inspect configured media locations and determine whether files belong to a library and which movie/episode/etc. they represent, mostly from filenames and directory structure, sometimes with embedded metadata.
- Official Plex NFO docs describe an NFO Agent for populating movie and TV libraries from NFO files, useful for personal media or collections not found in online databases.
- Plex therefore validates file naming discipline, scanner/agent separation, and local metadata escape hatches.

Nako:

- Nako should keep its provider-neutral terms rather than adopting scanner/agent naming wholesale.
- `CanonicalMetadata` is the presentation/search/export authority (`CONTEXT.md:227`), and source facts remain separate from metadata (`CONTEXT.md:529`, `CONTEXT.md:530`).
- Local/NFO/provider precedence is already first-class through `Metadata Source Priority` (`CONTEXT.md:275`, `CONTEXT.md:529`, `CONTEXT.md:673`, `CONTEXT.md:768`).
- Core metadata structures exist in code: `MediaKind` (`crates/nako-core/src/media/item.rs:9`), `CanonicalMetadata` (`crates/nako-core/src/media/item.rs:28`), `ExternalProvider` (`crates/nako-core/src/media/provider.rs:9`), and `ProviderMapping` (`crates/nako-core/src/media/provider.rs:110`).
- Library presets already encode local readers and provider defaults, including NFO and TMDB/Douban/Bangumi combinations (`crates/nako-core/src/media/profile.rs:23`).
- NFO import/export is a real workflow, not a raw file parsing helper (`crates/nako-nfo/src/import.rs:44`, `crates/nako-nfo/src/export.rs:30`, `crates/nako-nfo/src/codec.rs:39`).
- Nako should differentiate by surfacing uncertainty and repair: `MediaSource` (`crates/nako-core/src/media/source.rs:11`), fingerprint evidence/escalation, and `LocalInferenceEvidence` (`crates/nako-core/src/media/source.rs:467`) support more transparent diagnosis than a black-box scanner match.

### Playback, Transcode, and Remote Access

Jellyfin:

- Official transcoding docs describe a low-to-high server-load ladder: Direct Play, Remux, Direct Stream, and Transcode.
- The same docs cover HDR to SDR tone mapping and hardware acceleration as server/operator concerns.
- Official networking docs include default HTTP/HTTPS ports, LAN/remote access settings, user external access permissions, firewall/port-forwarding guidance, and caution that opening a port directly to the internet is insecure and not recommended.

Plex:

- Official Plex streaming docs explain playback from a central server over LAN, internet streaming, and the Direct Play/Direct Stream/transcode decision space.
- Official transcoding docs emphasize device differences, resolution/format compatibility, and CPU cost.
- Official remote access docs require Plex account sign-in and support automatic router configuration through UPnP/NAT-PMP or manual port forwarding to internal port 32400.
- Official hardware-accelerated streaming docs make acceleration a Plex Pass/server capability discussion.

Nako:

- Nako playback already uses the same conceptual ladder but with Nako terms: prefer Direct Play, then Remux, then Transcode (`docs/architecture/PLAYBACK.md:17`, `docs/architecture/PLAYBACK.md:24`).
- `PlaybackPlanner` produces typed decisions (`crates/nako-playback/src/lib.rs:32`, `crates/nako-playback/src/lib.rs:384`, `crates/nako-playback/src/lib.rs:393`), with `PlaybackMode` and transcode requirements (`crates/nako-playback/src/lib.rs:85`, `crates/nako-playback/src/lib.rs:216`).
- Client capability profiles are explicit (`crates/nako-playback/src/lib.rs:270`, `crates/nako-playback/src/capability.rs:144`, `crates/nako-playback/src/capability.rs:355`, `crates/nako-playback/src/capability.rs:368`, `crates/nako-playback/src/capability.rs:375`).
- HLS artifacts, adaptive ladders, sidecar renditions, and burn-in planning are typed (`crates/nako-transcode/src/artifact.rs:222`, `crates/nako-transcode/src/artifact.rs:315`, `crates/nako-transcode/src/artifact.rs:780`, `crates/nako-transcode/src/artifact.rs:961`).
- Transcode profiles and HLS profile construction exist (`crates/nako-transcode/src/profile.rs:227`, `crates/nako-transcode/src/profile.rs:667`), with explicit codec policy (`crates/nako-transcode/src/profile.rs:713`) and hardware inventory/encoder mapping (`crates/nako-transcode/src/hardware.rs:31`, `crates/nako-transcode/src/hardware.rs:430`, `crates/nako-transcode/src/ffmpeg/hls/encoders.rs:84`).
- Remote access should stay explicit and operator-owned. Nako docs say it validates operator-declared policy but does not start tunnel processes, own VPN configuration, or manage DNS (`docs/deployment/REMOTE_ACCESS.md:5`). Control-plane docs defer a built-in tunnel provider and caution against making core a relay service (`docs/architecture/CONTROL_PLANE.md:51`, `docs/architecture/CONTROL_PLANE.md:450`, `docs/architecture/CONTROL_PLANE.md:501`).
- Product distinction: Plex optimizes for account-mediated convenience; Jellyfin expects self-hosted network configuration; Nako should make remote access readiness and diagnostics first-class without owning a relay in M1.

### Competitive Implications for Nako

1. Positioning:
   - Do not position only as "Jellyfin in Rust" or "open Plex".
   - Better: self-hosted, video-first media server for operators who want transparent metadata, playback decisions, repairability, and extension safety.

2. Feature priority:
   - M1 must make the core operator journey feel complete: configure library, scan, inspect provisional matches, browse catalog, play media, diagnose transcode/remote access, and repair common failures.
   - Avoid broad parity claims around music/photo/live TV until those domains are real.

3. Extension strategy:
   - Keep the Addon Sidecar strategy. Jellyfin proves extension demand; Plex does not provide a better modern extension model.
   - Make addon installation and health checks approachable enough to satisfy Jellyfin-like extensibility expectations without loading third-party code into Nako.

4. Metadata strategy:
   - Treat Plex scanners and Jellyfin metadata providers as product evidence that local inference, naming, NFO, provider mappings, and repair UX matter.
   - Nako can differentiate by showing evidence, confidence, source/provider authority, and review/repair status instead of silently rematching.

5. Playback strategy:
   - The Direct Play/Remux/Transcode ladder is table stakes.
   - Nako's sharper opportunity is explainability: why a mode was chosen, what client capability forced it, which source facts were used, and what operator action would improve playback.

6. Remote access strategy:
   - Plex sets the convenience benchmark, but it depends on account sign-in and router automation.
   - Jellyfin sets the self-hosted configuration benchmark.
   - Nako should keep the explicit Remote Access Endpoint model and build config checks, redaction-safe diagnostics, and cookbook quality around it.

## Code Patterns

- Addon protocol is versioned and externalized: `ADDON_PROTOCOL_VERSION` and supported versions are constants (`crates/nako-addon-protocol/src/lib.rs:8`), and manifests declare resources, hosted pages, events, tasks, and permissions (`crates/nako-addon-protocol/src/lib.rs:134`, `crates/nako-addon-protocol/src/lib.rs:257`, `crates/nako-addon-protocol/src/lib.rs:384`, `crates/nako-addon-protocol/src/lib.rs:453`, `crates/nako-addon-protocol/src/lib.rs:483`, `crates/nako-addon-protocol/src/lib.rs:1880`).
- Playback planning is pure and explicit: `PlaybackDecision`, `PlaybackMode`, `ClientPlaybackCapabilities`, and `PlaybackPlanner::plan` are central records/functions (`crates/nako-playback/src/lib.rs:32`, `crates/nako-playback/src/lib.rs:85`, `crates/nako-playback/src/lib.rs:270`, `crates/nako-playback/src/lib.rs:393`).
- Playback capabilities are profile-driven: target, Direct Play, Remux, and Transcode profiles are separate (`crates/nako-playback/src/capability.rs:144`, `crates/nako-playback/src/capability.rs:355`, `crates/nako-playback/src/capability.rs:368`, `crates/nako-playback/src/capability.rs:375`).
- HLS output is manifest-backed and typed: burn-in plans, media renditions, adaptive ladder, and artifact manifest are modeled directly (`crates/nako-transcode/src/artifact.rs:222`, `crates/nako-transcode/src/artifact.rs:315`, `crates/nako-transcode/src/artifact.rs:780`, `crates/nako-transcode/src/artifact.rs:961`).
- Hardware acceleration is an inventory and planning concern, not an ad hoc FFmpeg flag: `HardwareAcceleration`, `HardwareAccelerationReport`, encoder mapping, and runtime inventory/limits are separate records (`crates/nako-transcode/src/hardware.rs:31`, `crates/nako-transcode/src/hardware.rs:430`, `crates/nako-transcode/src/ffmpeg/hls/encoders.rs:84`, `crates/nako-transcode/src/runtime.rs:28`, `crates/nako-transcode/src/runtime.rs:67`).
- Metadata is provider-neutral and authority-aware: `CanonicalMetadata`, `ExternalProvider`, `ProviderMapping`, and NFO workflows exist as independent concepts (`crates/nako-core/src/media/item.rs:28`, `crates/nako-core/src/media/provider.rs:9`, `crates/nako-core/src/media/provider.rs:110`, `crates/nako-nfo/src/import.rs:44`, `crates/nako-nfo/src/export.rs:30`).
- Source identity and local inference are visible product concepts: `MediaSource`, fingerprint matching, and `LocalInferenceEvidence` support repairable scan behavior (`crates/nako-core/src/media/source.rs:11`, `crates/nako-core/src/media/source.rs:380`, `crates/nako-core/src/media/source.rs:467`).

## External References

Official Jellyfin sources:

- Jellyfin website: https://jellyfin.org/
- Jellyfin server repository: https://github.com/jellyfin/jellyfin
- Jellyfin plugin docs: https://jellyfin.org/docs/general/server/plugins/
- Jellyfin plugin template: https://github.com/jellyfin/jellyfin-plugin-template
- Jellyfin metadata docs: https://jellyfin.org/docs/general/server/metadata/
- Jellyfin transcoding docs: https://jellyfin.org/docs/general/post-install/transcoding/
- Jellyfin networking docs: https://jellyfin.org/docs/general/post-install/networking/

Official Plex sources:

- Plex Personal Media Server page: https://www.plex.tv/personal-media-server/
- Plex Media Server Docker repository: https://github.com/plexinc/pms-docker
- Plex Remote Access support: https://support.plex.tv/articles/200289506-remote-access/
- Plex Direct Play / Direct Stream support: https://support.plex.tv/articles/200250387-streaming-media-direct-play-and-direct-stream/
- Plex Transcoding Media support: https://support.plex.tv/articles/200250377-transcoding-media/
- Plex hardware-accelerated streaming support: https://support.plex.tv/articles/115002178853-using-hardware-accelerated-streaming/
- Plex scanners support: https://support.plex.tv/articles/200241548-scanners/
- Plex NFO metadata support: https://support.plex.tv/articles/using-nfo-metadata-files-with-plex/
- Plex manual plugin install support: https://support.plex.tv/articles/201187656-how-do-i-manually-install-a-plugin/
- Plex PMS API docs: https://developer.plex.tv/pms/

## Related Specs

- `.trellis/spec/nako-addon-protocol/backend/index.md` - Addon Protocol boundary, validation, and no native plugin execution.
- `.trellis/spec/nako-playback/backend/index.md` - Pure playback planning boundary and Direct Play/Remux/Transcode decision behavior.
- `.trellis/spec/nako-transcode/backend/index.md` - FFmpeg command planning, HLS artifacts, hardware capability inventory, and runtime limits.
- `.trellis/spec/nako-metadata/backend/index.md` - Provider, strategy, mapping, hierarchy confirmation, and Candidate Review behavior.
- `.trellis/spec/nako-library/backend/index.md` - Scan, source ingestion, probe orchestration, local inference, and failure handling.

## Caveats / Not Found

- Official sources only were used where possible. Community comparisons, forum claims, Reddit, Wikipedia, third-party benchmark posts, and non-official plugin catalogs were intentionally excluded.
- Plex plugin model evidence is limited. The official Plex source found here documents manual `.bundle` plugin installation, but this pass did not find a modern official Plex extension platform comparable to Jellyfin plugins or Nako Addon Sidecars.
- Plex pricing, Plex Pass entitlement boundaries, client app purchase details, and account-service behavior can change. This note treats them only as product-positioning signals, not stable technical contracts.
- Jellyfin and Plex official docs are product/user docs, not full internal architecture specs. Inferences about Nako are mapped through Nako's own ADRs, specs, and code, not assumed from competitor internals.
- Nako should borrow expectations and UX lessons, not terminology wholesale. The project glossary explicitly avoids drifting to provider-centric, file-centric, or Jellyfin plugin compatibility language.
