# Subtitle Search Candidate Selection

Status: Complete
Last updated: 2026-05-28

## Problem

Nako has a shared read-only subtitle provider protocol, but the host still lacks
a product endpoint that calls `AddonResource::Subtitle`, returns safe candidate
cards, and records the user's selected candidate without leaking provider
payload authority to clients.

Without a host-owned selection reference, the later subtitle import plan would
have to trust browser-submitted subtitle content, download URLs, artifact ids,
or target paths. That would break the boundary accepted in ADR 0051.

## Target State

- `nako-addon-client` exposes a typed subtitle search helper that validates the
  subtitle schema and `subtitle_read` grant before HTTP.
- Admin addon APIs can search subtitle providers and return redaction-safe
  subtitle candidate summaries.
- Admin addon APIs can record an opaque selected subtitle candidate reference
  from a short-lived host session.
- The selected reference contains only host-owned identifiers and safe metadata
  in responses; raw inline text, download URLs, provider tokens, Source
  Locators, artifact payload details, and filesystem paths never cross back to
  the browser.
- This lane stops before import planning and Library File Write apply.

## Scope

- `docs/workstreams/subtitle-search-candidate-selection`
- `crates/nako-addon-client`
- `crates/nako-api`
- `crates/nako-server`

## Non-Goals

- No subtitle sidecar file writes.
- No subtitle download execution.
- No import plan preview or apply endpoint.
- No Library File Write integration in this slice.
- No playback subtitle rendering, HLS subtitle renditions, or burn-in.
- No frontend route work; the user's concurrent web route changes are out of
  scope.

## Architecture Direction

Mirror the proven resource-search product shape, but narrow the subtitle
surface:

1. `nako-addon-client` owns typed addon subtitle calls and schema validation.
2. `AddonAppService` owns search sessions and opaque selection ids.
3. Admin DTOs expose candidate metadata needed for choice: title, language,
   format, source, release, score, delivery kind, and safe reference fields.
4. The app stores raw provider candidates only in short-lived server memory so
   later host stages can consume the candidate without trusting browser echoes.
5. Selection response returns a `SubtitleSelectionRef` and candidate summary,
   not provider delivery payloads.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Subtitle payload leaks to Admin clients. | High | DTOs expose only delivery kind and fingerprints/booleans. |
| Generic resource calls drift from protocol constants. | Medium | Add typed client helper with request/response schema checks. |
| Selection endpoint implies import/write semantics. | Medium | Name it selected reference, not import/apply; keep request empty. |
| User web changes get committed accidentally. | High | Stage and commit only paths touched by this workstream. |

## Validation Strategy

- `cargo nextest run -p nako-addon-client subtitle --no-fail-fast`
- `cargo nextest run -p nako-api subtitle --no-fail-fast`
- `cargo nextest run -p nako-server addon_subtitle --no-fail-fast`
- `cargo check -p nako-addon-client -p nako-api -p nako-server --tests`
- `cargo fmt --all -- --check`
- Path-scoped `git diff --check` for touched files, because unrelated web files
  may already be dirty.
