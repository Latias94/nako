# Admin Media Management Context Links - Milestones

Status: Closed
Last updated: 2026-05-30

## M0 - Lane Open

Exit criteria:

- DONE. CSAPA-040 is split into this focused workstream.
- DONE. The target frontend is `web/`.
- DONE. Backend `/management/context-links` is treated as existing authority.

## M1 - Route Resolver And Data Source

Exit criteria:

- DONE. Public Client SDK link reads are wrapped behind a web data-source
  boundary.
- DONE. One resolver maps every known backend `route_name`.
- DONE. Unknown routes, disabled links, and unsafe query params have tests.

## M2 - Media-to-Admin Links

Exit criteria:

- DONE. Media library/detail/source/watch contexts render enabled management
  links only from backend state.
- DONE. Ordinary viewer states preserve backend disabled reasons instead of
  inventing access logic.
- DONE. Media Web does not import Admin API DTOs or mutation clients.

## M3 - Admin Command And Return Links

Exit criteria:

- DONE. Admin surfaces can receive link targets and own
  confirmation/mutation flows.
- DONE. Admin-to-Media links use Public Client-visible stable IDs only.
- DONE. Broad or destructive actions stay Admin-owned.

## M4 - Cross-Surface Verification

Exit criteria:

- DONE. Administrator, library manager, and viewer behavior is covered.
- DONE. Redaction tests cover tokens, raw paths, Source Locators, provider payloads,
  FFmpeg details, and storage handles.
- DONE. Browser smoke covers at least one Media-to-Admin and one Admin-to-Media
  transition.

## M5 - Closeout

Exit criteria:

- DONE. Workstream docs and evidence are complete.
- DONE. Remaining scope is split, not left as hidden TODOs.
