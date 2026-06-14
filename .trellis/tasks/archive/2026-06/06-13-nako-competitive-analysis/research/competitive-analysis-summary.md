# Nako competitive analysis summary

Date: 2026-06-13

## One-Line Read

Nako should compete first on self-hosted operator trust: transparent metadata,
explainable playback, repairable library state, and safe out-of-process Addons.
Jellyfin is the open/self-hosted capability benchmark; Plex is the polished
client and remote-access benchmark.

## Competitive Map

| Dimension | Jellyfin | Plex | Nako current direction |
| --- | --- | --- | --- |
| Product identity | Free software media system; community/open control | Commercial personal media ecosystem with broad client polish and account-mediated convenience | Open-source, self-hosted media server backend; video-first technical preview |
| Maturity | Mature server and clients | Mature server/client ecosystem | Alpha backend/control-plane foundation |
| Core library | Broad media library expectations | Strong personal media organization and client consumption | Video-first Media Library, Media Source, Media Item model |
| Metadata | Provider and local metadata/NFO expectations | Scanner/agent matching, online metadata, NFO support | Provider-neutral Canonical Metadata, Provider Mapping, Local Inference, NFO Round Trip |
| Playback | Direct Play/Remux/Transcode, hardware acceleration, self-hosted networking | Direct Play/Direct Stream/Transcode, strong device/remote story | Typed Direct Play/Remux/Transcode planning, capability profiles, hardware inventory |
| Remote access | Operator-owned network setup | Account-mediated remote access and router automation | Remote Access Endpoint model; reverse proxy/VPN/tunnel readiness, no built-in relay |
| Extension model | Plugin catalog and server-integrated plugin template | Legacy/manual plugin docs; modern first-party extension story limited | Addon Sidecars over HTTP with scoped tokens, grants, tasks, events, side effects |
| Operator diagnostics | Mature but broad/self-hosted admin surface | Polished but more account/product-controlled | Control-plane diagnostics, durable jobs, redaction, repair-first Admin direction |

## Where Nako Should Copy Expectations

Nako should treat these as table stakes:

* library setup, scan, and rescan;
* naming/local inference that users can understand;
* movie/series/anime video-first browsing;
* local metadata and NFO import/export;
* provider metadata and artwork;
* Direct Play/Remux/Transcode;
* hardware acceleration policy;
* remote access guidance;
* admin diagnostics;
* extension/addon ecosystem story;
* Docker/Compose and backup/upgrade docs.

## Where Nako Should Deliberately Diverge

Nako should not copy these directly:

* Jellyfin plugin compatibility or in-process plugin ABI.
* Plex account-mediated remote access as a core dependency.
* Black-box metadata matching without durable evidence and repair.
* Provider-specific domain models inside core media identity.
* One-click addon lifecycle before trust, signature, rollback, logs, and
  side-effect boundaries are ready.

## Strongest Differentiators

1. Addon Sidecars instead of native plugins.
   * Safer failure boundary.
   * Versioned Addon Protocol.
   * Scoped Addon Tokens and grants.
   * Nako-owned side-effect APIs.

2. Metadata governance.
   * Provider Mapping and Candidate Review can become a visible operator
     workflow.
   * NFO/local authority is not a fallback hack; it is product language.

3. Playback explainability.
   * Nako's planner can expose why Direct Play, Remux, or Transcode was chosen.
   * Hardware, HDR/audio/subtitle, source, and client facts can be turned into
     operator-visible remediation.

4. Control-plane repairability.
   * Durable jobs, queue pressure, redacted diagnostics, VFS repair, source
     hash, and Addon task/event boundaries give Nako a credible large-library
     operations story.

## Largest Gaps

1. Product surface maturity.
   * Nako is still an alpha technical preview.
   * Jellyfin/Plex have mature daily-use client experiences.

2. Client ecosystem.
   * Plex and Jellyfin are experienced through clients.
   * Nako's backend can be strong while still feeling incomplete without web,
     mobile, TV, casting, and player polish.

3. Addon install ergonomics.
   * The Addon Protocol/catalog is strong.
   * Operators still need sidecar deployment steps.

4. Remote access polish.
   * Nako's explicit endpoint model is architecturally sound.
   * Plex sets user expectations for easy remote playback.

5. Feature breadth.
   * Live TV/DVR, music/photo maturity, broad TV clients, sharing, and
     ecosystem integrations are not M1 strengths yet.

## Recommended Roadmap Implications

### M1: Do Not Chase Broad Parity

The M1 promise should be narrow and excellent:

* one operator;
* one real video library;
* scan/index;
* browse;
* play;
* diagnose;
* repair.

This is more valuable than claiming partial parity across too many surfaces.

### M2-M3: Make Reliability and Playback Feel Productized

After M1:

* watcher/incremental scan;
* large-library scheduling;
* source identity/hash repair;
* playback mode explanation;
* device profiles;
* subtitle/HDR/audio policy;
* player error recovery.

### M4: Turn Metadata Governance Into the Flagship

Nako can beat both competitors for transparent metadata operations if it makes
these visible:

* local vs NFO vs provider vs addon evidence;
* Candidate Review queue;
* Provider Mapping;
* hierarchy confirmation/repair;
* undo/audit for metadata mutations.

### M5: Make Addons Feel Installable Without Weakening Boundaries

Do not rush native plugins. Instead:

* improve official catalog UX;
* generate clear install guides;
* register/health-check/grant/rotate tokens smoothly;
* keep hosted pages and diagnostics isolated;
* add Addon Manager only when process lifecycle policy is ready.

## Product Narrative

Recommended public narrative:

> Nako is a self-hosted, video-first media server for people who want their
> library to stay understandable: local files remain authoritative, metadata
> changes are reviewable, playback decisions are explainable, and extensions run
> safely outside the server process.

This narrative is more defensible than "Rust Jellyfin" because it maps to the
architecture already present in the repo.

## Research References

* `research/nako-current-positioning.md`
* `research/jellyfin-plex-competitive-landscape.md`
* `README.md`
* `CONTEXT.md`
* `docs/ARCHITECTURE.md`
* `docs/ROADMAP.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `docs/addons/OFFICIAL_ADDON_CATALOG.md`
* `../nako-official-addons/README.md`
* Jellyfin official docs: https://jellyfin.org/docs/
* Plex official support: https://support.plex.tv/
