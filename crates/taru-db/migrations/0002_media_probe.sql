CREATE TABLE media_source_probes (
    source_id TEXT PRIMARY KEY NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    duration_ms INTEGER,
    container TEXT,
    bit_rate INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE media_streams (
    source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    stream_index INTEGER NOT NULL,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    codec TEXT,
    language TEXT,
    duration_ms INTEGER,
    bit_rate INTEGER,
    width INTEGER,
    height INTEGER,
    channels INTEGER,
    sample_rate INTEGER,
    PRIMARY KEY (source_id, stream_index)
);

CREATE INDEX media_streams_source_id_idx ON media_streams(source_id);
CREATE INDEX media_streams_kind_idx ON media_streams(kind, kind_key);
