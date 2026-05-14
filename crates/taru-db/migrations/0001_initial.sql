CREATE TABLE libraries (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    roots_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE media_items (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    parent_id TEXT REFERENCES media_items(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    original_title TEXT,
    sort_title TEXT,
    overview TEXT,
    release_date TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX media_items_parent_id_idx ON media_items(parent_id);
CREATE INDEX media_items_kind_idx ON media_items(kind);

CREATE TABLE media_item_external_ids (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (item_id, provider, provider_key, value)
);

CREATE INDEX media_item_external_ids_lookup_idx
    ON media_item_external_ids(provider, provider_key, value);

CREATE TABLE media_sources (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    locator TEXT NOT NULL,
    file_name TEXT NOT NULL,
    size_bytes INTEGER,
    fingerprint TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX media_sources_locator_idx ON media_sources(locator);
CREATE INDEX media_sources_library_id_idx ON media_sources(library_id);
CREATE INDEX media_sources_item_id_idx ON media_sources(item_id);
