# Official Addon E2E Alpha2 - Milestones

Status: Completed
Last updated: 2026-05-23

## M0 - Scope Frozen

Exit criteria:

- workstream docs agree on target state and non-goals;
- first executable task is identified;
- cross-repository ownership is explicit.

## M1 - Released Pieces Start

Exit criteria:

- Nako release artifact or image can be started in the alpha smoke;
- `nako-metadata-scraper@0.1.0-alpha.1` can be started in fixture/default mode;
- commands avoid private provider secrets.

## M2 - Host Calls Sidecar

Exit criteria:

- Nako registers the metadata scraper manifest;
- hosted health succeeds;
- at least one hosted resource call succeeds through Nako;
- request/response evidence is redacted.

## M3 - Compatibility Diagnostics

Exit criteria:

- supported Addon Protocol version is accepted;
- unsupported version produces a clear diagnostic;
- docs explain Addon Version versus Addon Protocol Version in the context of the
  official addon.

## M4 - Closeout

Exit criteria:

- final gates are recorded;
- docs match the proven behavior;
- provider breadth and Addon Manager automation are split or deferred.
