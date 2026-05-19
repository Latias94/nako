# Android Presentation Runtime Adapters - Milestones

Status: Closed
Last updated: 2026-05-20

## M0 - Lane Opened

Exit criteria:

- problem and target state are written;
- scope is separated from player lifecycle/session ownership;
- task ledger and evidence plan are present.

## M1 - Artwork Runtime Adapter

Exit criteria:

- Browse shell no longer reads the token only to construct Home/Libraries artwork requests;
- artwork request creation has a small testable runtime adapter;
- existing artwork fallback and request behavior are preserved.

## M2 - Detail Presentation Contract

Exit criteria:

- detail route visual APIs do not accept raw access tokens;
- detail hero/poster/backdrop artwork still renders through existing artwork components;
- source selection, playback decision, and facet callbacks are unchanged.

## M3 - Player Route Runtime Renderer

Exit criteria:

- Browse shell no longer depends on the concrete player route dependency list;
- player launch rendering has a narrow interface from Browse's point of view;
- no Media3 or exit-effect rewrite is performed in this lane.

## M4 - Closeout

Exit criteria:

- final JVM tests pass;
- `git diff --check` passes;
- documents record shipped behavior and residual follow-ons;
- workstream status is closed or split with an explicit next lane.
