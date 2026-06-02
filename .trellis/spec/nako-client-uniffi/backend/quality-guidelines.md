# Quality Guidelines

UniFFI changes must stay in lockstep with `nako-client-core`.

## Required Patterns

- Add mirror records/enums when core exposes a new binding-needed type.
- Add `From` conversions for every mirrored input/output type.
- Add exported function tests that assert request IDs, URLs, methods, optional
  preflight requests, and safe previews.
- Keep dependency surface limited to `nako-client-core` and `uniffi`.
- Preserve `cdylib` and `rlib` crate types.

## Forbidden Patterns

- Do not reimplement core builders.
- Do not introduce network, async runtime, or SDK dependencies.
- Do not skip tests for exported functions.
- Do not expose raw tokens through safe preview records.

## Tests Required

- Connection probe export tests.
- Playback target export tests.
- Browse/search export tests.
- Artwork export tests.
- User playback export tests.
- Conversion tests when enum variants or records change.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-client-uniffi --no-fail-fast`
- Core binding:
  `cargo nextest run -p nako-client-core -p nako-client-uniffi --no-fail-fast`
