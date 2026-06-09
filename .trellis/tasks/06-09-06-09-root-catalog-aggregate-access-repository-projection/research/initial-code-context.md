# Initial code context

Date: 2026-06-09

## Current behavior

- `crates/nako-server/src/http/catalog.rs` routes `/people`, `/tags`, and `/genres` through plain app service list methods, then filters each returned aggregate DTO in the handler.
- The helper path is:
  - `list_people` -> `person_has_accessible_item` -> `list_person_items(MAX_LIMIT)` -> per-item `item_has_access`
  - `list_tags` -> `tag_has_accessible_item` -> `list_tag_items(MAX_LIMIT)` -> per-item `item_has_access`
  - `list_genres` -> `genre_has_accessible_item` -> `list_genre_items(MAX_LIMIT)` -> per-item `item_has_access`
- This performs access filtering after pagination and can leave short pages when inaccessible aggregate records occupy the requested page.

## Existing repository surface

- `CatalogRepository` has plain root list methods:
  - `list_people(page)`
  - `list_tags(page)`
  - `list_genres(page)`
- `CatalogRepository` already has accessible relation item methods:
  - `list_accessible_person_items(principal, person_id, page)`
  - `list_accessible_tag_items(principal, tag_id, page)`
  - `list_accessible_genre_items(principal, genre_id, page)`

## Adapter reuse points

- SQLite and PostgreSQL both have adapter-local `access.rs` helpers:
  - `push_media_item_access_filter`
  - `push_media_item_access_exists`
  - `push_media_item_id_filter`
- Existing `list_accessible_catalog_item_rows` helpers in SQLite/PostgreSQL catalog adapters already demonstrate the relation-table-plus-access-filter query pattern.

## Semantics to preserve

- Non-admin principals need at least one related item with browse-or-higher access through user or role library policy.
- Administrators currently see aggregate records even if the aggregate has no related items because the HTTP helper returns true for admin when relation item hydration returns an empty list. Accessible root aggregate repository methods should preserve this by making admin list behavior equivalent to plain root list behavior.
- Ordering should match existing plain root list ordering.
