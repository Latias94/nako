# Subtitle Complete Chain Handoff

Status: active.

Current task: SCC-020.

Completed:

- SCC-010 opened the durable workstream.
- ADR 0051 records host-owned subtitle import and rejects direct addon file
  writes.

Next:

- Move subtitle search wire structs and schema constants into
  `nako-addon-protocol`.
- Update official catalog facts to use the shared constants.

Watch points:

- Do not implement subtitle file writes in this lane.
- Do not put target paths, Source Locators, storage handles, or write policies
  into provider candidate payloads.
- Keep official provider fixture-backed until provider breadth is split.
