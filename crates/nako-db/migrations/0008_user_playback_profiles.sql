CREATE TABLE user_playback_profiles (
    profile_id TEXT PRIMARY KEY NOT NULL,
    principal_id TEXT NOT NULL,
    name TEXT NOT NULL,
    capabilities_json TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    updated_at_ms INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (length(profile_id) > 0),
    CHECK (length(principal_id) > 0),
    CHECK (length(name) > 0),
    CHECK (length(capabilities_json) > 0),
    CHECK (json_valid(capabilities_json)),
    CHECK (is_default IN (0, 1)),
    CHECK (version >= 1)
);

CREATE INDEX user_playback_profiles_principal_idx
    ON user_playback_profiles(principal_id, updated_at_ms DESC, profile_id);

CREATE UNIQUE INDEX user_playback_profiles_default_idx
    ON user_playback_profiles(principal_id)
    WHERE is_default = 1;

INSERT INTO user_playback_profiles (
    profile_id,
    principal_id,
    name,
    capabilities_json,
    is_default,
    updated_at_ms,
    version
)
SELECT
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-' ||
    '7' || substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', abs(random() % 4) + 1, 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6))) AS profile_id,
    principal_id,
    'Default',
    capabilities_json,
    1,
    updated_at_ms,
    version
FROM user_playback_profile_preferences;
