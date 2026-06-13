# Playback Support Evidence Plan

## Status

Shipped. The Admin Web route, navigation links, data source wiring, mock data, and route tests now exist for `Playback Support Evidence`.

This note is retained as predecessor context for the redacted incident bundle follow-on. Do not reopen this as the main task unless regression evidence proves the shipped route is broken.

## Why this slice mattered

Nako already has a backend and API contract for playback support evidence, but Admin Web does not yet give operators a dedicated place to inspect it. That makes the feature useful in tests and server code but awkward in the actual operator journey.

## Shipped slice

Add a read-only Admin Web view for `Playback Support Evidence`, linked from the existing item support card or playback/session context.

## Existing contract

* Server route: `GET /admin/v1/playback/support`
* Query: `session_id?` and `source_id?`
* Response: `AdminPlaybackSupportEvidenceResponse`
* Admin Web client: `getPlaybackSupport()` already exists

## What the view should show

* Support subject
* Session evidence, if present
* Source evidence, if present
* Runtime diagnostics
* Redaction summary

## What it should not show

* Raw paths
* Source locators
* Tokens or credentials
* FFmpeg command text
* Backend URLs or provider payloads
* Unbounded log blobs or generic incident packaging

## Why this is one-day sized

* The backend contract already exists.
* The Admin Web client already has a typed call.
* The main work is route wiring, projection, and tests.

## Implementation shape used

* Add a dedicated Admin Web route/page or item-detail support tile.
* Bind it to the existing `getPlaybackSupport()` data source call.
* Render the redacted evidence blocks with the current validation-oriented patterns.
* Add route and rendering tests that reject unsafe fields.

## Follow-up

The next supportability slice is a broader JSON-only redacted incident bundle export, not another playback support evidence view.
