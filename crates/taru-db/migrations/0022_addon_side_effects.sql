CREATE TABLE addon_side_effects (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    permission TEXT NOT NULL,
    library_id TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    validation_status TEXT NOT NULL,
    safe_error_code TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, idempotency_key)
);

CREATE INDEX addon_side_effects_addon_created_idx
    ON addon_side_effects(addon_id, created_at, id);

CREATE INDEX addon_side_effects_library_target_idx
    ON addon_side_effects(library_id, target_kind, target_id, created_at);
