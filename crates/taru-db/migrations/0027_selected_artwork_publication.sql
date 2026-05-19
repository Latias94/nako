CREATE TABLE selected_artworks (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    artifact_id TEXT NOT NULL REFERENCES managed_artwork_artifacts(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(item_id, kind, kind_key)
);

CREATE INDEX selected_artworks_artifact_idx
    ON selected_artworks(artifact_id);

CREATE INDEX selected_artworks_item_idx
    ON selected_artworks(item_id, kind, kind_key);
