# Library Metadata Scan Policy

Status: Closed
Last updated: 2026-05-25

This workstream turns `MetadataProfile` from mostly descriptive library
configuration into the scan-time metadata acquisition plan for a Media Library.

The first implementation slice keeps behavior narrow: library scans should be
able to run local NFO import automatically when the library profile says NFO is
an enabled local reader. Provider refresh, Addon bulk metadata scrape, embedded
readers, image fetching, and richer priority controls are planned as explicit
follow-ons rather than hidden inside scan.

## Closeout

Closed on 2026-05-25. The shipped slice adds a profile-derived scan acquisition
plan, runs NFO Import after index/probe when enabled, exposes the metadata
summary in scan output, and verifies local plus SMB real-directory playback
smoke.

## Links

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
