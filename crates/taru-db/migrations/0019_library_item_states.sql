CREATE TABLE library_item_states (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provisional INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, item_id)
);

CREATE INDEX library_item_states_item_id_idx
    ON library_item_states(item_id);

CREATE INDEX library_item_states_library_provisional_idx
    ON library_item_states(library_id, provisional);
