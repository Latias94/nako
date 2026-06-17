CREATE TABLE IF NOT EXISTS user_playback_profiles (
    profile_id uuid PRIMARY KEY NOT NULL,
    principal_id text NOT NULL,
    name text NOT NULL,
    capabilities_json jsonb NOT NULL,
    is_default boolean NOT NULL DEFAULT false,
    updated_at_ms bigint NOT NULL,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (length(principal_id) > 0),
    CHECK (length(name) > 0),
    CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS user_playback_profiles_principal_idx
    ON user_playback_profiles(principal_id, updated_at_ms DESC, profile_id);

CREATE UNIQUE INDEX IF NOT EXISTS user_playback_profiles_default_idx
    ON user_playback_profiles(principal_id)
    WHERE is_default;

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
    (
        substr(md5(principal_id || ':nako-default-playback-profile'), 1, 8) || '-' ||
        substr(md5(principal_id || ':nako-default-playback-profile'), 9, 4) || '-' ||
        substr(md5(principal_id || ':nako-default-playback-profile'), 13, 4) || '-' ||
        substr(md5(principal_id || ':nako-default-playback-profile'), 17, 4) || '-' ||
        substr(md5(principal_id || ':nako-default-playback-profile'), 21, 12)
    )::uuid,
    principal_id,
    'Default',
    capabilities_json,
    true,
    updated_at_ms,
    version
FROM user_playback_profile_preferences
ON CONFLICT (profile_id) DO NOTHING;
