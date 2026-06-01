# Client Surface And Access Product Architecture Closeout

Status: Closed
Date: 2026-06-01

## Decision

Close `client-surface-and-access-product-architecture`. The broad product
architecture questions have been split or deferred:

- identity and Library Access are owned by `identity-and-library-access-contract`;
- browser Media Web is owned by `media-web-client-foundation` and later `web/`
  follow-ons;
- Management Context Links are owned by
  `admin-media-management-context-links`;
- desktop playback strategy is deferred from the MVP/browser-first path.

## Desktop Decision

Do not keep CSAPA active for desktop playback. When desktop playback becomes a
product priority, open a focused `desktop-tauri-native-playback-spike` or
equivalent lane that compares browser/WebView playback against a native player
core with codec, subtitle, hardware acceleration, packaging, and platform smoke
evidence.

## Gates

- `python -m json.tool docs/workstreams/client-surface-and-access-product-architecture/WORKSTREAM.json`
- `python C:/Users/Frankorz/.codex/skills/plan-engineering-program/scripts/workstream_inventory.py --root .`
- `git diff --check -- docs/workstreams/client-surface-and-access-product-architecture docs/architecture/LANES.md docs/workstreams/README.md`

## Residual Risk

- Desktop/native playback remains unimplemented and intentionally deferred.
- Account UX, mobile/native product work, and broader Web product execution
  require focused follow-on workstreams.
