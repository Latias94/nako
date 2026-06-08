# Jellyfin remote image selection comparison

## Reference Observed

- Local reference: `repo-ref/jellyfin/Jellyfin.Api/Controllers/RemoteImageController.cs`.
- Relevant behavior: `DownloadRemoteImage` is an elevated `POST` action scoped to an item ID, image type, and optional image URL. The server resolves the item, delegates image saving to provider/runtime code, updates repository state, and returns `204`.

## Takeaways For Nako

- Jellyfin treats remote image selection/download as an explicit privileged action, not as an arbitrary frontend-managed storage write.
- Nako should keep candidate acceptance similarly server-owned: Admin Web submits only the candidate ID, and Nako decides how to queue/fetch/store the Managed Artwork.
- Nako's separation is more conservative than Jellyfin's direct save path because accept queues ingestion and does not immediately publish selected artwork.

## Mapping To This Task

- Generating `artwork/candidates/{candidate_id}/accept` is appropriate once the response is typed and redaction-safe.
- The route should not accept raw remote URLs, cache URIs, storage paths, addon tokens, or artifact file details from Admin Web.
- This task uses Jellyfin only as architectural comparison; no Jellyfin source, comments, schemas, or tests are copied.
