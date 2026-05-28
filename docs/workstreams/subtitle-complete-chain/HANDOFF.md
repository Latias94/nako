# Subtitle Complete Chain Handoff

Status: active.

Current task: SCC-030.

Completed:

- SCC-010 opened the durable workstream.
- ADR 0051 records host-owned subtitle import and rejects direct addon file
  writes.
- SCC-020 moved subtitle search wire types and schema constants into
  `nako-addon-protocol`; official catalog subtitle schema facts now reference
  the shared constants.

Next:

- Make `nako-subtitle-provider` use the shared protocol types and constants.

Watch points:

- Do not implement subtitle file writes in this lane.
- Do not put target paths, Source Locators, storage handles, or write policies
  into provider candidate payloads.
- Keep official provider fixture-backed until provider breadth is split.
