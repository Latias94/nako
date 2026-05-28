# Addon Resource Search Product Flow

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

`addon-resource-search-protocol` gave Nako the protocol, typed client,
diagnostic call, and intake handoff seam. The remaining gap is product flow:
an operator still cannot safely run a real resource search, inspect result
cards, and explicitly select one link into acquisition intake through Nako.

The existing diagnostic route intentionally returns only counts and provider
summaries. That is correct for diagnostics, but it is not a user-facing search
flow.

## Problem

Resource-search results contain useful display data and sensitive link data in
the same payload. A product API must let the Admin UI show enough information
to choose a result without echoing raw URLs, passwords, provider exceptions, or
download credentials.

Without a host-owned search-session boundary, the obvious implementation would
either:

- return raw addon links to the browser and ask the browser to submit them back;
- overload the diagnostic route with product behavior;
- or let a read-only addon write acquisition candidates directly.

All three collapse boundaries this repo just separated.

## Target State

When this lane closes:

- Nako has an Admin API route for running a resource search that returns
  display-safe result cards and redacted link summaries.
- Search results are backed by a short-lived host-owned selection store or
  equivalent opaque-selection mechanism.
- The Admin API exposes opaque `search_id` and `selection_id` values for
  explicit selection, not raw link URLs or passwords.
- A second Admin API route converts one selected link into a
  `resource_search_selection` acquisition intake candidate through the existing
  app-service handoff.
- Diagnostics remain diagnostics; product search does not reuse diagnostic
  response DTOs.
- Generated Admin TypeScript contracts stay in sync.

## In Scope

- `nako-api` Admin/addon DTOs for product search and selection.
- `nako-server` app-service and HTTP routes for search sessions and selection.
- Redaction-safe result/link summaries.
- Short-lived in-memory selection storage if no durable store is justified yet.
- Tests proving no raw URLs, passwords, request context, or provider exception
  text are exposed in Admin responses.
- Generated Admin TypeScript contracts.

## Out Of Scope

- `nako-official-addons` manifest migration.
- Admin UI pages and visual design.
- Link availability checks.
- Downloader execution.
- Cloud-drive save or transfer.
- Persistent password/extraction-code storage.
- Any acquisition write scope for read-only search addons.

## Architecture Direction

`nako-server::app::addons` owns the host search call, host limit, addon grant
checks, session storage, and safe response shaping.

`nako-server::app::acquisition_intake` remains the only owner of
`resource_search_selection` candidate recording.

The product search response should expose result titles and non-secret display
metadata, but link data should be summarized as link type/source/redacted ref
and opaque selection IDs. Selection should use the host session, not a raw link
round-trip through the browser.

The first implementation should prefer an in-memory TTL selection store over a
database migration unless a durable replay requirement appears. Search sessions
are transient operator workflow state, not acquisition records.

## Non-Goals

- Search does not imply download.
- Search does not imply cloud-drive save.
- Search result URLs are not playback stream URLs.
- `acquisition_search_read` stays read-only.
- Password/code handling is only represented as `has_password` until a separate
  secret model exists.

## Closeout Condition

This lane can close when:

- product search and explicit selection routes exist and are tested,
- raw link/password/context/provider exception text is not exposed,
- selected links enter acquisition intake through `resource_search_selection`,
- generated Admin TypeScript contracts are refreshed,
- focused gates and broad checks pass,
- and remaining UI/official-addon/download/link-check work is split clearly.
