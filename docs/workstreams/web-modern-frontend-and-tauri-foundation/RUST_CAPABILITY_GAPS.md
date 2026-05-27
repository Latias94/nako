# Rust Capability Gaps Against A Jellyfin-Class Target

Status: Active
Last updated: 2026-05-28

This inventory prevents the new `web/` frontend and Tauri shell from faking
server capabilities that Nako has not accepted, implemented, or verified yet.
It is not a promise to copy Jellyfin's model. Jellyfin remains a workflow and
capability reference; Nako keeps its own domain model, API boundaries, Addon
boundary, playback policy, and redaction rules.

## Gap Inventory

| Area | Current Nako evidence | Gap to track | Boundary rule | Follow-on |
| --- | --- | --- | --- | --- |
| Desktop native playback core | `web/src-tauri` now packages the product frontend shell. ADR 0026 rejects WebView-only flagship playback. | No Rust/native desktop player core for broad codecs, subtitle rendering, audio output, HDR, frame stepping, local hotkeys, or native diagnostics. | Tauri WebView is a convenience tier only. Native player ownership must be split and proven before claiming desktop playback quality. | Desktop native playback spike. |
| Credential/session UX | ADR 0037 accepts local credentials and opaque sessions; Public Client SDK exposes auth/session routes. | New `web/` has no real login, server selection, account switching, invitation redeem, or current-principal state. | Do not create frontend-only users, roles, or library access. | Credential/session UX route. |
| Management Context Links | Public Client API route inventory includes management context link concepts; CSAPA keeps route matrix open. | Media-to-Admin and Admin-to-Media links are not yet implemented in the new route shell with role and Library Access gates. | Links carry stable IDs only: no local paths, raw Source Locators, provider payloads, or secrets. | Management Context Links route matrix. |
| Remote access and tunnel provider UX | `network-access-boundary` records remote access policy/readiness. | Release frontend does not yet expose operator-grade tunnel provider setup, origin policy, trusted proxy review, or remote playback risk controls. | Admin-only controls; redacted diagnostics; no implicit public exposure. | Network/remote access frontend lane. |
| Playback capability depth | Playback/transcode policy and browser tickets exist; new web watch route is only a shell. | Rich device profiles, subtitle/audio track UX, HDR policy, renderer target capability negotiation, and native desktop playback diagnostics remain incomplete in product clients. | Playback planner owns policy and reasons; clients render evidence without inventing capability. | Media Web playback depth and native desktop playback. |
| Casting protocols | Renderer/casting lanes exist, with protocol-specific adapters still being split and deepened. | Chromecast, DLNA, AirPlay, and Nako remote renderer breadth need productized setup, diagnostics, and transport evidence before release claims. | Keep cast transport target-scoped and expiring; protocol adapters stay outside core playback policy. | External casting adapter lanes. |
| Addon Manager lifecycle | Addon protocol, sidecar, grants, install-guide, and lifecycle planning exist across completed lanes. | Release frontend still lacks a coherent install/update/remove/rollback UI, marketplace trust UX, signing policy, and sidecar lifecycle supervision story. | Use `Addon` terminology; do not load untrusted hosted pages as privileged UI. | Addon Manager product frontend lane. |
| Library file/link operations | NFO sidecar write policy, Library File Write policy, and managed import promotion lanes exist. | Product UX for soft/hard link management, safe bulk relinking, rollback, dry-run comparison, and operator review remains incomplete. | All file writes stay VFS/storage-owned and confirmation-backed. | Library file/link management lane. |
| Acquisition/download intake | Watch-folder intake and managed import staging exist as backend concepts. | Release frontend has no truthful acquisition queue, downloader integration, import review, or conflict resolution workflow. | Keep acquisition outside library mutation until accepted promotion/apply. | Acquisition/downloads product lane. |
| Backup/restore and disaster recovery | Release/readiness docs and storage diagnostics exist, but no full server backup product surface is accepted. | Jellyfin-class self-hosting needs operator backup/restore, config export, database migration, media artifact retention, and restore drills. | Server owns backup boundaries; frontend renders evidence and explicit operations only. | Self-hosted operations lane. |
| Live TV/DVR | No accepted Nako release scope for Live TV/DVR. | Jellyfin has Live TV/DVR breadth; Nako should not imply it until product scope, storage, scheduler, tuner, EPG, and playback contracts are accepted. | Hide or omit Live TV UI until an ADR/workstream accepts the domain. | Deferred product decision. |

## Frontend Rule

The new frontend may show capability gaps and readiness states, but it must not
render working controls unless the Rust/server boundary has a real API,
redaction policy, validation evidence, and ownership model.
