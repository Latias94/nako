# Web Deferred Product Reentry Plan - Public Client Follow-Ons

Status: Active
Last updated: 2026-05-28

## WDRP-065 Decision

WMLP closeout produced four concrete follow-ons. WDRP-065 routes them as
follows:

| Follow-on | Decision | Reason |
| --- | --- | --- |
| Browser playback session identity / heartbeat | Open `public-client-browser-playback-session-identity`. | Browser playback works, but heartbeat needs a web-visible session id or equivalent control identity. |
| Library-scoped item browse | Open `public-client-library-browse-query-contract`. | `/media/library` is truthful but cannot show scoped live items without a Public Client route/query contract. |
| Catalog sort/filter for Recently Added and watched filters | Fold into `public-client-library-browse-query-contract`. | Sort/filter and library browse share the same public query vocabulary and access-filtering behavior. |
| Desktop native playback | Defer to the existing Rust/Tauri capability gap. | ADR-0026 rejects WebView-only flagship playback, but native player ownership requires a separate capability lane after browser playback contracts stabilize. |

## Opened Lanes

- `docs/workstreams/public-client-browser-playback-session-identity`
- `docs/workstreams/public-client-library-browse-query-contract`

## Deferred Trigger: Desktop Native Playback

Open a desktop native playback lane when at least these are true:

- browser playback session identity and heartbeat are stable;
- playback capability/profile evidence is deep enough to choose native vs
  WebView behavior honestly;
- the Tauri/Rust boundary has an accepted native player ownership model;
- the lane can test codecs, subtitles, audio output, HDR policy, hotkeys, and
  native diagnostics without pretending the WebView path is enough.

Until then, `web/src-tauri` remains a packaged frontend shell with browser-path
playback, and `web-modern-frontend-and-tauri-foundation/RUST_CAPABILITY_GAPS.md`
continues to track the native playback gap.
