# Addon Resource Link Check Product Flow

Status: Closed
Last updated: 2026-05-28

This lane added the first host-owned product route for checking a selected
resource-search link by opaque ids. The browser submits `addon_id`, `search_id`,
`selection_id`, and a `refresh` flag only; Nako retrieves the selected raw link
from its resource-search session store and calls the addon's declared
`resource_link_check` resource.

The lane deliberately did not add Admin UI, checker providers, downloader
execution, cloud-drive transfer, or durable password/code persistence. Those
remain separate product and addon lanes.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `MILESTONES.md`
- `HANDOFF.md`
- `CLOSEOUT.md`
