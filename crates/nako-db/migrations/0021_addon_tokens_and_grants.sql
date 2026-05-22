CREATE TABLE addon_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    token_prefix TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    rotated_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT
);

CREATE INDEX addon_tokens_addon_status_idx
    ON addon_tokens(addon_id, status, created_at);

CREATE TABLE addon_grants (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, permission, library_id)
);

CREATE INDEX addon_grants_addon_idx
    ON addon_grants(addon_id, permission, library_id);

CREATE UNIQUE INDEX addon_grants_unique_scope_idx
    ON addon_grants(addon_id, permission, COALESCE(library_id, ''));
