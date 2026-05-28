# Web Media Live Public Client Parity - Route/API Readiness

Status: Active
Last updated: 2026-05-28

| Web route / surface | Current new `web/` state | Public Client readiness | Reentry decision |
| --- | --- | --- | --- |
| `/media` home rails | Fixture/live list seam exists through `listItems`; rails still use broad local categories. | `NakoClient.listItems`; continue-watching route exists in SDK. | WMLP-020 audits exact SDK DTOs, then WMLP-030 wires live home rails with truthful empty/error states. |
| `/media/search` | Live search seam exists through `searchItems`; route owns `q`. | `NakoClient.searchItems` exists. | WMLP-030 strengthens search fallback and route-state tests. |
| `/media/detail` | Live `getItem` mapping exists in data source, but detail UI still relies on local fixture shape. | `NakoClient.getItem` exists; source/version and management context data need audit. | WMLP-030 adds detail read model mapping and explicit missing-field states. |
| `/media/library` | Route owns id/view/sort/filter, but data source does not expose library-scoped browse. | Public Client route inventory needs verification for library item filters or route. | WMLP-020 records whether existing SDK supports this; otherwise WMLP-030 keeps readiness gap visible. |
| Player entry | Local `VideoPlayer` mock exists. | SDK includes playback decision, browser-ticket, playback session, heartbeat, stream, and user playback-state methods. | WMLP-040 wires browser-ticket playback entry when route contract is verified. |
| Continue watching | Local fixture cards exist. | SDK includes `continueWatching`, item playback state, progress, and watched writes. | WMLP-040 wires read/write state after playback entry has stable session identity. |
| Tauri desktop | Tauri build passes; playback remains browser-path only. | Native desktop playback is a follow-on from `media-web-client-foundation`. | WMLP closeout verifies Tauri build but does not implement native playback. |

## Known Follow-Ons

- Desktop native playback capability matrix.
- Management Context Links.
- Invitations/account onboarding.
- Local-media recommendations.
- User playlists backend contract.

