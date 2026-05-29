# Web Deferred Product Reentry Matrix

Status: Active
Last updated: 2026-05-29

| Deferred surface | Product decision | Existing related work | Frontend reentry condition | First executable slice |
| --- | --- | --- | --- | --- |
| Downloads | Admin operation first, not Media client chrome. Downloads enter through Acquisition Intake / Managed Import until a downloader provider protocol exists. | `downloads-watch-folder-intake`, `admin-web-v2-acquisition-intake-route`, `managed-import-staging`, `web-admin-acquisition-intake` | New `web/` Admin can read acquisition candidates and managed import state through generated Admin contracts. `web-admin-acquisition-intake` closed at WAAI-050 with the read-only route shipped. | Next slices are guarded watch-folder discovery mutation controls, downloader provider/protocol planning, or Managed Import promotion/apply UI. |
| Playlists | User-owned Media feature, but not before a Public Client playlist contract exists. WDRP-050 decided prerequisites are ready for a contract lane. | `user-playback-state-contract`, `identity-and-library-access-contract`, `user-playlists-contract-and-web-slice` | Backend owns playlist identity, item membership, Library Access filtering, and mutation policy. UI waits for UPCW-020 contract freeze. | Continue `user-playlists-contract-and-web-slice` at UPCW-020 before adding UI. |
| Photos | Future non-video media domain. Not part of current video-first live product. WDRP-060 keeps it deferred. | ADR-0021 video-first media server domain model; `NON_VIDEO_DOMAIN_DECISION.md` | Image domain metadata, browse facets, thumbnail/variant policy, and permission behavior are accepted. | Open `non-video-media-domain-baseline` only when photo support is pulled forward. |
| Music | Future non-video media domain. Not part of current video-first live product. WDRP-060 keeps it deferred. | ADR-0021 video-first media server domain model; `NON_VIDEO_DOMAIN_DECISION.md` | Audio domain model for albums, artists, tracks, playback queue, and scan metadata is accepted. | Same `non-video-media-domain-baseline`; do not add music UI first. |
| Podcasts | Future non-video media domain plus subscription/download behavior. WDRP-060 keeps it deferred. | ADR-0021 video-first media server domain model; `NON_VIDEO_DOMAIN_DECISION.md` | Podcast feed/subscription domain, episode identity, acquisition policy, and progress state are accepted. | Same `non-video-media-domain-baseline`; podcast UI is later than music/photo baseline. |
| AI assistant | Admin review workflow, not a free-form Media chat panel. | `ai-assisted-library-ops`, `admin-web-v2-automation-generated-artifacts-route`, `web-admin-generated-artifacts-automation` | Generated Artifact proposals are now visible in the new `web/` Admin surface. `web-admin-generated-artifacts-automation` closed at WAGA-050 with the read-only route shipped. | Next slices are guarded Generated Artifact review mutations, Automation Provider adapter breadth, or metadata-authority apply workflow. |
| Automation | Admin diagnostics and guarded actions, not Media sidebar chrome. | `addons-automation`, `ai-assisted-library-ops`, `admin-web-v2-automation-generated-artifacts-route`, `web-admin-generated-artifacts-automation` | Addon/Automation task, event, and Generated Artifact contracts are available through Admin API. `web-admin-generated-artifacts-automation` shipped proposal diagnostics and deferred review mutations. | Open a focused guarded review mutation lane or addon task/event diagnostics lane instead of reopening WAGA. |

## Recommended Order

1. Media Web live playback and library browse parity.
2. New `web/` Admin Acquisition Intake route.
3. New `web/` Admin Generated Artifacts / Automation route.
4. User Playlists backend contract and first Media slice.
5. Public Client follow-on planning from WMLP closeout. Completed by WDRP-065.
