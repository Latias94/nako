CREATE TABLE addon_registrations (
    id TEXT PRIMARY KEY NOT NULL,
    manifest_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    protocol_version TEXT NOT NULL,
    base_url TEXT NOT NULL,
    manifest_json TEXT NOT NULL,
    granted_scopes_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX addon_registrations_status_idx
    ON addon_registrations(status, created_at);
