# Quality Guidelines

SDK changes must preserve mockability, public protocol alignment, and redaction.

## Required Patterns

- Use `MockTransport` tests instead of live network tests.
- Assert method, URL, headers, body, and decoded DTO for new JSON methods.
- Assert API version header behavior.
- Assert streaming builders return requests and do not call transport.
- Assert Cargo manifest does not gain server/internal dependencies.

## Forbidden Patterns

- Do not add untested route methods.
- Do not use real Nako server instances in unit tests.
- Do not log or print bearer tokens in SDK behavior.
- Do not add dependencies on server-side crates.

## Tests Required

- Health/login/current-user auth behavior tests.
- Browse/search/playback/user playback/playlist method tests when touched.
- Streaming direct/head/remux/HLS request builder tests.
- Error mapping tests.
- Manifest dependency boundary test.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-client --no-fail-fast`
- Client stack:
  `cargo nextest run -p nako-client-protocol -p nako-client-core -p nako-client --no-fail-fast`
