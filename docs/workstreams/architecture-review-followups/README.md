# Architecture Review Follow-Ups

Status: Completed
Last updated: 2026-05-18

This lane tracked architecture review findings that were too broad to leave in
a chat transcript but not yet ready to become one implementation workstream.

It is closed after routing each finding to an existing workstream, a focused
follow-up workstream, an explicit deferral, or a completed lane.

## Current Finding Queue

1. Metadata refresh, catalog graph hydration, and search projection need a
   clearer atomicity story.
2. NFO import merge behavior and provider refresh merge behavior should share
   one metadata authority policy.
3. Server application services still expose broad persistence/configuration
   knowledge in places where deeper workflow interfaces would improve
   locality.
4. Media Library configuration and persisted Library records need an explicit
   post-startup source of truth.
5. Public Client DTOs should be checked for Source Locator leakage before
   remote access, multi-user access, and network traversal mature.
6. Addon side effects need token, grant, audit, and Nako-mediated effect
   seams before powerful addons are enabled.
7. HLS request identity and future Transcode Profile shape need to account for
   quality, client capability, subtitles, audio selection, and hardware policy.
8. Hardware acceleration diagnostics should eventually prove planned encode
   viability, not only encoder-name presence.

## Operating Rule

Future architecture reviews should open a new review follow-up lane or update
the assigned execution lanes directly. Do not implement code directly in this
coordination lane.
