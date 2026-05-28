# Web Deferred Product Reentry Matrix

Status: Active
Last updated: 2026-05-28

| Deferred surface | Product decision | Existing related work | Frontend reentry condition | First executable slice |
| --- | --- | --- | --- | --- |
| Downloads | Admin operation first, not Media client chrome. Downloads enter through Acquisition Intake / Managed Import until a downloader provider protocol exists. | `downloads-watch-folder-intake`, `admin-web-v2-acquisition-intake-route`, `managed-import-staging`, `web-admin-acquisition-intake` | New `web/` Admin can read acquisition candidates and managed import state through generated Admin contracts. WDRP-030 opened the implementation lane. | Continue `web-admin-acquisition-intake` at WAAI-020, then port a read-only route into `web/src/features/admin` with route-owned query state and fixture/live data source. |
| Playlists | User-owned Media feature, but not before a Public Client playlist contract exists. | No dedicated playlist workstream found. Related: `user-playback-state-contract`, `identity-and-library-access-contract`. | Backend owns playlist identity, item membership, Library Access filtering, and mutation policy. | Open `user-playlists-contract-and-web-slice` before adding UI. |
| Photos | Future non-video media domain. Not part of current video-first live product. | ADR-0021 video-first media server domain model. | Image domain metadata, browse facets, thumbnail/variant policy, and permission behavior are accepted. | Open `non-video-media-domain-baseline` only when photo support is pulled forward. |
| Music | Future non-video media domain. Not part of current video-first live product. | ADR-0021 video-first media server domain model. | Audio domain model for albums, artists, tracks, playback queue, and scan metadata is accepted. | Same `non-video-media-domain-baseline`; do not add music UI first. |
| Podcasts | Future non-video media domain plus subscription/download behavior. | ADR-0021 video-first media server domain model. | Podcast feed/subscription domain, episode identity, acquisition policy, and progress state are accepted. | Same `non-video-media-domain-baseline`; podcast UI is later than music/photo baseline. |
| AI assistant | Admin review workflow, not a free-form Media chat panel. | `ai-assisted-library-ops`, `admin-web-v2-automation-generated-artifacts-route` | Generated Artifact proposals, review plans, and accepted model/provider configuration are available to `web/`. | Port Generated Artifacts review/readiness route into the new `web/` Admin surface. |
| Automation | Admin diagnostics and guarded actions, not Media sidebar chrome. | `addons-automation`, `ai-assisted-library-ops`, `admin-web-v2-automation-generated-artifacts-route` | Addon/Automation task, event, and Generated Artifact contracts are available through Admin API. | Port Automation / Generated Artifact diagnostics into `web/` with mutation guards. |

## Recommended Order

1. Media Web live playback and library browse parity.
2. New `web/` Admin Acquisition Intake route.
3. New `web/` Admin Generated Artifacts / Automation route.
4. User Playlists backend contract and first Media slice.
5. Non-video media domain baseline for photos/music/podcasts.
