# Selected Artwork Unpublish Delete Policy Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Contract

Exit criteria:

- Workstream docs exist and agree on terminology.
- Unpublish is separated from artifact deletion and physical byte cleanup.
- Route direction and public image behavior are documented.
- `WORKSTREAM.json` parses.

Status: Completed.

## M1 - Admin Unpublish Command

Exit criteria:

- `DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection` exists.
- Existing item with a selected slot returns `changed = true`.
- Existing item with no selected slot returns `changed = false`.
- Missing item and invalid kind are mapped through existing Admin API error
  conventions.
- No Admin response exposes storage locators, local paths, raw source URLs,
  cache URIs, or content hashes.

Status: Completed.

## M2 - Public And Lifecycle Consequences

Exit criteria:

- Public item image responses omit an unpublished slot.
- `GET` and `HEAD /images/{old_selected_id}` return `404` after unpublish.
- The linked Managed Artwork Artifact row and bytes are not deleted by
  unpublish.
- Existing lifecycle cleanup rules are the only path that can later mark/delete
  unselected artifacts.

Status: Completed.

## M3 - Closeout

Exit criteria:

- HTTP docs describe the command and lifecycle policy.
- Focused tests and cargo check evidence are recorded.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Follow-ons are explicit and outside this lane.

Status: Completed.
