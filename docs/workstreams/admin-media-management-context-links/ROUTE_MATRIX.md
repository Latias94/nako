# Admin Media Management Context Links - Route Matrix

Status: Draft
Last updated: 2026-05-29

Backend source of truth:

- Public route: `GET /management/context-links`
- SDK method: `NakoClient.managementContextLinks(query)`
- Context keys: `library_id`, `item_id`, `source_id`,
  `playback_session_id`

## Backend Link Names

| `route_name` | Action | Required access | Target context | Frontend destination | Ownership |
| --- | --- | --- | --- | --- | --- |
| `library.scan` | `scan_library` | `library_manage` | `library_id` | Admin library command flow | Admin confirmation/mutation owns POST. |
| `library.metadata_profile` | `update_library_metadata_profile` | `administrator` | `library_id` | Admin Libraries or Settings metadata section | Admin route owns editing. |
| `item.metadata_refresh` | `refresh_item_metadata` | `library_manage` | `item_id` plus library context when available | Admin item governance/metadata refresh flow | Admin confirmation/mutation owns POST. |
| `jobs.filtered` | `view_jobs` | `administrator` | any context | `/admin/tasks` with safe filter state when supported | Admin route owns job details. |
| `playback.support` | `view_playback_diagnostics` | `administrator` | `source_id` or `playback_session_id` | Admin playback/support diagnostics route or readiness state | Admin route owns diagnostics. |
| `playback.runtime` | `view_playback_runtime` | `administrator` | any context | `/admin/transcoding` or playback runtime panel | Admin route owns runtime details. |
| `access.library_policies` | `manage_library_access` | `administrator` | `library_id` | `/admin/users` or access policy section | Admin route owns policy editing. |

## First Media Surfaces

| Media surface | Context query | Link group |
| --- | --- | --- |
| `/media/detail?id=:itemId&type=:type` | `item_id`, selected `source_id` when known, `library_id` from source/item data when known | Manage item, refresh metadata, playback support, jobs. |
| `/media/library?id=:libraryId` | `library_id` | Manage library, scan, metadata profile, access policies, jobs. |
| Source/version picker | `source_id`, `item_id`, `library_id` | Inspect source playback support, refresh item metadata, jobs. |
| Watch/player error state | `source_id`, `playback_session_id` when available | Playback support, playback runtime, current jobs. |

## Admin-to-Media Links

| Admin surface | Media target | Condition |
| --- | --- | --- |
| Admin Libraries | `/media/library?id=:libraryId` | Current principal has Public Client browse access to the library. |
| Admin item/governance detail | `/media/detail?id=:itemId&type=:type` | Current principal has Public Client browse access to the item. |
| Admin playback session detail | `/media/detail` or `/media/watch` when item/source context exists | Playback session exposes safe item/source context. |

## Resolver Rules

- Accept known backend `route_name` values only.
- Preserve only stable IDs and typed frontend search params.
- Do not render raw local paths, Source Locators, provider payloads, FFmpeg
  argv/stderr, output paths, bearer tokens, or storage handles.
- Mutating links must enter an Admin-owned confirmation or command surface.
- Unknown route names should be omitted or shown as unsupported in development
  diagnostics, never guessed.
