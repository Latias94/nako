# Handoff

Status: closed.

Completed:

- Add the first Nako first-class `resource_link_check` addon contract.
- Keep it read-only and separate from search, download, cloud-drive transfer,
  and password/code persistence.
- Do not add Admin UI or product routes in this lane.

Follow-ons:

- Server/product route that consumes an opaque selected-link reference.
- Official or third-party checker addon implementation.
- Admin UI remains intentionally out of scope.
- Downloader, cloud-drive transfer, and password/code persistence remain
  separate contracts.

Watch points:

- Browser APIs must not submit raw URLs/passwords.
- Addon protocol request DTOs may carry host-owned selected links.
- Response DTOs must remain safe to expose after adaptation.
- Do not reuse `acquisition_search_read`.
