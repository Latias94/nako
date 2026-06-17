CREATE TABLE user_playback_profile_preferences (
    principal_id TEXT PRIMARY KEY NOT NULL,
    capabilities_json TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (length(principal_id) > 0),
    CHECK (length(capabilities_json) > 0),
    CHECK (json_valid(capabilities_json)),
    CHECK (version >= 1)
);

CREATE INDEX user_playback_profile_preferences_updated_idx
    ON user_playback_profile_preferences(updated_at_ms DESC, principal_id);
