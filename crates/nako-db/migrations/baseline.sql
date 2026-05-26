-- Consolidated SQLite baseline for Nako's pre-production schema.
-- Generated from the previous backend-owned migration chain on 2026-05-26.
-- Runtime migrations intentionally start from this single baseline while Nako has no production database compatibility burden.

-- From 0001_initial.sql
CREATE TABLE libraries (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    roots_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE media_items (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    parent_id TEXT REFERENCES media_items(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    original_title TEXT,
    sort_title TEXT,
    overview TEXT,
    release_date TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX media_items_parent_id_idx ON media_items(parent_id);
CREATE INDEX media_items_kind_idx ON media_items(kind);

CREATE TABLE media_item_external_ids (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (item_id, provider, provider_key, value)
);

CREATE INDEX media_item_external_ids_lookup_idx
    ON media_item_external_ids(provider, provider_key, value);

CREATE TABLE media_sources (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    locator TEXT NOT NULL,
    file_name TEXT NOT NULL,
    size_bytes INTEGER,
    fingerprint TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX media_sources_library_locator_idx ON media_sources(library_id, locator);
CREATE INDEX media_sources_library_id_idx ON media_sources(library_id);
CREATE INDEX media_sources_item_id_idx ON media_sources(item_id);


-- From 0002_media_probe.sql
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


-- From 0003_jobs.sql
CREATE TABLE jobs (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    resource_class TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    summary_json TEXT,
    error TEXT,
    queued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX jobs_status_idx ON jobs(status);
CREATE INDEX jobs_kind_idx ON jobs(kind);
CREATE INDEX jobs_library_id_idx ON jobs(library_id);
CREATE INDEX jobs_source_id_idx ON jobs(source_id);


-- From 0004_job_input_payload.sql
ALTER TABLE jobs ADD COLUMN input_json TEXT;


-- From 0005_metadata_policy.sql
ALTER TABLE media_items ADD COLUMN metadata_json TEXT;

CREATE TABLE metadata_field_locks (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    locked INTEGER NOT NULL DEFAULT 1,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, field)
);

CREATE INDEX metadata_field_locks_item_id_idx ON metadata_field_locks(item_id);

CREATE TABLE provider_raw_responses (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    body_json TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, provider, provider_key)
);

CREATE INDEX provider_raw_responses_item_id_idx ON provider_raw_responses(item_id);
CREATE INDEX provider_raw_responses_lookup_idx
    ON provider_raw_responses(provider, provider_key, item_id);


-- From 0006_library_profiles.sql
ALTER TABLE libraries ADD COLUMN domain TEXT NOT NULL DEFAULT 'mixed';
ALTER TABLE libraries ADD COLUMN preset TEXT NOT NULL DEFAULT 'mixed_video';
ALTER TABLE libraries ADD COLUMN options_json TEXT;


-- From 0007_catalog_ingestion.sql
CREATE TABLE people (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    sort_name TEXT,
    overview TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX people_name_idx ON people(name);

CREATE TABLE person_external_ids (
    person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (person_id, provider, provider_key, value)
);

CREATE INDEX person_external_ids_lookup_idx
    ON person_external_ids(provider, provider_key, value);

CREATE TABLE item_credits (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    role_key TEXT NOT NULL DEFAULT '',
    character TEXT NOT NULL DEFAULT '',
    sort_order INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (item_id, person_id, role, role_key, character)
);

CREATE INDEX item_credits_item_id_idx ON item_credits(item_id);
CREATE INDEX item_credits_person_id_idx ON item_credits(person_id);
CREATE INDEX item_credits_role_idx ON item_credits(role, role_key);

CREATE TABLE genres (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX genres_name_source_idx ON genres(name, source, source_key);

CREATE TABLE item_genres (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    genre_id TEXT NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, genre_id)
);

CREATE INDEX item_genres_genre_id_idx ON item_genres(genre_id);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX tags_name_source_idx ON tags(name, source, source_key);

CREATE TABLE item_tags (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE INDEX item_tags_tag_id_idx ON item_tags(tag_id);

CREATE TABLE collections (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    overview TEXT,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX collections_name_idx ON collections(name);

CREATE TABLE collection_external_ids (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (collection_id, provider, provider_key, value)
);

CREATE INDEX collection_external_ids_lookup_idx
    ON collection_external_ids(provider, provider_key, value);

CREATE TABLE collection_items (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    sort_order INTEGER,
    PRIMARY KEY (collection_id, item_id)
);

CREATE INDEX collection_items_item_id_idx ON collection_items(item_id);

CREATE TABLE studios (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX studios_name_idx ON studios(name);

CREATE TABLE studio_external_ids (
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (studio_id, provider, provider_key, value)
);

CREATE INDEX studio_external_ids_lookup_idx
    ON studio_external_ids(provider, provider_key, value);

CREATE TABLE item_studios (
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    studio_id TEXT NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, studio_id)
);

CREATE INDEX item_studios_studio_id_idx ON item_studios(studio_id);

CREATE TABLE image_assets (
    id TEXT PRIMARY KEY NOT NULL,
    owner_kind TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    source_uri TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_key TEXT NOT NULL DEFAULT '',
    cache_uri TEXT,
    width INTEGER,
    height INTEGER,
    language TEXT,
    selected INTEGER NOT NULL DEFAULT 0,
    content_hash TEXT,
    etag TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX image_assets_owner_idx ON image_assets(owner_kind, owner_id);
CREATE INDEX image_assets_kind_idx ON image_assets(kind, kind_key);
CREATE UNIQUE INDEX image_assets_source_idx
    ON image_assets(owner_kind, owner_id, kind, kind_key, source_uri);

CREATE TABLE scan_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    root TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    status TEXT NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX scan_snapshots_library_id_idx ON scan_snapshots(library_id);
CREATE INDEX scan_snapshots_status_idx ON scan_snapshots(status);

CREATE TABLE directory_snapshots (
    scan_id TEXT NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    uri TEXT NOT NULL,
    etag TEXT,
    modified_at TEXT,
    child_count INTEGER NOT NULL,
    PRIMARY KEY (scan_id, uri)
);

CREATE TABLE source_states (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    uri TEXT NOT NULL,
    size_bytes INTEGER,
    modified_at TEXT,
    etag TEXT,
    fingerprint TEXT,
    last_seen_scan_id TEXT NOT NULL REFERENCES scan_snapshots(id) ON DELETE CASCADE,
    tombstoned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, uri)
);

CREATE INDEX source_states_source_id_idx ON source_states(source_id);
CREATE INDEX source_states_fingerprint_idx ON source_states(fingerprint);
CREATE INDEX source_states_scan_id_idx ON source_states(last_seen_scan_id);

CREATE TABLE search_documents (
    item_id TEXT PRIMARY KEY NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    projection_version INTEGER NOT NULL DEFAULT 1,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    aliases_json TEXT NOT NULL DEFAULT '[]',
    facets_json TEXT NOT NULL,
    facets_text TEXT NOT NULL,
    sort_keys_json TEXT NOT NULL DEFAULT '[]',
    provider_identifiers_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX search_documents_title_idx ON search_documents(title);

CREATE TABLE artwork_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    image_id TEXT NOT NULL REFERENCES image_assets(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    resource_class TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    error TEXT,
    queued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX artwork_tasks_status_idx ON artwork_tasks(status);
CREATE INDEX artwork_tasks_resource_class_idx ON artwork_tasks(resource_class);


-- From 0008_transcode_sessions.sql
CREATE TABLE transcode_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES media_sources(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    request_key TEXT NOT NULL,
    output_path TEXT NOT NULL,
    state TEXT NOT NULL,
    failure_category TEXT,
    failure_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX transcode_sessions_source_idx
    ON transcode_sessions(source_id);

CREATE INDEX transcode_sessions_request_idx
    ON transcode_sessions(source_id, kind, request_key, updated_at);

CREATE INDEX transcode_sessions_state_idx
    ON transcode_sessions(state);

CREATE UNIQUE INDEX transcode_sessions_active_request_idx
    ON transcode_sessions(source_id, kind, request_key)
    WHERE state IN ('planned', 'starting', 'running', 'cancel_requested');


-- From 0009_event_outbox.sql
CREATE TABLE event_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    idempotency_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    next_attempt_at TEXT,
    UNIQUE(kind, idempotency_key)
);

CREATE INDEX event_outbox_status_idx
    ON event_outbox(status, occurred_at);

CREATE INDEX event_outbox_subject_idx
    ON event_outbox(subject_kind, subject_id, occurred_at);

CREATE INDEX event_outbox_library_idx
    ON event_outbox(library_id, occurred_at);

CREATE INDEX event_outbox_source_idx
    ON event_outbox(source_id, occurred_at);


-- From 0010_webhooks.sql
CREATE TABLE webhook_endpoints (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret_env TEXT,
    subscribed_event_kinds_json TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX webhook_endpoints_status_idx
    ON webhook_endpoints(status);

CREATE TABLE webhook_delivery_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    endpoint_id TEXT NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    http_status INTEGER,
    error TEXT,
    requested_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    next_retry_at TEXT,
    UNIQUE(endpoint_id, event_id, attempt_number)
);

CREATE INDEX webhook_delivery_attempts_event_idx
    ON webhook_delivery_attempts(event_id, requested_at);

CREATE INDEX webhook_delivery_attempts_endpoint_idx
    ON webhook_delivery_attempts(endpoint_id, requested_at);

CREATE INDEX webhook_delivery_attempts_status_idx
    ON webhook_delivery_attempts(status, next_retry_at);


-- From 0011_automation.sql
CREATE TABLE automation_providers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    secret_env TEXT,
    capabilities_json TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX automation_providers_status_idx
    ON automation_providers(status);

CREATE TABLE automation_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES automation_providers(id) ON DELETE CASCADE,
    capability TEXT NOT NULL,
    kind TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL,
    item_id TEXT REFERENCES media_items(id) ON DELETE SET NULL,
    source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    artifact_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    accepted_at TEXT
);

CREATE INDEX automation_artifacts_job_idx
    ON automation_artifacts(job_id, created_at);

CREATE INDEX automation_artifacts_item_idx
    ON automation_artifacts(item_id, created_at);

CREATE INDEX automation_artifacts_status_idx
    ON automation_artifacts(status, created_at);


-- From 0012_addons.sql
CREATE TABLE addon_registrations (
    id TEXT PRIMARY KEY NOT NULL,
    manifest_id TEXT NOT NULL,
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


-- From 0013_vfs_cache.sql
CREATE TABLE vfs_cache_objects (
    uri TEXT PRIMARY KEY NOT NULL,
    scheme TEXT NOT NULL,
    kind TEXT NOT NULL,
    len INTEGER,
    modified_at TEXT,
    etag TEXT,
    fingerprint TEXT,
    capabilities_bits INTEGER NOT NULL,
    fetched_at_ms INTEGER NOT NULL,
    fresh_until_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX vfs_cache_objects_scheme_idx
    ON vfs_cache_objects(scheme, fresh_until_ms);

CREATE TABLE vfs_cache_listings (
    uri TEXT PRIMARY KEY NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    scheme TEXT NOT NULL,
    fetched_at_ms INTEGER NOT NULL,
    fresh_until_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX vfs_cache_listings_scheme_idx
    ON vfs_cache_listings(scheme, fresh_until_ms);

CREATE TABLE vfs_cache_listing_entries (
    listing_uri TEXT NOT NULL REFERENCES vfs_cache_listings(uri) ON DELETE CASCADE,
    entry_uri TEXT NOT NULL REFERENCES vfs_cache_objects(uri) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (listing_uri, entry_uri)
);

CREATE INDEX vfs_cache_listing_entries_entry_idx
    ON vfs_cache_listing_entries(entry_uri);

CREATE TABLE vfs_cache_failures (
    uri TEXT NOT NULL,
    scheme TEXT NOT NULL,
    operation TEXT NOT NULL,
    failed_at_ms INTEGER NOT NULL,
    failure_count INTEGER NOT NULL,
    error TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (uri, operation)
);

CREATE INDEX vfs_cache_failures_scheme_idx
    ON vfs_cache_failures(scheme, operation, failed_at_ms);


-- From 0014_staging_manifest.sql
CREATE TABLE IF NOT EXISTS staging_manifest_records (
    id TEXT PRIMARY KEY NOT NULL,
    source_uri TEXT NOT NULL,
    source_scheme TEXT NOT NULL,
    purpose TEXT NOT NULL,
    local_path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER,
    etag TEXT,
    fingerprint TEXT,
    state TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    last_accessed_at_ms INTEGER NOT NULL,
    expires_at_ms INTEGER,
    active_leases INTEGER NOT NULL DEFAULT 0,
    validation_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_staging_manifest_source_purpose
ON staging_manifest_records (source_uri, purpose);

CREATE INDEX IF NOT EXISTS idx_staging_manifest_cleanup
ON staging_manifest_records (state, active_leases, expires_at_ms, last_accessed_at_ms);


-- From 0015_media_source_library_locator.sql
DROP INDEX IF EXISTS media_sources_locator_idx;

CREATE UNIQUE INDEX IF NOT EXISTS media_sources_library_locator_idx
    ON media_sources(library_id, locator);


-- From 0016_metadata_provider_attempts.sql
CREATE TABLE metadata_provider_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_key TEXT,
    status TEXT NOT NULL,
    matched_by TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL,
    error_class TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX metadata_provider_attempts_job_id_idx
    ON metadata_provider_attempts(job_id, started_at);

CREATE INDEX metadata_provider_attempts_item_id_idx
    ON metadata_provider_attempts(item_id, started_at);


-- From 0017_ingestion_failures.sql
CREATE TABLE ingestion_failures (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    phase TEXT NOT NULL,
    target_uri TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    job_id TEXT,
    scan_id TEXT,
    source_id TEXT,
    failure_class TEXT NOT NULL,
    status TEXT NOT NULL,
    message TEXT NOT NULL,
    retryable INTEGER NOT NULL,
    attempts INTEGER NOT NULL,
    first_failed_at_ms INTEGER NOT NULL,
    last_failed_at_ms INTEGER NOT NULL,
    resolved_at_ms INTEGER,
    ignored_at_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, phase, target_uri)
);

CREATE INDEX ingestion_failures_library_status_idx
    ON ingestion_failures(library_id, status, phase, target_uri);

CREATE INDEX ingestion_failures_job_idx
    ON ingestion_failures(job_id);

CREATE INDEX ingestion_failures_scan_idx
    ON ingestion_failures(scan_id);

CREATE INDEX ingestion_failures_source_idx
    ON ingestion_failures(source_id);


-- From 0018_metadata_catalog_domain.sql
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


-- From 0019_library_item_states.sql
CREATE TABLE library_item_states (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provisional INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (library_id, item_id)
);

CREATE INDEX library_item_states_item_id_idx
    ON library_item_states(item_id);

CREATE INDEX library_item_states_library_provisional_idx
    ON library_item_states(library_id, provisional);


-- From 0020_local_inference_evidence_snapshot_key.sql
DELETE FROM local_inference_evidence
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM local_inference_evidence
    GROUP BY source_id, evidence_source, evidence_source_key, inference_version
);

CREATE UNIQUE INDEX local_inference_evidence_snapshot_idx
    ON local_inference_evidence(
        source_id,
        evidence_source,
        evidence_source_key,
        inference_version
    );


-- From 0021_addon_tokens_and_grants.sql
CREATE TABLE addon_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    token_prefix TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    rotated_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT
);

CREATE INDEX addon_tokens_addon_status_idx
    ON addon_tokens(addon_id, status, created_at);

CREATE TABLE addon_grants (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    library_id TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, permission, library_id)
);

CREATE INDEX addon_grants_addon_idx
    ON addon_grants(addon_id, permission, library_id);

CREATE UNIQUE INDEX addon_grants_unique_scope_idx
    ON addon_grants(addon_id, permission, COALESCE(library_id, ''));


-- From 0022_addon_side_effects.sql
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


-- From 0023_addon_side_effect_apply_outcome.sql
ALTER TABLE addon_side_effects
    ADD COLUMN apply_status TEXT NOT NULL DEFAULT 'pending';

ALTER TABLE addon_side_effects
    ADD COLUMN apply_error_code TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_item_id TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_source TEXT;

ALTER TABLE addon_side_effects
    ADD COLUMN applied_at TEXT;

CREATE INDEX addon_side_effects_apply_status_idx
    ON addon_side_effects(apply_status, created_at, id);


-- From 0024_addon_side_effect_apply_report.sql
ALTER TABLE addon_side_effects
    ADD COLUMN apply_report_json TEXT;


-- From 0025_addon_artwork_candidates.sql
CREATE TABLE addon_artwork_candidates (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    side_effect_id TEXT NOT NULL UNIQUE REFERENCES addon_side_effects(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    source_kind TEXT NOT NULL CHECK(source_kind IN ('remote_url')),
    source_uri TEXT NOT NULL CHECK(length(source_uri) <= 2048),
    width INTEGER CHECK(width IS NULL OR (width BETWEEN 1 AND 20000)),
    height INTEGER CHECK(height IS NULL OR (height BETWEEN 1 AND 20000)),
    language TEXT CHECK(language IS NULL OR length(language) <= 32),
    status TEXT NOT NULL CHECK(status IN ('proposed', 'accepted', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, library_id, item_id, kind, kind_key, source_kind, source_uri)
);

CREATE INDEX addon_artwork_candidates_item_idx
    ON addon_artwork_candidates(item_id, status, kind, kind_key);


-- From 0026_managed_artwork_ingest.sql
CREATE TABLE managed_artwork_ingests (
    id TEXT PRIMARY KEY NOT NULL,
    candidate_id TEXT NOT NULL UNIQUE REFERENCES addon_artwork_candidates(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL UNIQUE REFERENCES jobs(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL CHECK(status IN ('queued', 'fetching', 'validating', 'stored', 'failed')),
    artifact_id TEXT,
    failure_code TEXT CHECK(failure_code IS NULL OR length(failure_code) <= 128),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX managed_artwork_ingests_item_idx
    ON managed_artwork_ingests(item_id, status, kind, kind_key);

CREATE INDEX managed_artwork_ingests_job_idx
    ON managed_artwork_ingests(job_id);

CREATE TABLE managed_artwork_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    ingest_id TEXT NOT NULL UNIQUE REFERENCES managed_artwork_ingests(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    storage_uri TEXT NOT NULL,
    content_hash TEXT,
    width INTEGER CHECK(width IS NULL OR (width BETWEEN 1 AND 20000)),
    height INTEGER CHECK(height IS NULL OR (height BETWEEN 1 AND 20000)),
    byte_len INTEGER CHECK(byte_len IS NULL OR byte_len >= 0),
    media_type TEXT CHECK(media_type IS NULL OR length(media_type) <= 128),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX managed_artwork_artifacts_item_idx
    ON managed_artwork_artifacts(item_id, kind, kind_key);


-- From 0027_selected_artwork_publication.sql
CREATE TABLE selected_artworks (
    id TEXT PRIMARY KEY NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    kind_key TEXT NOT NULL DEFAULT '',
    artifact_id TEXT NOT NULL REFERENCES managed_artwork_artifacts(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(item_id, kind, kind_key)
);

CREATE INDEX selected_artworks_artifact_idx
    ON selected_artworks(artifact_id);

CREATE INDEX selected_artworks_item_idx
    ON selected_artworks(item_id, kind, kind_key);


-- From 0028_managed_artwork_artifact_cleanup.sql
ALTER TABLE managed_artwork_artifacts
    ADD COLUMN deleted_at TEXT;

CREATE INDEX managed_artwork_artifacts_deleted_idx
    ON managed_artwork_artifacts(deleted_at, created_at, id);


-- From 0029_job_ownership_leases.sql
ALTER TABLE jobs ADD COLUMN worker_id TEXT;
ALTER TABLE jobs ADD COLUMN run_token TEXT;
ALTER TABLE jobs ADD COLUMN heartbeat_at TEXT;
ALTER TABLE jobs ADD COLUMN lease_expires_at TEXT;
ALTER TABLE jobs ADD COLUMN cancel_requested_at TEXT;
ALTER TABLE jobs ADD COLUMN cancel_reason TEXT;

CREATE INDEX jobs_lease_claim_idx
    ON jobs(status, kind, resource_class, queued_at, id);

CREATE INDEX jobs_lease_expiry_idx
    ON jobs(status, lease_expires_at);


-- From 0030_user_playback_states.sql
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


-- From 0031_managed_import_artifacts.sql
CREATE TABLE managed_import_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_kind_key TEXT NOT NULL DEFAULT '',
    source_uri TEXT NOT NULL,
    staging_manifest_id TEXT REFERENCES staging_manifest_records(id) ON DELETE SET NULL,
    artifact_uri TEXT,
    original_file_name TEXT,
    intended_locator TEXT,
    size_bytes INTEGER,
    fingerprint TEXT,
    state TEXT NOT NULL,
    diagnostics_json TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_uri),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX managed_import_artifacts_library_state_idx
    ON managed_import_artifacts(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX managed_import_artifacts_source_kind_idx
    ON managed_import_artifacts(target_library_id, source_kind, source_kind_key, source_uri);

CREATE INDEX managed_import_artifacts_staging_manifest_idx
    ON managed_import_artifacts(staging_manifest_id);

CREATE INDEX managed_import_artifacts_fingerprint_idx
    ON managed_import_artifacts(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;


-- From 0032_managed_import_promotion_applies.sql
CREATE TABLE managed_import_promotion_applies (
    id TEXT PRIMARY KEY NOT NULL,
    artifact_id TEXT NOT NULL REFERENCES managed_import_artifacts(id) ON DELETE CASCADE,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    requested_by TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    source_artifact_uri TEXT,
    destination_locator TEXT NOT NULL,
    accepted_plan_json TEXT NOT NULL,
    accepted_warnings_json TEXT,
    state TEXT NOT NULL,
    outcome_json TEXT,
    safe_error_code TEXT,
    safe_message TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, idempotency_key),
    CHECK (length(idempotency_key) > 0),
    CHECK (length(destination_locator) > 0),
    CHECK (length(accepted_plan_json) > 0)
);

CREATE INDEX managed_import_promotion_applies_artifact_idx
    ON managed_import_promotion_applies(artifact_id, updated_at_ms DESC, id);

CREATE INDEX managed_import_promotion_applies_library_state_idx
    ON managed_import_promotion_applies(target_library_id, state, updated_at_ms DESC, id);


-- From 0033_nfo_sidecar_applies.sql
CREATE TABLE nfo_sidecar_applies (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    media_item_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    media_source_id TEXT REFERENCES media_sources(id) ON DELETE SET NULL,
    requested_by TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    sidecar_locator TEXT NOT NULL,
    accepted_preview_json TEXT NOT NULL,
    accepted_warnings_json TEXT,
    policy_version TEXT NOT NULL,
    state TEXT NOT NULL,
    outcome_json TEXT,
    safe_error_code TEXT,
    safe_message TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, idempotency_key),
    CHECK (length(idempotency_key) > 0),
    CHECK (length(sidecar_locator) > 0),
    CHECK (length(accepted_preview_json) > 0),
    CHECK (length(policy_version) > 0)
);

CREATE INDEX nfo_sidecar_applies_item_idx
    ON nfo_sidecar_applies(media_item_id, updated_at_ms DESC, id);

CREATE INDEX nfo_sidecar_applies_source_idx
    ON nfo_sidecar_applies(media_source_id, updated_at_ms DESC, id);

CREATE INDEX nfo_sidecar_applies_library_state_idx
    ON nfo_sidecar_applies(target_library_id, state, updated_at_ms DESC, id);


-- From 0034_acquisition_intake_candidates.sql
CREATE TABLE acquisition_intake_candidates (
    id TEXT PRIMARY KEY NOT NULL,
    target_library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL,
    source_kind_key TEXT NOT NULL DEFAULT '',
    source_key TEXT NOT NULL,
    source_uri TEXT NOT NULL,
    display_name TEXT,
    intended_locator TEXT,
    size_bytes INTEGER,
    fingerprint TEXT,
    managed_import_artifact_id TEXT REFERENCES managed_import_artifacts(id) ON DELETE SET NULL,
    state TEXT NOT NULL,
    diagnostics_json TEXT,
    first_seen_at_ms INTEGER NOT NULL,
    last_seen_at_ms INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(target_library_id, source_kind, source_kind_key, source_key),
    CHECK (length(source_key) > 0),
    CHECK (length(source_uri) > 0),
    CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE INDEX acquisition_intake_candidates_library_state_idx
    ON acquisition_intake_candidates(target_library_id, state, updated_at_ms DESC, id);

CREATE INDEX acquisition_intake_candidates_source_kind_idx
    ON acquisition_intake_candidates(target_library_id, source_kind, source_kind_key, source_key);

CREATE INDEX acquisition_intake_candidates_managed_import_idx
    ON acquisition_intake_candidates(managed_import_artifact_id)
    WHERE managed_import_artifact_id IS NOT NULL;

CREATE INDEX acquisition_intake_candidates_fingerprint_idx
    ON acquisition_intake_candidates(target_library_id, fingerprint)
    WHERE fingerprint IS NOT NULL;


-- From 0035_addon_unregistration.sql
CREATE UNIQUE INDEX IF NOT EXISTS addon_registrations_active_manifest_idx
    ON addon_registrations(manifest_id)
    WHERE status <> 'unregistered';


-- From 0036_addon_routing_plans.sql
CREATE TABLE addon_routing_plans (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id TEXT NOT NULL,
    manifest_version TEXT NOT NULL,
    manifest_fingerprint TEXT NOT NULL,
    declaration_kind TEXT NOT NULL,
    declaration_id TEXT NOT NULL,
    status TEXT NOT NULL,
    target TEXT NOT NULL,
    safe_reason_code TEXT,
    job_kind TEXT,
    event_kind TEXT,
    plan_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, declaration_kind, declaration_id)
);

CREATE INDEX addon_routing_plans_addon_idx
    ON addon_routing_plans(addon_id, declaration_kind, declaration_id);

CREATE INDEX addon_routing_plans_status_idx
    ON addon_routing_plans(status, target, updated_at);


-- From 0037_addon_task_runs.sql
CREATE TABLE addon_task_runs (
    job_id TEXT PRIMARY KEY NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    manifest_id TEXT NOT NULL,
    manifest_version TEXT NOT NULL,
    manifest_fingerprint TEXT NOT NULL,
    declaration_id TEXT NOT NULL,
    declaration_name TEXT NOT NULL,
    declaration_path TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL,
    attempt INTEGER NOT NULL,
    max_attempts INTEGER,
    retry_of_job_id TEXT REFERENCES jobs(id) ON DELETE SET NULL,
    input_json TEXT NOT NULL,
    progress_json TEXT,
    result_json TEXT,
    safe_error_code TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(addon_id, idempotency_key)
);

CREATE INDEX addon_task_runs_addon_declaration_idx
    ON addon_task_runs(addon_id, declaration_id, created_at, job_id);

CREATE INDEX addon_task_runs_retry_idx
    ON addon_task_runs(retry_of_job_id);


-- From 0038_addon_outbound_task_dispatch_credentials.sql
ALTER TABLE addon_registrations
    ADD COLUMN outbound_task_dispatch_secret_env TEXT;


-- From 0039_addon_event_delivery_attempts.sql
CREATE TABLE addon_event_delivery_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    addon_id TEXT NOT NULL REFERENCES addon_registrations(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL REFERENCES event_outbox(id) ON DELETE CASCADE,
    declaration_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    http_status INTEGER,
    error TEXT,
    requested_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    next_retry_at TEXT,
    UNIQUE(addon_id, event_id, declaration_id, attempt_number)
);

CREATE INDEX addon_event_delivery_attempts_event_idx
    ON addon_event_delivery_attempts(event_id, requested_at);

CREATE INDEX addon_event_delivery_attempts_addon_idx
    ON addon_event_delivery_attempts(addon_id, event_id, declaration_id, requested_at);

CREATE INDEX addon_event_delivery_attempts_status_idx
    ON addon_event_delivery_attempts(status, next_retry_at);


-- From 0040_addon_event_delivery_leases.sql
ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN lease_expires_at TEXT;

CREATE INDEX addon_event_delivery_attempts_lease_idx
    ON addon_event_delivery_attempts(status, lease_expires_at);


-- From 0041_addon_event_forced_replay.sql
ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN forced_replay INTEGER NOT NULL DEFAULT 0;

ALTER TABLE addon_event_delivery_attempts
    ADD COLUMN replay_reason_code TEXT;


-- From 0042_admin_settings.sql
CREATE TABLE admin_metadata_raw_cache_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_ms INTEGER NOT NULL,
    cleanup_on_startup INTEGER NOT NULL,
    source TEXT NOT NULL,
    effect TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL
);


-- Identity and Library Access baseline.
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    principal_id TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL,
    normalized_username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (length(id) > 0),
    CHECK (length(principal_id) > 0),
    CHECK (length(username) > 0),
    CHECK (length(normalized_username) > 0),
    CHECK (length(display_name) > 0),
    CHECK (status IN ('active', 'disabled')),
    CHECK (created_at_ms >= 0),
    CHECK (updated_at_ms >= 0)
);

CREATE INDEX users_status_idx ON users(status, normalized_username);

CREATE TABLE local_user_credentials (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (length(password_hash) > 0),
    CHECK (updated_at_ms >= 0)
);

CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at_ms INTEGER NOT NULL,
    last_seen_at_ms INTEGER NOT NULL,
    expires_at_ms INTEGER NOT NULL,
    revoked_at_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (length(id) > 0),
    CHECK (length(token_hash) > 0),
    CHECK (created_at_ms >= 0),
    CHECK (last_seen_at_ms >= 0),
    CHECK (expires_at_ms >= 0),
    CHECK (revoked_at_ms IS NULL OR revoked_at_ms >= 0)
);

CREATE INDEX user_sessions_user_idx
    ON user_sessions(user_id, expires_at_ms);

CREATE INDEX user_sessions_active_idx
    ON user_sessions(expires_at_ms, user_id)
    WHERE revoked_at_ms IS NULL;

CREATE TABLE user_role_assignments (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    granted_at_ms INTEGER NOT NULL,
    PRIMARY KEY (user_id, role),
    CHECK (role IN ('administrator', 'library_manager', 'viewer')),
    CHECK (granted_at_ms >= 0)
);

CREATE INDEX user_role_assignments_role_idx
    ON user_role_assignments(role, user_id);

CREATE TABLE user_library_access_policies (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    access TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    PRIMARY KEY (user_id, library_id),
    CHECK (access IN ('none', 'browse', 'play', 'manage')),
    CHECK (created_at_ms >= 0),
    CHECK (updated_at_ms >= 0)
);

CREATE INDEX user_library_access_policies_library_idx
    ON user_library_access_policies(library_id, access);

CREATE TABLE role_library_access_policies (
    role TEXT NOT NULL,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    access TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    PRIMARY KEY (role, library_id),
    CHECK (role IN ('administrator', 'library_manager', 'viewer')),
    CHECK (access IN ('none', 'browse', 'play', 'manage')),
    CHECK (created_at_ms >= 0),
    CHECK (updated_at_ms >= 0)
);

CREATE INDEX role_library_access_policies_library_idx
    ON role_library_access_policies(library_id, access);
