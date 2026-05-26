# Client Surface And Access Product Architecture - Evidence And Gates

Status: Draft
Last updated: 2026-05-26

## Gate Policy

This is a docs-first product architecture lane. It does not claim runtime
behavior until narrower implementation lanes run their own gates.

Required planning gates:

- `git diff --check`
- `python -m json.tool docs/workstreams/client-surface-and-access-product-architecture/WORKSTREAM.json`
- Workstream docs agree on scope, target state, task IDs, and follow-ons.

Implementation follow-ons must choose their own gates:

- Rust identity/access work should use `cargo fmt --all -- --check` and focused
  `cargo nextest run` package filters.
- Admin Web or Media Web work should use package-local check/test/build and
  browser smoke.
- Desktop/Tauri work should include platform smoke evidence and explicit
  playback capability checks.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | CSAPA-010 planning open | Read `CONTEXT.md`, PRODUCT/DESIGN context, ADR 0024/0026/0027/0028, Admin Web V2 docs, Users & Access readiness, Android UX context, Public Client API design, and Jellyfin reference notes. | Draft planning lane opened. |
| 2026-05-26 | CSAPA-010 validation | `python -m json.tool docs/workstreams/client-surface-and-access-product-architecture/WORKSTREAM.json > $null`; `git diff --check -- docs/workstreams/client-surface-and-access-product-architecture docs/workstreams/README.md` | Pass. `git diff --check` emitted the existing CRLF conversion warning for `docs/workstreams/README.md` only. |

## Redaction And Safety Checks

The lane does not introduce routes or UI. Follow-ons must continue to block
these from normal user-facing views:

- bearer tokens;
- password hashes or reset tokens;
- raw local filesystem paths;
- raw Source Locators;
- provider payloads;
- addon tokens or webhook secrets;
- FFmpeg paths, argv, output paths, or raw stderr;
- storage credentials;
- unsafe external URLs containing credentials.
