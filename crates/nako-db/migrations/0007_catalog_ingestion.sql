CREATE TABLE people (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    sort_name TEXT,
    overview TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX people_name_idx ON people(name);

CREATE TABLE person_external_ids (
    person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (person_id, provider, provider_key, value)
);

CREATE INDEX person_external_ids_lookup_idx
    ON person_external_ids(provider, provider_key, value);

CREATE TABLE item_credits (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    role_key TEXT NOT NULL DEFAULT '',
    character TEXT NOT NULL DEFAULT '',
    sort_order INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, person_id, role, role_key, character)
);

CREATE INDEX item_credits_item_id_idx ON item_credits(item_id);
CREATE INDEX item_credits_person_id_idx ON item_credits(person_id);
CREATE INDEX item_credits_role_idx ON item_credits(role, role_key);

CREATE TABLE genres (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX genres_name_source_idx ON genres(name, source, source_key);

CREATE TABLE item_genres (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    genre_id TEXT NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, genre_id)
);

CREATE INDEX item_genres_genre_id_idx ON item_genres(genre_id);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX tags_name_source_idx ON tags(name, source, source_key);

CREATE TABLE item_tags (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE INDEX item_tags_tag_id_idx ON item_tags(tag_id);

CREATE TABLE collections (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    overview TEXT,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX collections_name_idx ON collections(name);

CREATE TABLE collection_external_ids (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (collection_id, provider, provider_key, value)
);

CREATE INDEX collection_external_ids_lookup_idx
    ON collection_external_ids(provider, provider_key, value);

CREATE TABLE collection_items (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    sort_order INTEGER,
    PRIMARY KEY (collection_id, item_id)
);

CREATE INDEX collection_items_item_id_idx ON collection_items(item_id);

CREATE TABLE studios (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX studios_name_idx ON studios(name);

CREATE TABLE studio_external_ids (
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (studio_id, provider, provider_key, value)
);

CREATE INDEX studio_external_ids_lookup_idx
    ON studio_external_ids(provider, provider_key, value);

CREATE TABLE item_studios (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, studio_id)
);

CREATE INDEX item_studios_studio_id_idx ON item_studios(studio_id);

CREATE TABLE image_assets (
    id TEXT PRIMARY KEY NOT NULL,
    owner_kind TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    source_uri TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    cache_uri TEXT,
    width INTEGER,
    height INTEGER,
    language TEXT,
    selected INTEGER NOT NULL DEFAULT 0,
    content_hash TEXT,
    etag TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX image_assets_owner_idx ON image_assets(owner_kind, owner_id);
CREATE INDEX image_assets_kind_idx ON image_assets(kind, kind_key);
CREATE UNIQUE INDEX image_assets_source_idx
    ON image_assets(owner_kind, owner_id, kind, kind_key, source_uri);

CREATE TABLE scan_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    root TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    status TEXT NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX scan_snapshots_library_id_idx ON scan_snapshots(library_id);
CREATE INDEX scan_snapshots_status_idx ON scan_snapshots(status);

CREATE TABLE directory_snapshots (
    scan_id TEXT NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    uri TEXT NOT NULL,
    etag TEXT,
    modified_at TEXT,
    child_count INTEGER NOT NULL,
    PRIMARY KEY (scan_id, uri)
);

CREATE TABLE source_states (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    uri TEXT NOT NULL,
    size_bytes INTEGER,
    modified_at TEXT,
    etag TEXT,
    fingerprint TEXT,
    last_seen_scan_id TEXT NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    tombstoned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, uri)
);

CREATE INDEX source_states_source_id_idx ON source_states(source_id);
CREATE INDEX source_states_fingerprint_idx ON source_states(fingerprint);
CREATE INDEX source_states_scan_id_idx ON source_states(last_seen_scan_id);

CREATE TABLE search_documents (
    item_id TEXT PRIMARY KEY NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    projection_version INTEGER NOT NULL DEFAULT 1,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    aliases_json TEXT NOT NULL DEFAULT '[]',
    facets_json TEXT NOT NULL,
    facets_text TEXT NOT NULL,
    sort_keys_json TEXT NOT NULL DEFAULT '[]',
    provider_identifiers_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX search_documents_title_idx ON search_documents(title);

CREATE TABLE artwork_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    image_id TEXT NOT NULL REFERENCES image_assets(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    resource_class TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    error TEXT,
    queued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX artwork_tasks_status_idx ON artwork_tasks(status);
CREATE INDEX artwork_tasks_resource_class_idx ON artwork_tasks(resource_class);
