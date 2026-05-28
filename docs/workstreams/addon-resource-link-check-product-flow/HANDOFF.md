# Addon Resource Link Check Product Flow - Handoff

Status: closed.

Completed:

- Added a product API route for checking a resource-search selected link by opaque
  ids.
- Kept the request body raw-link-free: `refresh` is the only accepted field.
- Returned safe facts only from the Admin API response.
- Refreshed generated Admin API TypeScript contracts for both retained frontend
  consumers.
- Verified targeted server/API gates and focused cargo check.

Follow-ons:

- Admin UI for invoking link checks from acquisition/search flows.
- Real checker addon/provider implementations.
- Downloader, cloud-drive transfer, and password/code persistence lanes.
