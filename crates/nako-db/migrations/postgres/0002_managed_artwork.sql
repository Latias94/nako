CREATE TABLE IF NOT EXISTS artwork_tasks (
    id uuid PRIMARY KEY NOT NULL,
    image_id uuid NOT NULL REFERENCES image_assets(id) ON DELETE CASCADE,
    kind text NOT NULL,
    status text NOT NULL,
    resource_class text NOT NULL,
    attempts bigint NOT NULL DEFAULT 0,
    max_attempts bigint NOT NULL DEFAULT 3,
    error text,
    queued_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (attempts >= 0),
    CHECK (max_attempts >= 1)
);

CREATE INDEX IF NOT EXISTS artwork_tasks_status_idx
    ON artwork_tasks(status);
CREATE INDEX IF NOT EXISTS artwork_tasks_resource_class_idx
    ON artwork_tasks(resource_class);

CREATE TABLE IF NOT EXISTS addon_artwork_candidates (
    id uuid PRIMARY KEY NOT NULL,
    addon_id uuid NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    side_effect_id uuid NOT NULL UNIQUE REFERENCES addon_side_effects(id) ON DELETE CASCADE,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    source_kind text NOT NULL,
    source_uri text NOT NULL,
    width bigint,
    height bigint,
    language text,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(addon_id, library_id, item_id, kind, kind_key, source_kind, source_uri),
    CHECK (source_kind IN ('remote_url')),
    CHECK (length(source_uri) <= 2048),
    CHECK (width IS NULL OR (width BETWEEN 1 AND 20000)),
    CHECK (height IS NULL OR (height BETWEEN 1 AND 20000)),
    CHECK (language IS NULL OR length(language) <= 32),
    CHECK (status IN ('proposed', 'accepted', 'rejected'))
);

CREATE INDEX IF NOT EXISTS addon_artwork_candidates_item_idx
    ON addon_artwork_candidates(item_id, status, kind, kind_key);

CREATE TABLE IF NOT EXISTS managed_artwork_ingests (
    id uuid PRIMARY KEY NOT NULL,
    candidate_id uuid NOT NULL UNIQUE REFERENCES addon_artwork_candidates(id) ON DELETE CASCADE,
    job_id uuid NOT NULL UNIQUE REFERENCES jobs(id) ON DELETE CASCADE,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    status text NOT NULL,
    artifact_id uuid,
    failure_code text,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (status IN ('queued', 'fetching', 'validating', 'stored', 'failed')),
    CHECK (failure_code IS NULL OR length(failure_code) <= 128)
);

CREATE INDEX IF NOT EXISTS managed_artwork_ingests_item_idx
    ON managed_artwork_ingests(item_id, status, kind, kind_key);
CREATE INDEX IF NOT EXISTS managed_artwork_ingests_job_idx
    ON managed_artwork_ingests(job_id);

CREATE TABLE IF NOT EXISTS managed_artwork_artifacts (
    id uuid PRIMARY KEY NOT NULL,
    ingest_id uuid NOT NULL UNIQUE REFERENCES managed_artwork_ingests(id) ON DELETE CASCADE,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    storage_uri text NOT NULL,
    content_hash text,
    width bigint,
    height bigint,
    byte_len bigint,
    media_type text,
    deleted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    CHECK (width IS NULL OR (width BETWEEN 1 AND 20000)),
    CHECK (height IS NULL OR (height BETWEEN 1 AND 20000)),
    CHECK (byte_len IS NULL OR byte_len >= 0),
    CHECK (media_type IS NULL OR length(media_type) <= 128)
);

CREATE INDEX IF NOT EXISTS managed_artwork_artifacts_item_idx
    ON managed_artwork_artifacts(item_id, kind, kind_key);
CREATE INDEX IF NOT EXISTS managed_artwork_artifacts_deleted_idx
    ON managed_artwork_artifacts(deleted_at, created_at, id);

CREATE TABLE IF NOT EXISTS selected_artworks (
    id uuid PRIMARY KEY NOT NULL,
    library_id uuid NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id uuid NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind text NOT NULL,
    kind_key text NOT NULL DEFAULT '',
    artifact_id uuid NOT NULL REFERENCES managed_artwork_artifacts(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT statement_timestamp(),
    UNIQUE(item_id, kind, kind_key)
);

CREATE INDEX IF NOT EXISTS selected_artworks_artifact_idx
    ON selected_artworks(artifact_id);
CREATE INDEX IF NOT EXISTS selected_artworks_item_idx
    ON selected_artworks(item_id, kind, kind_key);
