# Subtitle Complete Chain Handoff

Status: complete.

Current task: none.

Completed:

- SCC-010 opened the durable workstream.
- ADR 0051 records host-owned subtitle import and rejects direct addon file
  writes.
- SCC-020 moved subtitle search wire types and schema constants into
  `nako-addon-protocol`; official catalog subtitle schema facts now reference
  the shared constants.
- SCC-030 migrated `nako-subtitle-provider` to the shared protocol types and
  schema constants in official addons commit `fce9871`.
- SCC-040 recorded host-owned candidate selection, import planning, Library
  File Write apply, refresh/playback visibility, and provider breadth
  follow-ons.
- SCC-050 closed the lane with fresh protocol and official provider gates.

Next:

- Future lanes should implement host candidate selection, subtitle import
  planning, Library File Write apply, refresh/playback visibility, and provider
  breadth in that order.

Watch points:

- Do not implement subtitle file writes in this lane.
- Do not put target paths, Source Locators, storage handles, or write policies
  into provider candidate payloads.
- Keep official provider fixture-backed until provider breadth is split.
