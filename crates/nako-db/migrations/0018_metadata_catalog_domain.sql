CREATE TABLE provider_subjects (
    id TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    subject_kind TEXT NOT NULL,
    subject_kind_key TEXT NOT NULL DEFAULT '',
    subject_key TEXT NOT NULL,
    title TEXT,
    release_year INTEGER,
    locale TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX provider_subjects_lookup_idx
    ON provider_subjects(provider, provider_key, subject_kind, subject_kind_key, subject_key);

CREATE INDEX provider_subjects_provider_idx
    ON provider_subjects(provider, provider_key);

CREATE TABLE provider_mappings (
    id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    subject_id TEXT NOT NULL REFERENCES provider_subjects(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    confidence_milli INTEGER,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(item_id, subject_id)
);

CREATE INDEX provider_mappings_item_id_idx
    ON provider_mappings(item_id, status);

CREATE INDEX provider_mappings_subject_id_idx
    ON provider_mappings(subject_id, status);

CREATE TABLE source_duplicate_relationships (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    duplicate_source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    evidence_kind TEXT NOT NULL,
    evidence_kind_key TEXT NOT NULL DEFAULT '',
    evidence_value TEXT,
    status TEXT NOT NULL,
    confidence_milli INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK(source_id < duplicate_source_id),
    UNIQUE(source_id, duplicate_source_id)
);

CREATE INDEX source_duplicate_relationships_source_id_idx
    ON source_duplicate_relationships(source_id, status);

CREATE INDEX source_duplicate_relationships_duplicate_source_id_idx
    ON source_duplicate_relationships(duplicate_source_id, status);

CREATE TABLE local_inference_evidence (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    inferred_kind TEXT NOT NULL,
    inferred_title TEXT,
    inferred_year INTEGER,
    inferred_season INTEGER,
    inferred_episode INTEGER,
    confidence_milli INTEGER,
    evidence_source TEXT NOT NULL,
    evidence_source_key TEXT NOT NULL DEFAULT '',
    evidence_value TEXT NOT NULL,
    inference_version TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX local_inference_evidence_source_id_idx
    ON local_inference_evidence(source_id, inference_version);
