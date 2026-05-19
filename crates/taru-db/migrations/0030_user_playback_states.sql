CREATE TABLE user_playback_states (
    principal_id TEXT NOT NULL,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    resume_position_ms INTEGER,
    duration_ms INTEGER,
    watched INTEGER NOT NULL DEFAULT 0,
    watched_at_ms INTEGER,
    last_played_at_ms INTEGER,
    updated_at_ms INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (principal_id, item_id),
    CHECK (length(principal_id) > 0),
    CHECK (resume_position_ms IS NULL OR resume_position_ms >= 0),
    CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CHECK (watched IN (0, 1)),
    CHECK (version >= 1)
);

CREATE INDEX user_playback_states_continue_watching_idx
    ON user_playback_states(principal_id, watched, last_played_at_ms DESC, item_id);

CREATE INDEX user_playback_states_source_id_idx
    ON user_playback_states(source_id);
