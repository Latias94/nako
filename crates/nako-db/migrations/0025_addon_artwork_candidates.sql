CREATE TABLE addon_artwork_candidates (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    side_effect_id TEXT NOT NULL UNIQUE REFERENCES addon_side_effects(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    source_kind TEXT NOT NULL CHECK(source_kind IN ('remote_url')),
    source_uri TEXT NOT NULL CHECK(length(source_uri) <= 2048),
    width INTEGER CHECK(width IS NULL OR (width BETWEEN 1 AND 20000)),
    height INTEGER CHECK(height IS NULL OR (height BETWEEN 1 AND 20000)),
    language TEXT CHECK(language IS NULL OR length(language) <= 32),
    status TEXT NOT NULL CHECK(status IN ('proposed', 'accepted', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, library_id, item_id, kind, kind_key, source_kind, source_uri)
);

CREATE INDEX addon_artwork_candidates_item_idx
    ON addon_artwork_candidates(item_id, status, kind, kind_key);
