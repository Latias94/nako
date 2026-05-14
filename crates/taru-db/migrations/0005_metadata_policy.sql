ALTER TABLE media_items ADD COLUMN metadata_json TEXT;

CREATE TABLE metadata_field_locks (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    locked INTEGER NOT NULL DEFAULT 1,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, field)
);

CREATE INDEX metadata_field_locks_item_id_idx ON metadata_field_locks(item_id);

CREATE TABLE provider_raw_responses (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    body_json TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, provider, provider_key)
);

CREATE INDEX provider_raw_responses_item_id_idx ON provider_raw_responses(item_id);
CREATE INDEX provider_raw_responses_lookup_idx
    ON provider_raw_responses(provider, provider_key, item_id);
