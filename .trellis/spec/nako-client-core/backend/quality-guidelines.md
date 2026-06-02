# Quality Guidelines

Core client changes must stay deterministic, transport-neutral, and redaction
safe.

## Required Patterns

- Assert exact request IDs, methods, URLs, headers, bodies, and safe previews.
- Test percent encoding for spaces, slashes, colons, commas, and segment names.
- Test auth header insertion and existing-header behavior when changed.
- Test streaming builders separately from authenticated JSON builders.
- Test response interpretation with public error body redaction.

## Forbidden Patterns

- Do not add network calls, async runtime requirements, or reqwest types.
- Do not rely on global state.
- Do not skip safe preview assertions.
- Do not add a route builder without a focused URL test.

## Tests Required

- Connection probe state tests.
- Generic request builder tests.
- Browse/search/artwork request tests.
- Playback target and HLS segment request tests.
- User playback read/write request tests.
- Response interpreter tests.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-client-core --no-fail-fast`
- Consumer compile:
  `cargo check -p nako-client -p nako-client-uniffi -p nako-client-cli --tests`
