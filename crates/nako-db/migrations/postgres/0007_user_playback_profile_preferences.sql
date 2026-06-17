CREATE TABLE IF NOT EXISTS user_playback_profile_preferences (
    principal_id text PRIMARY KEY NOT NULL,
    capabilities_json jsonb NOT NULL,
    updated_at_ms bigint NOT NULL,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (length(principal_id) > 0),
    CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS user_playback_profile_preferences_updated_idx
    ON user_playback_profile_preferences(updated_at_ms DESC, principal_id);
