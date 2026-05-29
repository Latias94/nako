# Web Player Subtitle Track Evidence And Gates

| Gate | Command | Result |
| --- | --- | --- |
| Focused playback/data-source tests | `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx` | Passed: 2 files / 17 tests |
| Full web tests | `npm --prefix web run test` | Passed: 8 files / 47 tests |
| TypeScript | `npm --prefix web run check` | Passed |
| Bundle budget | `npm --prefix web run build:budget` | Passed |
| Diff whitespace | `git diff --cached --check` | Passed |

## Evidence Log

- 2026-05-28: Added `loadPlaybackPlan` to the Public Client media data source.
  It fetches item source identity, playback decision, source probe, video
  browser ticket, and one subtitle browser ticket per sidecar subtitle stream.
- 2026-05-28: Added native `<video>` source and `<track>` rendering in
  `VideoPlayer` when browser-ticket URLs are available, while preserving the
  existing mock fallback when no playable URL exists.
- 2026-05-28: Added tests asserting media/subtitle URLs use opaque ticket URLs
  and do not contain the bearer token.
