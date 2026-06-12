# Incident Bundle Plan

## Why this slice

Nako already has safe diagnostics spread across multiple Admin surfaces, but it does not yet package them into a single operator support artifact. Jellyfin-style operator maturity leans heavily on system info, logs, scheduled tasks, backup/restore, and explicit maintenance surfaces. In Nako, the equivalent supportable gap is a redacted incident bundle.

## Recommended slice

Build a JSON-only incident bundle export with one Admin API route and one Admin Web read-only projection.

## Include

* System/config summary
* Network / endpoint posture
* Playback readiness and policy readiness
* Storage / VFS repair state
* Durable job queue pressure

## Exclude

* Raw paths, locators, tokens, credentials, backend URLs, provider payloads, FFmpeg commands, and raw job payloads
* Zip packaging
* Upload/sharing workflow
* Backup/restore execution

## Why this is one-day sized

* The required facts already exist as safe summaries.
* The slice stays inside Admin API / Admin Web.
* It does not require schema changes or a new durable subsystem.

## Implementation shape

* Add a redacted Admin DTO that aggregates existing safe diagnostics.
* Add a thin server handler that composes those diagnostics.
* Add a validation-oriented Admin Web view or action to fetch and inspect the bundle.
* Add redaction tests for every sensitive field family already guarded elsewhere.

## Redaction rules

* No raw filesystem identity
* No raw storage identity
* No raw provider payloads
* No FFmpeg command text
* No bearer tokens or credentials
* No backend URLs or query strings
* No raw job input, summary, or error blobs

## Follow-up

If this slice lands cleanly, the next supportability follow-up can decide whether to add:

* a downloadable archive wrapper
* log excerpts with bounded retention
* upload/share transport

