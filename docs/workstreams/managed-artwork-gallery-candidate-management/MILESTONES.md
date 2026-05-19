# Managed Artwork Gallery Candidate Management Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Open Management Boundary

Exit criteria:

- Workstream docs exist.
- Admin/Public scope split is explicit.
- Candidate, artifact, and selected artwork terminology is preserved.
- Redaction policy and follow-ons are explicit.

Status: Done.

## M1 - Admin Gallery Read Model

Exit criteria:

- Admin can inspect item-scoped artwork choices through one redacted response.
- Response distinguishes candidates, managed artifacts, and selected artwork.
- Response exposes public image refs only for selected/public-safe images.
- Raw candidate source URLs, storage URIs, cache URIs, paths, tokens, and
  content hashes are absent.

Status: Done. `GET /admin/v1/items/{item_id}/artwork` returns the first
redacted Admin gallery read model with candidate, artifact, and selected
artwork sections.

## M2 - Selection Management

Exit criteria:

- Admin can intentionally replace Selected Artwork from an eligible artifact.
- Replacement is scoped to the target item and image kind.
- Public Client item images reflect the new selected public image reference.
- Artifact deletion, file cleanup, unpublish, retry/cancel, and repair remain
  outside the action unless explicitly split and tested.

Status: Done. Admin can select/replace an item artwork slot with
`POST /admin/v1/items/{item_id}/artwork/{kind}/select`, guarded by item and
image kind.

## M3 - Closeout

Exit criteria:

- Focused tests and relevant checks pass.
- HTTP/API docs document the shipped Admin route/action.
- Workstream docs record evidence and follow-ons.
- No raw locator/hash exposure is introduced.

Status: Done. Fresh closeout gates passed and follow-ons are documented.
