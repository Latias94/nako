# Browser Playback Auth Transport - Milestones

Status: Active
Last updated: 2026-05-26

## M0 - Transport Decision

Exit criteria:

- Transport options are compared against browser, security, and playback
  constraints.
- The accepted transport is documented.
- ADR impact is decided.

## M1 - Public Contract And SDK

Exit criteria:

- Public OpenAPI exposes the accepted transport issuance contract.
- TypeScript SDK is current.
- Contract redaction and boundary checks are explicit.

## M2 - Server Validation And Stream Use

Exit criteria:

- Protected stream/remux/HLS requests validate the accepted transport.
- Expiry, source scope, mode scope, Library Access denial, Range handling, and
  redaction are tested.

## M3 - Media Web Real Player

Exit criteria:

- `/media/watch/:itemId` renders a real player for the accepted transport.
- Source selection and playback decision state remain URL-owned and Public
  Client API-backed.
- No bearer token, raw locator, or privileged permanent URL is rendered.

## M4 - Playback Progress Writes

Exit criteria:

- Real playback events write progress through User Playback State.
- Writes are throttled and source-aware.
- End-of-play watched behavior is covered.

## M5 - Closeout

Exit criteria:

- Relevant Rust gates pass.
- `cd apps/admin-web && npm run check && npm run test && npm run build` passes.
- Browser desktop/mobile smoke is recorded.
- Desktop native playback, credential/session UX, subtitles, and advanced
  codec work are split or deferred.
- `WORKSTREAM.json` status is updated.
